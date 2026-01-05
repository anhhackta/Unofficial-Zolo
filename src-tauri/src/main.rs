// =============================================================================
// UNOFFICIAL ZOLO - Main Entry Point
// =============================================================================
// Linux Desktop Multi-Account Zalo Web Manager
//
// LEGAL DISCLAIMER:
// This is an unofficial, community-maintained web client for Zalo Web.
// It is NOT affiliated with, endorsed by, or sponsored by Zalo or VNG Corporation.
//
// WHAT THIS APP DOES:
// - Wraps the official Zalo Web (https://chat.zalo.me) in a desktop window
// - Manages multiple accounts with isolated browser profiles
// - Persists sessions across restarts
//
// WHAT THIS APP DOES NOT DO:
// - Does NOT inject JavaScript into Zalo Web
// - Does NOT intercept or modify network requests
// - Does NOT automate login, messaging, or any actions
// - Does NOT scrape data from Zalo
// - Does NOT reverse engineer Zalo's protocols
//
// Zalo Web is treated as a complete black box.
// =============================================================================

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;

use core::{Account, AccountManager, ProfileManager, WebviewFactory};
use tauri::{AppHandle, Manager, State};
use std::sync::Mutex;

/// Application state shared across all commands
pub struct AppState {
    account_manager: Mutex<Option<AccountManager>>,
    profile_manager: Mutex<Option<ProfileManager>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            account_manager: Mutex::new(None),
            profile_manager: Mutex::new(None),
        }
    }

    fn init(&self, app: &AppHandle) {
        let mut am = self.account_manager.lock().unwrap();
        *am = Some(AccountManager::new(app));

        let mut pm = self.profile_manager.lock().unwrap();
        *pm = Some(ProfileManager::new(app));
    }
}

// =============================================================================
// Tauri Commands
// =============================================================================

/// Gets all registered accounts
#[tauri::command]
fn get_accounts(state: State<AppState>) -> Result<Vec<Account>, String> {
    let am = state.account_manager.lock().unwrap();
    let manager = am.as_ref().ok_or("App not initialized")?;
    manager.load_accounts().map_err(|e| e.to_string())
}

/// Adds a new account
#[tauri::command]
fn add_account(
    state: State<AppState>,
    name: String,
    proxy: Option<String>,
) -> Result<Account, String> {
    let am = state.account_manager.lock().unwrap();
    let manager = am.as_ref().ok_or("App not initialized")?;
    manager.add_account(name, proxy).map_err(|e| e.to_string())
}

/// Removes an account and its profile data
#[tauri::command]
fn remove_account(state: State<AppState>, id: String) -> Result<(), String> {
    // First, remove the profile data
    {
        let pm = state.profile_manager.lock().unwrap();
        let profile_manager = pm.as_ref().ok_or("App not initialized")?;
        profile_manager.delete_profile(&id).map_err(|e| e.to_string())?;
    }

    // Then remove the account record
    let am = state.account_manager.lock().unwrap();
    let manager = am.as_ref().ok_or("App not initialized")?;
    manager.remove_account(&id).map_err(|e| e.to_string())
}

/// Renames an account
#[tauri::command]
fn rename_account(
    state: State<AppState>,
    id: String,
    new_name: String,
) -> Result<Account, String> {
    let am = state.account_manager.lock().unwrap();
    let manager = am.as_ref().ok_or("App not initialized")?;
    manager.rename_account(&id, new_name).map_err(|e| e.to_string())
}

/// Updates an account's proxy configuration
#[tauri::command]
fn update_account_proxy(
    state: State<AppState>,
    id: String,
    proxy: Option<String>,
) -> Result<Account, String> {
    let am = state.account_manager.lock().unwrap();
    let manager = am.as_ref().ok_or("App not initialized")?;
    manager.update_proxy(&id, proxy).map_err(|e| e.to_string())
}

/// Opens an account's Zalo Web window
#[tauri::command]
async fn open_account(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<(), String> {
    let account = {
        let am = state.account_manager.lock().unwrap();
        let manager = am.as_ref().ok_or("App not initialized")?;
        manager.get_account(&id).map_err(|e| e.to_string())?
    };

    let pm = state.profile_manager.lock().unwrap();
    let profile_manager = pm.as_ref().ok_or("App not initialized")?;

    WebviewFactory::open_account_window(&app, &account, profile_manager)
        .map_err(|e| e.to_string())
}

/// Closes an account's window
#[tauri::command]
fn close_account(app: AppHandle, id: String) -> Result<(), String> {
    WebviewFactory::close_account_window(&app, &id).map_err(|e| e.to_string())
}

/// Checks if an account window is open
#[tauri::command]
fn is_account_open(app: AppHandle, id: String) -> bool {
    WebviewFactory::is_window_open(&app, &id)
}

// =============================================================================
// Main Application
// =============================================================================

fn main() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .setup(|app| {
            // Initialize managers with app handle
            let state: State<AppState> = app.state();
            state.init(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_accounts,
            add_account,
            remove_account,
            rename_account,
            update_account_proxy,
            open_account,
            close_account,
            is_account_open,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
