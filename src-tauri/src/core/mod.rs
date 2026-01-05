// =============================================================================
// UNOFFICIAL ZOLO - Core Module
// =============================================================================
// This module exports all core functionality for the application.
// =============================================================================

pub mod account_manager;
pub mod profile_manager;
pub mod proxy_config;
pub mod webview_factory;

// Re-export commonly used types
pub use account_manager::{Account, AccountError, AccountManager};
pub use profile_manager::{ProfileError, ProfileManager};
pub use proxy_config::{ProxyConfig, ProxyError, ProxyType};
pub use webview_factory::{WebviewError, WebviewFactory, WindowTracker};
