// =============================================================================
// UNOFFICIAL ZOLO - Proxy Configuration Module
// =============================================================================
// This module handles proxy configuration for accounts.
//
// ⚠️ IMPORTANT LIMITATION ⚠️
// Tauri v2 with WebKitGTK does NOT natively support per-WebView proxy
// configuration. This module provides:
// 1. Data structures for storing proxy configuration per account
// 2. Validation for proxy URLs
// 3. Documentation for future implementation
//
// Current status: EXPERIMENTAL / DOCUMENTATION ONLY
// The proxy settings are stored but NOT applied to WebView connections.
//
// Potential future approaches:
// 1. System-wide proxy (affects all connections, not per-account)
// 2. Custom WebKitGTK network context configuration (complex, platform-specific)
// 3. External proxy manager integration
// =============================================================================

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Supported proxy types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProxyType {
    Http,
    Https,
    Socks5,
}

impl std::fmt::Display for ProxyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyType::Http => write!(f, "http"),
            ProxyType::Https => write!(f, "https"),
            ProxyType::Socks5 => write!(f, "socks5"),
        }
    }
}

/// Proxy configuration for an account
/// 
/// ⚠️ SECURITY WARNING ⚠️
/// Using proxies with messaging applications carries risks:
/// 1. Proxy operators can potentially see unencrypted traffic
/// 2. Some proxies may log connection metadata
/// 3. Misconfigured proxies can leak your real IP
/// 
/// Only use trusted proxies. This feature is provided for users who
/// understand the implications and have specific requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Proxy type (HTTP, HTTPS, SOCKS5)
    pub proxy_type: ProxyType,
    /// Proxy host (IP or hostname)
    pub host: String,
    /// Proxy port
    pub port: u16,
    /// Optional username for authentication
    pub username: Option<String>,
    /// Optional password for authentication
    /// 
    /// ⚠️ SECURITY NOTE: Password is stored in plain text in config.
    /// Consider using environment variables or a secure credential store
    /// for production deployments.
    pub password: Option<String>,
}

/// Custom error types for proxy operations
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("Invalid proxy URL: {0}")]
    InvalidUrl(String),
    #[error("Unsupported proxy type: {0}")]
    UnsupportedType(String),
    #[error("Invalid port: {0}")]
    InvalidPort(String),
    #[error("Missing host")]
    MissingHost,
}

impl From<ProxyError> for String {
    fn from(err: ProxyError) -> String {
        err.to_string()
    }
}

impl ProxyConfig {
    /// Creates a new ProxyConfig
    pub fn new(proxy_type: ProxyType, host: String, port: u16) -> Self {
        Self {
            proxy_type,
            host,
            port,
            username: None,
            password: None,
        }
    }

    /// Creates a ProxyConfig with authentication
    pub fn with_auth(
        proxy_type: ProxyType,
        host: String,
        port: u16,
        username: String,
        password: String,
    ) -> Self {
        Self {
            proxy_type,
            host,
            port,
            username: Some(username),
            password: Some(password),
        }
    }

    /// Parses a proxy URL string into a ProxyConfig
    /// 
    /// Supported formats:
    /// - http://host:port
    /// - http://user:pass@host:port
    /// - socks5://host:port
    /// - socks5://user:pass@host:port
    pub fn from_url(url: &str) -> Result<Self, ProxyError> {
        let parsed = url::Url::parse(url)
            .map_err(|e| ProxyError::InvalidUrl(e.to_string()))?;

        let proxy_type = match parsed.scheme() {
            "http" => ProxyType::Http,
            "https" => ProxyType::Https,
            "socks5" => ProxyType::Socks5,
            other => return Err(ProxyError::UnsupportedType(other.to_string())),
        };

        let host = parsed
            .host_str()
            .ok_or(ProxyError::MissingHost)?
            .to_string();

        let port = parsed
            .port()
            .ok_or_else(|| ProxyError::InvalidPort("Port required".to_string()))?;

        let username = if parsed.username().is_empty() {
            None
        } else {
            Some(parsed.username().to_string())
        };

        let password = parsed.password().map(|p| p.to_string());

        Ok(Self {
            proxy_type,
            host,
            port,
            username,
            password,
        })
    }

    /// Converts the ProxyConfig back to a URL string
    pub fn to_url(&self) -> String {
        let auth = match (&self.username, &self.password) {
            (Some(user), Some(pass)) => format!("{}:{}@", user, pass),
            (Some(user), None) => format!("{}@", user),
            _ => String::new(),
        };

        format!("{}://{}{}:{}", self.proxy_type, auth, self.host, self.port)
    }

    /// Validates the proxy configuration
    pub fn validate(&self) -> Result<(), ProxyError> {
        if self.host.is_empty() {
            return Err(ProxyError::MissingHost);
        }

        if self.port == 0 {
            return Err(ProxyError::InvalidPort("Port cannot be 0".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_proxy() {
        let config = ProxyConfig::from_url("http://proxy.example.com:8080").unwrap();
        assert_eq!(config.proxy_type, ProxyType::Http);
        assert_eq!(config.host, "proxy.example.com");
        assert_eq!(config.port, 8080);
        assert!(config.username.is_none());
    }

    #[test]
    fn test_parse_socks5_with_auth() {
        let config = ProxyConfig::from_url("socks5://user:pass@localhost:1080").unwrap();
        assert_eq!(config.proxy_type, ProxyType::Socks5);
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 1080);
        assert_eq!(config.username, Some("user".to_string()));
        assert_eq!(config.password, Some("pass".to_string()));
    }
}
