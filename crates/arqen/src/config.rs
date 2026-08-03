//! Configuration module for Arqen.
//!
//! Provides typed configuration with env/file loading, validation, and secret redaction.
//!
//! # Environment Variables
//!
//! All env vars use the `ARQEN_` prefix:
//! - `ARQEN_HOST` - Server host (default: 127.0.0.1)
//! - `ARQEN_PORT` - Server port (default: 3000)
//! - `ARQEN_STORAGE_MODE` - Storage mode: memory, persistent, http (default: memory)
//! - `ARQEN_PERSISTENT_PATH` - Path for persistent storage
//! - `ARQEN_THINGD_URL` - Thingd HTTP URL
//! - `ARQEN_JWT_SECRET` - JWT secret for authentication
//! - `ARQEN_LOG_LEVEL` - Log level (default: info)
//! - `ARQEN_LOG_FORMAT` - Log format: pretty, json, compact (default: pretty)

use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use crate::core::{AppError, ErrorKind};

/// A wrapper that redacts sensitive values in Display/Debug output.
#[derive(Serialize, Deserialize)]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    /// Create a new secret value.
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Unwrap the secret value.
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Get a reference to the secret value.
    pub fn inner(&self) -> &T {
        &self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl<T: fmt::Display> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl<T: Clone> Clone for Secret<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: PartialEq> PartialEq for Secret<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Eq> Eq for Secret<T> {}

use serde::{Deserialize, Serialize};

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host (default: 127.0.0.1).
    #[serde(default = "default_host")]
    pub host: String,
    /// Server port (default: 3000).
    #[serde(default = "default_port")]
    pub port: u16,
    /// Request timeout (default: 30s).
    #[serde(default = "default_request_timeout")]
    pub request_timeout: Duration,
    /// Maximum request body size in bytes (default: 1MB).
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_request_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_max_body_size() -> usize {
    1024 * 1024
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            request_timeout: default_request_timeout(),
            max_body_size: default_max_body_size(),
        }
    }
}

/// Storage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage mode (default: memory).
    #[serde(default)]
    pub mode: StorageMode,
    /// Path for persistent storage.
    pub persistent_path: Option<PathBuf>,
    /// Thingd HTTP URL.
    pub http_url: Option<String>,
}

/// Storage mode selector.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StorageMode {
    /// In-memory storage (default).
    #[default]
    Memory,
    /// Persistent local storage.
    Persistent,
    /// Remote thingd HTTP storage.
    Http,
}

impl StorageMode {
    /// Parse a storage mode from a string.
    pub fn from_str(s: &str) -> Result<Self, ConfigError> {
        match s.to_lowercase().as_str() {
            "memory" => Ok(Self::Memory),
            "persistent" => Ok(Self::Persistent),
            "http" => Ok(Self::Http),
            _ => Err(ConfigError::InvalidValue {
                field: "storage_mode".to_string(),
                value: s.to_string(),
                expected: "memory, persistent, or http".to_string(),
            }),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            mode: StorageMode::Memory,
            persistent_path: None,
            http_url: None,
        }
    }
}

/// Authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Whether authentication is enabled (default: false).
    #[serde(default)]
    pub enabled: bool,
    /// JWT secret for token validation.
    pub jwt_secret: Option<Secret<String>>,
    /// API key header name (default: X-API-Key).
    #[serde(default = "default_api_key_header")]
    pub api_key_header: String,
}

fn default_api_key_header() -> String {
    "X-API-Key".to_string()
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            jwt_secret: None,
            api_key_header: default_api_key_header(),
        }
    }
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (default: info).
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Log format (default: pretty).
    #[serde(default)]
    pub format: LogFormat,
}

fn default_log_level() -> String {
    "info".to_string()
}

/// Log format selector.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Pretty-printed output (default).
    #[default]
    Pretty,
    /// JSON structured output.
    Json,
    /// Compact output.
    Compact,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: LogFormat::Pretty,
        }
    }
}

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Server configuration.
    #[serde(default)]
    pub server: ServerConfig,
    /// Storage configuration.
    #[serde(default)]
    pub storage: StorageConfig,
    /// Authentication configuration.
    #[serde(default)]
    pub auth: AuthConfig,
    /// Logging configuration.
    #[serde(default)]
    pub logging: LoggingConfig,
}

