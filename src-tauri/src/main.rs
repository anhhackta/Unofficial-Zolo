#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub proxy: Option<String>,
}

fn get_accounts_path(app: &AppHandle) -> PathBuf {
    app.path().app_config_dir().unwrap().join("accounts.json")
}

#[tauri::command]
fn get_accounts(app: AppHandle) -> Result<Vec<Account>, String> {
    let path = get_accounts_path(&app);
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let accounts: Vec<Account> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(accounts)
}

#[tauri::command]
fn add_account(app: AppHandle, name: String, proxy: Option<String>) -> Result<Account, String> {
    let mut accounts = get_accounts(app.clone())?;
    let id = uuid::Uuid::new_v4().to_string();
    let new_account = Account {
        id,
        name,
        proxy,
    };
    accounts.push(new_account.clone());
    save_accounts(&app, &accounts)?;
    Ok(new_account)
}

#[tauri::command]
fn remove_account(app: AppHandle, id: String) -> Result<(), String> {
    let mut accounts = get_accounts(app.clone())?;
    accounts.retain(|a| a.id != id);
    save_accounts(&app, &accounts)?;
    Ok(())
}

#[tauri::command]
async fn open_account(app: AppHandle, id: String) -> Result<(), String> {
    let accounts = get_accounts(app.clone())?;
    let account = accounts.iter().find(|a| a.id == id).ok_or("Account not found")?;
    let label = format!("zolo-{}", id);

    if let Some(window) = app.get_webview_window(&label) {
        let _ = window.set_focus();
        return Ok(());
    }

    let app_data_dir = app.path().app_data_dir().unwrap();
    let account_dir = app_data_dir.join("profiles").join(&id);
    fs::create_dir_all(&account_dir).map_err(|e| e.to_string())?;

    let url = Url::parse("https://chat.zalo.me").unwrap();
    
    let mut builder = WebviewWindowBuilder::new(
        &app,
        &label,
        WebviewUrl::External(url)
    )
    .title(format!("Zolo - {}", account.name))
    .data_directory(account_dir);
    
    // Note: If proxy needs implementation, Tauri v2 might support it via other means or specialized plugins.
    // For now we assume vanilla networking but isolated storage.

    builder.build().map_err(|e| e.to_string())?;

    Ok(())
}

fn save_accounts(app: &AppHandle, accounts: &Vec<Account>) -> Result<(), String> {
    let path = get_accounts_path(app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(accounts).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_accounts, add_account, remove_account, open_account])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
