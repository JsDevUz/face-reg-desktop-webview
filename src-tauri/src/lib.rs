use serde::Serialize;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tauri::{webview::PageLoadEvent, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const APP_URL: &str = "https://face-reg-cyan.vercel.app/";
const API_URL: &str = "https://api.tayin.uz";
const EPOS_URL: &str = "http://localhost:8347/uzpos";
const EPOS_TOKEN: &str = "DXJFX32CN1296678504F2";

#[derive(Default)]
struct PendingAuth(Mutex<Option<String>>);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginError {
    code: &'static str,
    message: String,
    store_name: Option<String>,
    terminal_id: Option<String>,
    allowed_terminal_ids: Vec<String>,
}

impl LoginError {
    fn simple(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            store_name: None,
            terminal_id: None,
            allowed_terminal_ids: Vec::new(),
        }
    }
}

fn employee_data(payload: &Value) -> Option<&Value> {
    payload.get("data").filter(|value| value.is_object())
}

fn has_terminal_permission(employee: &Value) -> bool {
    if employee
        .get("type")
        .or_else(|| employee.get("role_type"))
        .and_then(Value::as_str)
        == Some("SUPERADMIN")
    {
        return true;
    }

    employee
        .get("role_actions")
        .or_else(|| employee.get("permissions"))
        .and_then(Value::as_array)
        .is_some_and(|actions| {
            actions.iter().any(|action| {
                action.get("route").and_then(Value::as_str) == Some("check-terminal-id")
            })
        })
}

fn string_values(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|value| match value {
            Value::String(value) => Some(value.clone()),
            Value::Number(value) => Some(value.to_string()),
            _ => None,
        })
        .collect()
}

fn terminal_id(payload: &Value) -> Option<String> {
    let sender = payload.pointer("/message/Sender")?.as_object()?;
    ["ZReportFilesSent", "FullReceiptFilesSent", "TotalFilesSent"]
        .iter()
        .find_map(|key| {
            sender
                .get(*key)
                .and_then(Value::as_object)
                .filter(|report| !report.is_empty())
                .and_then(|report| report.keys().next().cloned())
        })
}

async fn validate_terminal(client: &reqwest::Client, employee: &Value) -> Result<(), LoginError> {
    if !has_terminal_permission(employee) {
        return Ok(());
    }

    let is_superadmin = employee
        .get("type")
        .or_else(|| employee.get("role_type"))
        .and_then(Value::as_str)
        == Some("SUPERADMIN");
    let store = employee.get("store").filter(|value| value.is_object());
    let store_name = store
        .and_then(|store| store.get("name"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let allowed_terminal_ids = string_values(
        store
            .and_then(|store| store.get("terminal_ids"))
            .or_else(|| employee.get("terminal_ids")),
    );

    let status = client
        .post(EPOS_URL)
        .json(&json!({ "token": EPOS_TOKEN, "method": "checkStatus" }))
        .send()
        .await;

    let status = match status {
        Ok(response) => response.json::<Value>().await.unwrap_or(Value::Null),
        Err(_) if !is_superadmin => {
            return Err(LoginError {
                code: "eposUnavailable",
                message: "EPOS terminaliga ulanib bo‘lmadi. Kirish bloklandi.".to_owned(),
                store_name,
                terminal_id: None,
                allowed_terminal_ids,
            });
        }
        Err(_) => return Ok(()),
    };

    // Pharma guard EPOS xizmatining o‘zi error qaytarsa foydalanuvchini
    // bloklamaydi; faqat lokal servisga umuman ulanib bo‘lmasa bloklaydi.
    if status.get("error").and_then(Value::as_bool) == Some(true) {
        return Ok(());
    }

    let terminal_response = client
        .post(EPOS_URL)
        .json(&json!({ "token": EPOS_TOKEN, "method": "getStatus" }))
        .send()
        .await;

    let terminal_response = match terminal_response {
        Ok(response) => response.json::<Value>().await.unwrap_or(Value::Null),
        Err(_) if !is_superadmin => {
            return Err(LoginError {
                code: "eposUnavailable",
                message: "EPOS terminaliga ulanib bo‘lmadi. Kirish bloklandi.".to_owned(),
                store_name,
                terminal_id: None,
                allowed_terminal_ids,
            });
        }
        Err(_) => return Ok(()),
    };

    // TerminalAccessGuard kabi terminal ID topilmasa kirishga ruxsat beradi.
    let Some(current_terminal_id) = terminal_id(&terminal_response) else {
        return Ok(());
    };

    if allowed_terminal_ids
        .iter()
        .any(|allowed| allowed == &current_terminal_id)
    {
        return Ok(());
    }

    Err(LoginError {
        code: "terminalMismatch",
        message: "Siz boshqa dorixonadasiz!".to_owned(),
        store_name,
        terminal_id: Some(current_terminal_id),
        allowed_terminal_ids,
    })
}

#[tauri::command]
async fn validate_and_open(
    token: String,
    webview: WebviewWindow,
    auth: tauri::State<'_, Arc<PendingAuth>>,
) -> Result<(), LoginError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|_| LoginError::simple("internal", "HTTP klientni yaratib bo‘lmadi"))?;

    let employee_response = client
        .get(format!("{API_URL}/v1/employee/info"))
        .bearer_auth(&token)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|_| LoginError::simple("connection", "Xodim ma’lumotini olib bo‘lmadi"))?;
    let employee_payload = employee_response
        .json::<Value>()
        .await
        .unwrap_or(Value::Null);
    let employee = employee_data(&employee_payload)
        .ok_or_else(|| LoginError::simple("invalidResponse", "Xodim ma’lumoti topilmadi"))?;

    validate_terminal(&client, employee).await?;

    *auth
        .0
        .lock()
        .map_err(|_| LoginError::simple("internal", "Ichki holat xatosi"))? = Some(token);

    let url = APP_URL
        .parse()
        .map_err(|_| LoginError::simple("internal", "FaceReg URL noto‘g‘ri"))?;
    webview.navigate(url).map_err(|error| {
        LoginError::simple(
            "navigation",
            format!("Web sahifani ochib bo‘lmadi: {error}"),
        )
    })
}

