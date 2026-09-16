use std::{fs, path::PathBuf};

fn main() {
    let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.env");
    println!("cargo:rerun-if-changed={}", env_path.display());
    println!("cargo:rerun-if-env-changed=MODE");
    println!("cargo:rerun-if-env-changed=VITE_MODE");
    println!("cargo:rerun-if-env-changed=APP_URL");
    println!("cargo:rerun-if-env-changed=VERCEL_URL");
    println!("cargo:rerun-if-env-changed=API_BASE_URL");
    println!("cargo:rerun-if-env-changed=API_URL");

    let env_file = fs::read_to_string(&env_path).unwrap_or_default();
    let get_val = |key: &str| -> Option<String> {
        std::env::var(key).ok().or_else(|| {
            env_file
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .find_map(|line| {
                    let prefix = format!("{key}=");
                    line.strip_prefix(&prefix).map(|v| {
                        v.trim_matches('"').trim_matches('\'').trim().to_owned()
                    })
                })
        })
    };

    let mode = get_val("MODE")
        .or_else(|| get_val("VITE_MODE"))
        .unwrap_or_else(|| "PROD".to_owned())
        .to_uppercase();

    let app_url = get_val("APP_URL")
        .or_else(|| get_val("VERCEL_URL").map(|u| {
            if u.starts_with("http") { u } else { format!("https://{u}") }
        }))
        .or_else(|| get_val("WEB_URL"))
        .unwrap_or_else(|| "https://face-reg-cyan.vercel.app/".to_owned());

    let api_url = get_val("API_BASE_URL")
        .or_else(|| get_val("API_URL"))
        .or_else(|| get_val("VITE_API_BASE_URL"))
        .unwrap_or_else(|| {
            if mode == "DEV" {
                "https://api.tayin.uz".to_owned()
            } else {
                "https://api.pharma-cosmos.uz:4443".to_owned()
            }
        });

    println!("cargo:rustc-env=APP_MODE={mode}");
    println!("cargo:rustc-env=APP_URL={app_url}");
    println!("cargo:rustc-env=API_URL={api_url}");

    tauri_build::build()
}
