// =============================================================================
// UNOFFICIAL ZOLO - Profile Manager Module
// =============================================================================
// This module manages isolated WebView profiles for each account.
// Each profile has its own directory containing cookies, localStorage,
// IndexedDB, and cache - ensuring complete session isolation.
//
// ARCHITECTURAL DECISION: Profiles are stored in app data directory
// (~/.local/share/unofficial-zolo/profiles/{account_id}/) following XDG spec.
// This separation from config allows for different backup strategies.
// =============================================================================

use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use thiserror::Error;

/// Custom error types for profile operations
#[derive(Error, Debug)]
pub enum ProfileError {
    #[error("Failed to create profile directory: {0}")]
    CreateError(String),
    #[error("Failed to delete profile directory: {0}")]
    DeleteError(String),
    #[error("Profile not found: {0}")]
    NotFound(String),
}

impl From<ProfileError> for String {
    fn from(err: ProfileError) -> String {
        err.to_string()
    }
}

/// Manages WebView profile directories for account isolation
pub struct ProfileManager {
    profiles_dir: PathBuf,
}

impl ProfileManager {
    /// Creates a new ProfileManager for the given Tauri app
    pub fn new(app: &AppHandle) -> Self {
        let data_dir = app
            .path()
            .app_data_dir()
            .expect("Failed to get app data directory");

        Self {
            profiles_dir: data_dir.join("profiles"),
        }
    }

    /// Returns the base profiles directory
    pub fn get_profiles_dir(&self) -> &PathBuf {
        &self.profiles_dir
    }

    /// Gets the profile directory path for a specific account
    pub fn get_profile_path(&self, account_id: &str) -> PathBuf {
        self.profiles_dir.join(account_id)
    }

    /// Creates a new profile directory for an account
    /// Returns the path to the created profile directory
    pub fn create_profile(&self, account_id: &str) -> Result<PathBuf, ProfileError> {
        let profile_path = self.get_profile_path(account_id);

        fs::create_dir_all(&profile_path)
            .map_err(|e| ProfileError::CreateError(e.to_string()))?;

        Ok(profile_path)
    }

    /// Deletes a profile directory and all its contents
    /// This removes all cookies, localStorage, IndexedDB, and cache for the account
    pub fn delete_profile(&self, account_id: &str) -> Result<(), ProfileError> {
        let profile_path = self.get_profile_path(account_id);

        if !profile_path.exists() {
            // Profile doesn't exist, nothing to delete
            return Ok(());
        }

        fs::remove_dir_all(&profile_path)
            .map_err(|e| ProfileError::DeleteError(e.to_string()))?;

        Ok(())
    }

    /// Checks if a profile exists for an account
    pub fn profile_exists(&self, account_id: &str) -> bool {
        self.get_profile_path(account_id).exists()
    }

    /// Gets the size of a profile directory in bytes
    pub fn get_profile_size(&self, account_id: &str) -> Result<u64, ProfileError> {
        let profile_path = self.get_profile_path(account_id);

        if !profile_path.exists() {
            return Err(ProfileError::NotFound(account_id.to_string()));
        }

        fn dir_size(path: &PathBuf) -> u64 {
            let mut size = 0;
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        size += entry.metadata().map(|m| m.len()).unwrap_or(0);
                    } else if path.is_dir() {
                        size += dir_size(&path);
                    }
                }
            }
            size
        }

        Ok(dir_size(&profile_path))
    }

    /// Lists all existing profile IDs
    pub fn list_profiles(&self) -> Vec<String> {
        if !self.profiles_dir.exists() {
            return vec![];
        }

        fs::read_dir(&self.profiles_dir)
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| e.path().is_dir())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default()
    }
}