const BROWSER_GUARDS: &str = r#"
(() => {
  if (window.__FACE_REG_DESKTOP_GUARDS__) return;
  window.__FACE_REG_DESKTOP_GUARDS__ = true;

  window.addEventListener('contextmenu', (event) => event.preventDefault(), true);

  window.addEventListener('keydown', (event) => {
    const key = event.key.toLowerCase();
    const refresh = event.key === 'F5' || (event.ctrlKey && key === 'r');
    const devtools =
      event.key === 'F12' ||
      (event.ctrlKey && event.shiftKey && ['i', 'j', 'c'].includes(key));

    if (refresh) {
      event.preventDefault();
      event.stopImmediatePropagation();
      window.location.reload();
      return;
    }

    if (devtools) {
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  }, true);

  const originalOpen = window.open;
  window.open = (url) => {
    if (url) window.location.assign(url);
    return window;
  };

  if (location.hostname === 'face-reg-cyan.vercel.app') {
    const showRuntimeError = (value) => {
      const message = value?.stack || value?.message || String(value);
      let panel = document.getElementById('__face_reg_runtime_error__');
      if (!panel) {
        panel = document.createElement('pre');
        panel.id = '__face_reg_runtime_error__';
        Object.assign(panel.style, {
          position: 'fixed',
          inset: '16px',
          zIndex: '2147483647',
          margin: '0',
          padding: '20px',
          overflow: 'auto',
          border: '2px solid #dc3545',
          borderRadius: '12px',
          background: '#fff',
          color: '#991b1b',
          font: '13px/1.5 monospace',
          whiteSpace: 'pre-wrap'
        });
        document.documentElement.appendChild(panel);
      }
      panel.textContent = `FaceReg runtime xatosi:\n\n${message}`;
    };
    window.addEventListener('error', (event) => showRuntimeError(event.error || event.message));
    window.addEventListener('unhandledrejection', (event) => showRuntimeError(event.reason));
  }
})();
"#;

pub fn run() {
    let pending_auth = Arc::new(PendingAuth::default());
    let page_auth = pending_auth.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .manage(pending_auth)
        .invoke_handler(tauri::generate_handler![validate_and_open])
        .setup(|app| {
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("FaceReg")
                .inner_size(1280.0, 800.0)
                .min_inner_size(800.0, 600.0)
                .maximized(true)
                .devtools(cfg!(debug_assertions))
                .initialization_script(BROWSER_GUARDS)
                .on_page_load(move |webview, payload| {
                    if payload.event() != PageLoadEvent::Finished
                        || payload.url().origin().ascii_serialization()
                            != "https://face-reg-cyan.vercel.app"
                    {
                        return;
                    }

                    // Token faqat bir marta remote origin'ga uzatiladi. Web
                    // restoreSession tokenni rad etsa uni qayta-qayta yozish
                    // reload loop hosil qilmasligi kerak.
                    let token = page_auth.0.lock().ok().and_then(|mut guard| guard.take());
                    let Some(token) = token else {
                        return;
                    };

                    let persisted = json!({
                        "state": {
                            "token": token,
                            "isAuthenticated": true
                        },
                        "version": 0
                    })
                    .to_string();
                    let persisted_js = serde_json::to_string(&persisted)
                        .expect("serialized auth state must be valid JavaScript");

                    let script = format!(
                        r#"
                        (() => {{
                          const nextAuth = {persisted_js};
                          if (localStorage.getItem('face-reg-auth') !== nextAuth) {{
                            localStorage.setItem('face-reg-auth', nextAuth);
                            location.reload();
                          }}
                        }})();
                        "#
                    );
                    let _ = webview.eval(script);
                })
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running FaceReg");
}
