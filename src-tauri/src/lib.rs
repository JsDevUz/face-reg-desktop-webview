use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tauri::{webview::PageLoadEvent, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const APP_URL: &str = "https://face-reg-cyan.vercel.app/";
const API_URL: &str = "https://api.pharma-cosmos.uz:4443";

#[derive(Default)]
struct PendingAuth(Mutex<Option<String>>);

#[derive(Deserialize)]
struct LoginInput {
    phone: String,
    password: String,
}

fn find_token(value: &Value) -> Option<&str> {
    let object = value.as_object()?;
    ["access_token", "accessToken", "token"]
        .iter()
        .find_map(|key| object.get(*key).and_then(Value::as_str))
        .or_else(|| object.get("data").and_then(find_token))
}

fn api_error_message(value: &Value, status: reqwest::StatusCode) -> String {
    value
        .get("message")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| match status.as_u16() {
            401 | 403 => "Telefon raqam yoki parol noto‘g‘ri".to_owned(),
            404 => "Foydalanuvchi topilmadi".to_owned(),
            _ => format!("Server xatosi ({status})"),
        })
}

#[tauri::command]
async fn login_and_open(
    input: LoginInput,
    webview: WebviewWindow,
    auth: tauri::State<'_, Arc<PendingAuth>>,
) -> Result<(), String> {
    let response = reqwest::Client::new()
        .post(format!("{API_URL}/v1/login"))
        .header("Accept", "application/json")
        .json(&json!({
            "phone": input.phone,
            "password": input.password,
        }))
        .send()
        .await
        .map_err(|_| "Server bilan bog‘lanib bo‘lmadi".to_owned())?;

    let status = response.status();
    let payload: Value = response.json().await.unwrap_or(Value::Null);
    if !status.is_success() {
        return Err(api_error_message(&payload, status));
    }

    let token = find_token(&payload)
        .ok_or_else(|| "Server token qaytarmadi".to_owned())?
        .to_owned();

    *auth.0.lock().map_err(|_| "Ichki holat xatosi".to_owned())? = Some(token);

    let url = APP_URL
        .parse()
        .map_err(|_| "FaceReg URL noto‘g‘ri".to_owned())?;
    webview
        .navigate(url)
        .map_err(|error| format!("Web sahifani ochib bo‘lmadi: {error}"))
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
})();
"#;

pub fn run() {
    let pending_auth = Arc::new(PendingAuth::default());
    let page_auth = pending_auth.clone();

    tauri::Builder::default()
        .manage(pending_auth)
        .invoke_handler(tauri::generate_handler![login_and_open])
        .setup(|app| {
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("FaceReg")
                .inner_size(1280.0, 800.0)
                .min_inner_size(800.0, 600.0)
                .maximized(true)
                .devtools(false)
                .initialization_script(BROWSER_GUARDS)
                .on_page_load(move |webview, payload| {
                    if payload.event() != PageLoadEvent::Finished
                        || payload.url().origin().ascii_serialization()
                            != "https://face-reg-cyan.vercel.app"
                    {
                        return;
                    }

                    let token = page_auth.0.lock().ok().and_then(|guard| guard.clone());
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
