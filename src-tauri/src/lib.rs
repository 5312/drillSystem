// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod license;

use license::{generate_license, validate_license, get_all_licenses, export_public_key, generate_new_key_pair, LicenseInfo, LicenseValidationResult};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn generate_license_key(customer_name: &str, customer_email: &str, expiry_days: u32, features: Vec<String>) -> Result<String, String> {
    generate_license(customer_name, customer_email, expiry_days, features)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn validate_license_key(license_key: &str) -> Result<LicenseValidationResult, String> {
    validate_license(license_key)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_licenses() -> Result<Vec<LicenseInfo>, String> {
    get_all_licenses()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn export_license_public_key() -> String {
    export_public_key()
}

#[tauri::command]
fn generate_rsa_key_pair(bits: usize) -> Result<(String, String), String> {
    generate_new_key_pair(bits).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            generate_license_key,
            validate_license_key,
            get_licenses,
            export_license_public_key,
            generate_rsa_key_pair
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
