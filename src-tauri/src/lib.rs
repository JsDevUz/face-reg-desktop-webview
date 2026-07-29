use tauri::{WebviewUrl, WebviewWindowBuilder};

const APP_URL: &str = "https://face-reg-cyan.vercel.app/";

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
    tauri::Builder::default()
        .setup(|app| {
            let url = APP_URL.parse().expect("FaceReg URL must be valid");

            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("FaceReg")
                .inner_size(1280.0, 800.0)
                .min_inner_size(800.0, 600.0)
                .maximized(true)
                .devtools(false)
                .initialization_script(BROWSER_GUARDS)
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running FaceReg");
}
