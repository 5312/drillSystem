// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod license;
mod machine_code;

use license::{
    export_public_key, generate_license, generate_license_with_machine_code, generate_new_key_pair,
    get_all_licenses, validate_license, validate_license_with_machine_code, LicenseInfo,
    LicenseValidationResult,
};
use machine_code::get_machine_id;
use tauri_plugin_updater::UpdaterExt;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn generate_license_key(
    customer_name: &str,
    customer_email: &str,
    expiry_days: u32,
    features: Vec<String>,
) -> Result<String, String> {
    generate_license(customer_name, customer_email, expiry_days, features)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn generate_license_key_with_machine_code(
    customer_name: &str,
    customer_email: &str,
    expiry_days: u32,
    features: Vec<String>,
    machine_code: &str,
) -> Result<String, String> {
    generate_license_with_machine_code(
        customer_name,
        customer_email,
        expiry_days,
        features,
        machine_code,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn validate_license_key(license_key: &str) -> Result<LicenseValidationResult, String> {
    validate_license(license_key).map_err(|e| e.to_string())
}

#[tauri::command]
fn validate_license_key_with_machine_code(
    license_key: &str,
    machine_code: &str,
) -> Result<LicenseValidationResult, String> {
    validate_license_with_machine_code(license_key, machine_code).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_licenses() -> Result<Vec<LicenseInfo>, String> {
    get_all_licenses().map_err(|e| e.to_string())
}

#[tauri::command]
fn export_license_public_key() -> String {
    export_public_key()
}

#[tauri::command]
fn generate_rsa_key_pair(bits: usize) -> Result<(String, String), String> {
    generate_new_key_pair(bits).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_current_machine_id() -> Result<String, String> {
    get_machine_id().map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_license_by_id(license_id: &str) -> Result<(), String> {
    license::delete_license(license_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_update(app: tauri::AppHandle) -> Result<String, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;

    let update_response = updater.check().await.map_err(|e| e.to_string())?;

    if update_response.is_some() {
        Ok("有更新可用".to_string())
    } else {
        Ok("当前已是最新版本".to_string())
    }
}

#[tauri::command]
async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;

    // 检查是否有可用更新
    let update = updater.check().await.map_err(|e| e.to_string())?;

    if let Some(update) = update {
        // 下载并安装更新
        // 第一个参数是进度回调函数，第二个参数是退出前的回调函数
        update
            .download_and_install(|_, _| {}, || {})
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("没有可用的更新".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            generate_license_key,
            validate_license_key,
            get_licenses,
            export_license_public_key,
            generate_rsa_key_pair,
            generate_license_key_with_machine_code,
            validate_license_key_with_machine_code,
            get_current_machine_id,
            delete_license_by_id,
            check_update,
            install_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
