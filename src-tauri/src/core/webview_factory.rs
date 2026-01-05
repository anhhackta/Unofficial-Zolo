// =============================================================================
// UNOFFICIAL ZOLO - WebView Factory Module
// =============================================================================
// This module handles creation and management of WebView windows for accounts.
// Each account gets its own isolated WebView window with separate data storage.
//
// CRITICAL: This module only wraps the official Zalo Web URL.
// It does NOT inject JavaScript, modify network requests, or automate actions.
// Zalo Web is treated as a black box.
// =============================================================================

use std::sync::Mutex;
use std::collections::HashMap;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use url::Url;
use thiserror::Error;

use super::account_manager::Account;
use super::profile_manager::ProfileManager;

/// The official Zalo Web URL - the ONLY URL this app loads
const ZALO_WEB_URL: &str = "https://chat.zalo.me";

/// Custom error types for webview operations
#[derive(Error, Debug)]
pub enum WebviewError {
    #[error("Failed to create window: {0}")]
    CreateError(String),
    #[error("Window not found: {0}")]
    NotFound(String),
    #[error("Window already exists: {0}")]
    AlreadyExists(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

impl From<WebviewError> for String {
    fn from(err: WebviewError) -> String {
        err.to_string()
    }
}

/// Tracks open account windows
pub struct WindowTracker {
    /// Maps account ID to window label
    windows: Mutex<HashMap<String, String>>,
}

impl WindowTracker {
    pub fn new() -> Self {
        Self {
            windows: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, account_id: &str, label: &str) {
        let mut windows = self.windows.lock().unwrap();
        windows.insert(account_id.to_string(), label.to_string());
    }

    pub fn unregister(&self, account_id: &str) {
        let mut windows = self.windows.lock().unwrap();
        windows.remove(account_id);
    }

    pub fn get_label(&self, account_id: &str) -> Option<String> {
        let windows = self.windows.lock().unwrap();
        windows.get(account_id).cloned()
    }

    pub fn is_open(&self, account_id: &str) -> bool {
        let windows = self.windows.lock().unwrap();
        windows.contains_key(account_id)
    }
}

impl Default for WindowTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates and manages WebView windows for accounts
pub struct WebviewFactory;

impl WebviewFactory {
    /// Generates a unique window label for an account
    pub fn get_window_label(account_id: &str) -> String {
        format!("zolo-{}", account_id)
    }

    /// Opens a new WebView window for an account
    /// 
    /// The window is configured with:
    /// - Isolated data directory (cookies, localStorage, IndexedDB, cache)
    /// - Custom title showing account name
    /// - Default size of 1200x800
    /// 
    /// SECURITY NOTE: This only loads the official Zalo Web URL.
    /// No JavaScript injection or network interception is performed.
    pub fn open_account_window(
        app: &AppHandle,
        account: &Account,
        profile_manager: &ProfileManager,
    ) -> Result<(), WebviewError> {
        let label = Self::get_window_label(&account.id);

        // Check if window already exists
        if let Some(window) = app.get_webview_window(&label) {
            // Focus existing window
            let _ = window.set_focus();
            return Ok(());
        }

        // Create profile directory
        let profile_path = profile_manager
            .create_profile(&account.id)
            .map_err(|e| WebviewError::CreateError(e.to_string()))?;

        // Parse Zalo Web URL
        let url = Url::parse(ZALO_WEB_URL)
            .map_err(|e| WebviewError::InvalidUrl(e.to_string()))?;

        // Build the WebView window
        // 
        // ARCHITECTURAL NOTE: We use data_directory() to ensure complete
        // session isolation between accounts. Each account has its own:
        // - Cookies
        // - LocalStorage  
        // - IndexedDB
        // - Cache
        //
        // PROXY NOTE: Tauri v2 with WebKitGTK does not currently support
        // per-WebView proxy configuration. The proxy field in Account is
        // stored for future implementation or documentation purposes.
        let builder = WebviewWindowBuilder::new(
            app,
            &label,
            WebviewUrl::External(url),
        )
        .title(format!("Zolo - {}", account.name))
        .inner_size(1200.0, 800.0)
        .min_inner_size(800.0, 600.0)
        .data_directory(profile_path);

        builder
            .build()
            .map_err(|e| WebviewError::CreateError(e.to_string()))?;

        Ok(())
    }

    /// Closes an account's WebView window
    pub fn close_account_window(app: &AppHandle, account_id: &str) -> Result<(), WebviewError> {
        let label = Self::get_window_label(account_id);

        if let Some(window) = app.get_webview_window(&label) {
            window
                .close()
                .map_err(|e| WebviewError::CreateError(e.to_string()))?;
            Ok(())
        } else {
            Err(WebviewError::NotFound(account_id.to_string()))
        }
    }

    /// Checks if a window is open for an account
    pub fn is_window_open(app: &AppHandle, account_id: &str) -> bool {
        let label = Self::get_window_label(account_id);
        app.get_webview_window(&label).is_some()
    }
}
