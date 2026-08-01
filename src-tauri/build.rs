use std::{fs, path::PathBuf};

fn main() {
    let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.env");
    println!("cargo:rerun-if-changed={}", env_path.display());
    println!("cargo:rerun-if-env-changed=MODE");

    let mode = std::env::var("MODE")
        .ok()
        .or_else(|| {
            let contents = fs::read_to_string(&env_path)
                .unwrap_or_else(|error| panic!("{} o‘qilmadi: {error}", env_path.display()));
            contents
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .find_map(|line| line.strip_prefix("MODE=").map(str::trim))
                .map(str::to_owned)
        })
        .unwrap_or_else(|| panic!("MODE muhitda yoki .env ichida belgilanishi shart"))
        .to_uppercase();

    if mode != "DEV" && mode != "PROD" {
        panic!(".env MODE faqat DEV yoki PROD bo‘lishi mumkin, hozir: {mode}");
    }
    println!("cargo:rustc-env=APP_MODE={mode}");

    tauri_build::build()
}