impl AppConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Server config
        if let Ok(host) = std::env::var("ARQEN_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = std::env::var("ARQEN_PORT") {
            config.server.port = port
                .parse()
                .map_err(|_| ConfigError::InvalidValue {
                    field: "port".to_string(),
                    value: port,
                    expected: "a valid u16".to_string(),
                })?;
        }

        // Storage config
        if let Ok(mode) = std::env::var("ARQEN_STORAGE_MODE") {
            config.storage.mode = StorageMode::from_str(&mode)?;
        }
        if let Ok(path) = std::env::var("ARQEN_PERSISTENT_PATH") {
            config.storage.persistent_path = Some(PathBuf::from(path));
        }
        if let Ok(url) = std::env::var("ARQEN_THINGD_URL") {
            config.storage.http_url = Some(url);
        }

        // Auth config
        if let Ok(secret) = std::env::var("ARQEN_JWT_SECRET") {
            config.auth.enabled = true;
            config.auth.jwt_secret = Some(Secret::new(secret));
        }
        if let Ok(header) = std::env::var("ARQEN_API_KEY_HEADER") {
            config.auth.api_key_header = header;
        }

        // Logging config
        if let Ok(level) = std::env::var("ARQEN_LOG_LEVEL") {
            config.logging.level = level;
        }
        if let Ok(format) = std::env::var("ARQEN_LOG_FORMAT") {
            config.logging.format = match format.to_lowercase().as_str() {
                "pretty" => LogFormat::Pretty,
                "json" => LogFormat::Json,
                "compact" => LogFormat::Compact,
                _ => return Err(ConfigError::InvalidValue {
                    field: "log_format".to_string(),
                    value: format,
                    expected: "pretty, json, or compact".to_string(),
                }),
            };
        }

        config.validate()?;
        Ok(config)
    }

    /// Load configuration from a TOML file.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| ConfigError::FileError {
            path: path.as_ref().to_path_buf(),
            source: e,
        })?;
        let config: Self = toml::from_str(&content).map_err(ConfigError::ParseError)?;
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::InvalidValue {
                field: "port".to_string(),
                value: "0".to_string(),
                expected: "a non-zero port number".to_string(),
            });
        }

        if self.storage.mode == StorageMode::Persistent && self.storage.persistent_path.is_none() {
            return Err(ConfigError::MissingField {
                field: "persistent_path".to_string(),
                context: "required when storage mode is persistent".to_string(),
            });
        }

        if self.storage.mode == StorageMode::Http && self.storage.http_url.is_none() {
            return Err(ConfigError::MissingField {
                field: "http_url".to_string(),
                context: "required when storage mode is http".to_string(),
            });
        }

        Ok(())
    }

    /// Get the server address as a SocketAddr.
    pub fn address(&self) -> Result<std::net::SocketAddr, ConfigError> {
        format!("{}:{}", self.server.host, self.server.port)
            .parse()
            .map_err(|_| ConfigError::InvalidValue {
                field: "address".to_string(),
                value: format!("{}:{}", self.server.host, self.server.port),
                expected: "a valid socket address".to_string(),
            })
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            storage: StorageConfig::default(),
            auth: AuthConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

/// Configuration error type.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Invalid configuration value.
    #[error("invalid value for {field}: '{value}' (expected {expected})")]
    InvalidValue {
        field: String,
        value: String,
        expected: String,
    },

    /// Missing required field.
    #[error("missing required field '{field}': {context}")]
    MissingField { field: String, context: String },

    /// File read error.
    #[error("failed to read config file {}: {source}", path.display())]
    FileError {
        path: PathBuf,
        source: std::io::Error,
    },

    /// TOML parse error.
    #[error("failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
}

impl From<ConfigError> for AppError {
    fn from(e: ConfigError) -> Self {
        AppError::new(ErrorKind::Validation, e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_redaction() {
        let secret = Secret::new("my-secret-key".to_string());
        assert_eq!(format!("{}", secret), "[REDACTED]");
        assert_eq!(format!("{:?}", secret), "[REDACTED]");
        assert_eq!(secret.into_inner(), "my-secret-key");
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.storage.mode, StorageMode::Memory);
        assert!(!config.auth.enabled);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        assert!(config.validate().is_ok());

        config.server.port = 0;
        assert!(config.validate().is_err());

        config.server.port = 3000;
        config.storage.mode = StorageMode::Persistent;
        assert!(config.validate().is_err());

        config.storage.persistent_path = Some(PathBuf::from("/tmp/data"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_storage_mode_from_str() {
        assert_eq!(StorageMode::from_str("memory").unwrap(), StorageMode::Memory);
        assert_eq!(StorageMode::from_str("persistent").unwrap(), StorageMode::Persistent);
        assert_eq!(StorageMode::from_str("http").unwrap(), StorageMode::Http);
        assert_eq!(StorageMode::from_str("MEMORY").unwrap(), StorageMode::Memory);
        assert!(StorageMode::from_str("invalid").is_err());
    }

    #[test]
    fn test_address_parsing() {
        let config = AppConfig::default();
        let addr = config.address().unwrap();
        assert_eq!(addr.port(), 3000);
    }

    #[test]
    fn test_config_from_toml() {
        let toml = r#"
[server]
host = "0.0.0.0"
port = 8080

[storage]
mode = "persistent"
persistent_path = "/tmp/data"

[logging]
level = "debug"
format = "json"
"#;
        let config: AppConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.storage.mode, StorageMode::Persistent);
        assert_eq!(config.logging.level, "debug");
        assert!(matches!(config.logging.format, LogFormat::Json));
    }
}
