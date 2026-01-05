// =============================================================================
// UNOFFICIAL ZOLO - Account Manager Module
// =============================================================================
// This module handles all account-related operations including CRUD operations,
// persistence, and account data management.
//
// ARCHITECTURAL DECISION: Accounts are stored as JSON in the app config directory
// (~/.config/unofficial-zolo/accounts.json) for easy backup and portability.
// =============================================================================

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use thiserror::Error;

/// Represents a Zalo account configuration.
/// Each account has its own isolated WebView profile.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    /// Unique identifier (UUID v4)
    pub id: String,
    /// User-friendly display name
    pub name: String,
    /// Optional proxy configuration string
    /// Format: "http://user:pass@host:port" or "socks5://host:port"
    /// 
    /// WARNING: Proxy support is experimental. WebKitGTK does not natively
    /// support per-WebView proxy configuration. This field is stored for
    /// future implementation or system-wide proxy hints.
    pub proxy: Option<String>,
    /// Whether the account window is currently open
    #[serde(default)]
    pub is_open: bool,
}

/// Custom error types for account operations
#[derive(Error, Debug)]
pub enum AccountError {
    #[error("Failed to read accounts file: {0}")]
    ReadError(String),
    #[error("Failed to write accounts file: {0}")]
    WriteError(String),
    #[error("Failed to parse accounts JSON: {0}")]
    ParseError(String),
    #[error("Account not found: {0}")]
    NotFound(String),
    #[error("Account name already exists: {0}")]
    DuplicateName(String),
}

impl From<AccountError> for String {
    fn from(err: AccountError) -> String {
        err.to_string()
    }
}

/// Manages all account operations
pub struct AccountManager {
    config_path: PathBuf,
}

impl AccountManager {
    /// Creates a new AccountManager for the given Tauri app
    pub fn new(app: &AppHandle) -> Self {
        let config_dir = app
            .path()
            .app_config_dir()
            .expect("Failed to get app config directory");
        
        Self {
            config_path: config_dir.join("accounts.json"),
        }
    }

    /// Returns the path to the accounts file
    pub fn get_accounts_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Loads all accounts from disk
    pub fn load_accounts(&self) -> Result<Vec<Account>, AccountError> {
        if !self.config_path.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| AccountError::ReadError(e.to_string()))?;

        let accounts: Vec<Account> = serde_json::from_str(&content)
            .map_err(|e| AccountError::ParseError(e.to_string()))?;

        Ok(accounts)
    }

    /// Saves all accounts to disk
    pub fn save_accounts(&self, accounts: &[Account]) -> Result<(), AccountError> {
        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AccountError::WriteError(e.to_string()))?;
        }

        let content = serde_json::to_string_pretty(accounts)
            .map_err(|e| AccountError::WriteError(e.to_string()))?;

        fs::write(&self.config_path, content)
            .map_err(|e| AccountError::WriteError(e.to_string()))?;

        Ok(())
    }

    /// Adds a new account
    pub fn add_account(&self, name: String, proxy: Option<String>) -> Result<Account, AccountError> {
        let mut accounts = self.load_accounts()?;

        // Check for duplicate names
        if accounts.iter().any(|a| a.name == name) {
            return Err(AccountError::DuplicateName(name));
        }

        let new_account = Account {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            proxy,
            is_open: false,
        };

        accounts.push(new_account.clone());
        self.save_accounts(&accounts)?;

        Ok(new_account)
    }

    /// Removes an account by ID
    pub fn remove_account(&self, id: &str) -> Result<(), AccountError> {
        let mut accounts = self.load_accounts()?;
        let original_len = accounts.len();

        accounts.retain(|a| a.id != id);

        if accounts.len() == original_len {
            return Err(AccountError::NotFound(id.to_string()));
        }

        self.save_accounts(&accounts)?;
        Ok(())
    }

    /// Renames an account
    pub fn rename_account(&self, id: &str, new_name: String) -> Result<Account, AccountError> {
        let mut accounts = self.load_accounts()?;

        // Check for duplicate names (excluding current account)
        if accounts.iter().any(|a| a.name == new_name && a.id != id) {
            return Err(AccountError::DuplicateName(new_name));
        }

        let account = accounts
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| AccountError::NotFound(id.to_string()))?;

        account.name = new_name;
        let updated_account = account.clone();

        self.save_accounts(&accounts)?;
        Ok(updated_account)
    }

    /// Updates an account's proxy configuration
    pub fn update_proxy(&self, id: &str, proxy: Option<String>) -> Result<Account, AccountError> {
        let mut accounts = self.load_accounts()?;

        let account = accounts
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| AccountError::NotFound(id.to_string()))?;

        account.proxy = proxy;
        let updated_account = account.clone();

        self.save_accounts(&accounts)?;
        Ok(updated_account)
    }

    /// Gets an account by ID
    pub fn get_account(&self, id: &str) -> Result<Account, AccountError> {
        let accounts = self.load_accounts()?;

        accounts
            .into_iter()
            .find(|a| a.id == id)
            .ok_or_else(|| AccountError::NotFound(id.to_string()))
    }

    /// Updates the is_open status of an account
    pub fn set_open_status(&self, id: &str, is_open: bool) -> Result<(), AccountError> {
        let mut accounts = self.load_accounts()?;

        let account = accounts
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| AccountError::NotFound(id.to_string()))?;

        account.is_open = is_open;
        self.save_accounts(&accounts)?;

        Ok(())
    }
}
