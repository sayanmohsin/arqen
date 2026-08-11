//! Configuration module for Arqen.
//!
//! Provides typed configuration with layered loading (CLI flags → env vars → file → defaults),
//! validation, and secret redaction.
//!
//! # Configuration Layers
//!
//! Configuration is loaded in order of precedence (highest wins):
//! 1. CLI flags (passed to `from_cli_overrides`)
//! 2. Environment variables (`ARQEN_*`)
//! 3. Configuration file (`arqen.toml`)
//! 4. Defaults
//!
//! # Environment Variables
//!
//! All env vars use the `ARQEN_` prefix:
//! - `ARQEN_HOST` - Server host (default: 127.0.0.1)
//! - `ARQEN_PORT` - Server port (default: 8888)
//! - `ARQEN_STORAGE_MODE` - Storage mode: memory, native, persistent, http, cloud (default: memory)
//! - `ARQEN_PERSISTENT_PATH` - Path for persistent storage
//! - `ARQEN_THINGD_URL` - Thingd HTTP URL
//! - `ARQEN_THINGD_AUTH_TOKEN` - Thingd HTTP bearer token (redacted)
//! - `ARQEN_CLOUD_URL` - Future public thingd.cloud URL
//! - `ARQEN_JWT_SECRET` - JWT secret for authentication
//! - `ARQEN_LOG_LEVEL` - Log level (default: info)
//! - `ARQEN_LOG_FORMAT` - Log format: pretty, json, compact (default: pretty)

use std::fmt;
use std::path::PathBuf;

use clap::ValueEnum;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::core::{AppError, ErrorKind};

/// A wrapper that redacts sensitive values in Display/Debug output.
#[derive(Serialize, Deserialize)]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }

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

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_request_timeout")]
    pub request_timeout: Duration,
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout: Duration,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}
fn default_port() -> u16 {
    8888
}
fn default_request_timeout() -> Duration {
    Duration::from_secs(30)
}
fn default_max_body_size() -> usize {
    1024 * 1024
}
fn default_shutdown_timeout() -> Duration {
    Duration::from_secs(10)
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            request_timeout: default_request_timeout(),
            max_body_size: default_max_body_size(),
            shutdown_timeout: default_shutdown_timeout(),
        }
    }
}

/// Storage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default)]
    pub mode: StorageMode,
    pub persistent_path: Option<PathBuf>,
    pub http_url: Option<String>,
    /// Optional hosted thingd/cloud endpoint.
    pub cloud_url: Option<String>,
    /// Server-side credential for remote thingd/cloud storage.
    pub auth_token: Option<Secret<String>>,
    /// Optional 64-character hexadecimal Thingd persistent encryption key.
    pub encryption_key: Option<Secret<String>>,
    /// Optional `.thingd` schema document used for startup validation.
    pub schema_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StorageMode {
    #[default]
    Memory,
    /// Embedded native thingd. `persistent` remains accepted as a legacy name.
    Native,
    Persistent,
    Http,
    Cloud,
}

impl StorageMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Native => "native",
            Self::Persistent => "persistent",
            Self::Http => "http",
            Self::Cloud => "cloud",
        }
    }
}

/// Replication transport supported by Arqen.
///
/// `Native` uses Thingd's public native replication service over embedded
/// storage and normally sends changes to an HTTP Thingd target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThingdSyncMode {
    /// Replication is disabled.
    #[default]
    Disabled,
    /// Replication uses Thingd's public HTTP API.
    Http,
    /// Replication uses a future public native Thingd API.
    Native,
}

impl ThingdSyncMode {
    /// Parse a configured sync mode.
    pub fn parse_str(value: &str) -> Result<Self, ConfigError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "disabled" | "off" | "none" => Ok(Self::Disabled),
            "http" => Ok(Self::Http),
            "native" => Ok(Self::Native),
            _ => Err(ConfigError::InvalidValue {
                field: "sync.mode".to_string(),
                value: value.to_string(),
                expected: "disabled, http, or native".to_string(),
            }),
        }
    }
}

impl StorageMode {
    pub fn parse_str(s: &str) -> Result<Self, ConfigError> {
        match s.to_lowercase().as_str() {
            "memory" => Ok(Self::Memory),
            "native" => Ok(Self::Native),
            "persistent" => Ok(Self::Persistent),
            "http" => Ok(Self::Http),
            "cloud" => Ok(Self::Cloud),
            _ => Err(ConfigError::InvalidValue {
                field: "storage_mode".to_string(),
                value: s.to_string(),
                expected: "memory, native, persistent, http, or cloud".to_string(),
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
            cloud_url: None,
            auth_token: None,
            encryption_key: None,
            schema_path: None,
        }
    }
}

/// Opt-in local-to-remote Thingd replication settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub mode: ThingdSyncMode,
    pub source_id: Option<String>,
    pub target_url: Option<String>,
    pub target_auth_token: Option<Secret<String>>,
    #[serde(default)]
    pub collections: Vec<String>,
    /// Explicitly opt into replicating every supported application collection.
    #[serde(default)]
    pub replicate_all: bool,
    #[serde(default = "default_sync_poll_interval")]
    pub poll_interval: Duration,
    #[serde(default = "default_sync_batch_size")]
    pub batch_size: u32,
    #[serde(default = "default_sync_snapshot_fallback")]
    pub snapshot_fallback: bool,
}

fn default_sync_poll_interval() -> Duration {
    Duration::from_secs(5)
}
fn default_sync_batch_size() -> u32 {
    500
}
fn default_sync_snapshot_fallback() -> bool {
    true
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: ThingdSyncMode::Disabled,
            source_id: None,
            target_url: None,
            target_auth_token: None,
            collections: Vec::new(),
            replicate_all: false,
            poll_interval: default_sync_poll_interval(),
            batch_size: default_sync_batch_size(),
            snapshot_fallback: default_sync_snapshot_fallback(),
        }
    }
}

/// Authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(default)]
    pub enabled: bool,
    pub jwt_secret: Option<Secret<String>>,
    #[serde(default = "default_api_key_header")]
    pub api_key_header: String,
    #[serde(default)]
    pub api_keys: Vec<ApiKeyEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyEntry {
    pub key: Secret<String>,
    pub subject: String,
    #[serde(default)]
    pub scopes: Vec<String>,
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
            api_keys: Vec::new(),
        }
    }
}

/// Worker configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_worker_queues")]
    pub queues: Vec<String>,
    #[serde(default = "default_poll_interval")]
    pub poll_interval: Duration,
    #[serde(default = "default_lease_seconds")]
    pub lease_seconds: u32,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_worker_concurrency")]
    pub concurrency: u32,
}

fn default_worker_queues() -> Vec<String> {
    vec!["default".to_string()]
}
fn default_poll_interval() -> Duration {
    Duration::from_secs(1)
}
fn default_lease_seconds() -> u32 {
    30
}
fn default_max_retries() -> u32 {
    3
}
fn default_worker_concurrency() -> u32 {
    4
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            queues: default_worker_queues(),
            poll_interval: default_poll_interval(),
            lease_seconds: default_lease_seconds(),
            max_retries: default_max_retries(),
            concurrency: default_worker_concurrency(),
        }
    }
}

/// Health check configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    #[serde(default = "default_health_timeout")]
    pub check_timeout: Duration,
    #[serde(default = "default_startup_delay")]
    pub startup_delay: Duration,
}

fn default_health_timeout() -> Duration {
    Duration::from_secs(5)
}
fn default_startup_delay() -> Duration {
    Duration::ZERO
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_timeout: default_health_timeout(),
            startup_delay: default_startup_delay(),
        }
    }
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub format: LogFormat,
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Pretty,
    Json,
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub worker: WorkerConfig,
    #[serde(default)]
    pub health: HealthConfig,
    #[serde(default)]
    pub sync: SyncConfig,
}

/// CLI overrides for configuration (highest precedence).
#[derive(Default)]
pub struct CliOverrides {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub log_level: Option<String>,
    pub log_format: Option<LogFormat>,
    pub storage_mode: Option<String>,
}

impl AppConfig {
    /// Load configuration with full precedence chain: CLI → env → file → defaults.
    pub fn load(cli: CliOverrides) -> Result<Self, ConfigError> {
        // Layer 1: Defaults
        let mut config = Self::default();

        // Layer 2: File (arqen.toml in current directory)
        let file_config = Self::from_file_optional("arqen.toml")?;
        if let Some(file) = file_config {
            config = file;
        }

        // Layer 3: Environment variables
        config = config.apply_env()?;

        // Layer 4: CLI overrides
        config = config.apply_cli(cli);

        config.validate()?;
        Ok(config)
    }

    /// Load from file if it exists, otherwise return None.
    fn from_file_optional(path: &str) -> Result<Option<Self>, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let config: Self = toml::from_str(&content).map_err(ConfigError::ParseError)?;
                Ok(Some(config))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(ConfigError::FileError {
                path: PathBuf::from(path),
                source: e,
            }),
        }
    }

    /// Apply environment variable overrides.
    fn apply_env(mut self) -> Result<Self, ConfigError> {
        if let Ok(host) = std::env::var("ARQEN_HOST") {
            self.server.host = host;
        }
        if let Ok(port) = std::env::var("ARQEN_PORT") {
            self.server.port = port.parse().map_err(|_| ConfigError::InvalidValue {
                field: "port".to_string(),
                value: port,
                expected: "a valid u16".to_string(),
            })?;
        }
        if let Ok(mode) = std::env::var("ARQEN_STORAGE_MODE") {
            self.storage.mode = StorageMode::parse_str(&mode)?;
        }
        if let Ok(path) = std::env::var("ARQEN_PERSISTENT_PATH") {
            self.storage.persistent_path = Some(PathBuf::from(path));
        }
        if let Ok(url) = std::env::var("ARQEN_THINGD_URL") {
            self.storage.http_url = Some(url);
        }
        if let Ok(url) = std::env::var("ARQEN_CLOUD_URL") {
            self.storage.cloud_url = Some(url);
        }
        if let Ok(token) = std::env::var("ARQEN_THINGD_AUTH_TOKEN") {
            self.storage.auth_token = Some(Secret::new(token));
        }
        if let Ok(key) = std::env::var("ARQEN_THINGD_ENCRYPTION_KEY") {
            self.storage.encryption_key = Some(Secret::new(key));
        }
        if let Ok(path) = std::env::var("ARQEN_THINGD_SCHEMA_PATH") {
            self.storage.schema_path = Some(PathBuf::from(path));
        }
        if let Ok(enabled) = std::env::var("ARQEN_SYNC_ENABLED") {
            self.sync.enabled = enabled == "true" || enabled == "1";
        }
        if let Ok(mode) = std::env::var("ARQEN_SYNC_MODE") {
            self.sync.mode = ThingdSyncMode::parse_str(&mode)?;
        } else if self.sync.enabled && self.sync.mode == ThingdSyncMode::Disabled {
            // Preserve compatibility with the original enabled-only config
            // while making the effective transport explicit internally.
            self.sync.mode = ThingdSyncMode::Http;
        }
        if let Ok(source_id) = std::env::var("ARQEN_SYNC_SOURCE_ID") {
            self.sync.source_id = Some(source_id);
        }
        if let Ok(url) = std::env::var("ARQEN_SYNC_TARGET_URL") {
            self.sync.target_url = Some(url);
        }
        if let Ok(token) = std::env::var("ARQEN_SYNC_TARGET_AUTH_TOKEN") {
            self.sync.target_auth_token = Some(Secret::new(token));
        }
        if let Ok(collections) = std::env::var("ARQEN_SYNC_COLLECTIONS") {
            self.sync.collections = collections
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        if let Ok(replicate_all) = std::env::var("ARQEN_SYNC_REPLICATE_ALL") {
            self.sync.replicate_all = replicate_all == "true" || replicate_all == "1";
        }
        if let Ok(interval) = std::env::var("ARQEN_SYNC_POLL_INTERVAL") {
            self.sync.poll_interval =
                Duration::from_secs(interval.parse().map_err(|_| ConfigError::InvalidValue {
                    field: "sync.poll_interval".to_string(),
                    value: interval,
                    expected: "a valid u64 (seconds)".to_string(),
                })?);
        }
        if let Ok(batch_size) = std::env::var("ARQEN_SYNC_BATCH_SIZE") {
            self.sync.batch_size = batch_size.parse().map_err(|_| ConfigError::InvalidValue {
                field: "sync.batch_size".to_string(),
                value: batch_size,
                expected: "a valid u32".to_string(),
            })?;
        }
        if let Ok(fallback) = std::env::var("ARQEN_SYNC_SNAPSHOT_FALLBACK") {
            self.sync.snapshot_fallback = fallback == "true" || fallback == "1";
        }
        if let Ok(secret) = std::env::var("ARQEN_JWT_SECRET") {
            self.auth.enabled = true;
            self.auth.jwt_secret = Some(Secret::new(secret));
        }
        if let Ok(header) = std::env::var("ARQEN_API_KEY_HEADER") {
            self.auth.api_key_header = header;
        }
        if let Ok(level) = std::env::var("ARQEN_LOG_LEVEL") {
            self.logging.level = level;
        }
        if let Ok(format) = std::env::var("ARQEN_LOG_FORMAT") {
            self.logging.format = match format.to_lowercase().as_str() {
                "pretty" => LogFormat::Pretty,
                "json" => LogFormat::Json,
                "compact" => LogFormat::Compact,
                _ => {
                    return Err(ConfigError::InvalidValue {
                        field: "log_format".to_string(),
                        value: format,
                        expected: "pretty, json, or compact".to_string(),
                    });
                }
            };
        }
        if let Ok(enabled) = std::env::var("ARQEN_WORKER_ENABLED") {
            self.worker.enabled = enabled == "true" || enabled == "1";
        }
        if let Ok(queues) = std::env::var("ARQEN_WORKER_QUEUES") {
            self.worker.queues = queues.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Ok(interval) = std::env::var("ARQEN_WORKER_POLL_INTERVAL") {
            self.worker.poll_interval =
                Duration::from_secs(interval.parse().map_err(|_| ConfigError::InvalidValue {
                    field: "worker_poll_interval".to_string(),
                    value: interval,
                    expected: "a valid u64 (seconds)".to_string(),
                })?);
        }
        if let Ok(lease) = std::env::var("ARQEN_WORKER_LEASE_SECONDS") {
            self.worker.lease_seconds = lease.parse().map_err(|_| ConfigError::InvalidValue {
                field: "worker_lease_seconds".to_string(),
                value: lease,
                expected: "a valid u32".to_string(),
            })?;
        }
        if let Ok(retries) = std::env::var("ARQEN_WORKER_MAX_RETRIES") {
            self.worker.max_retries = retries.parse().map_err(|_| ConfigError::InvalidValue {
                field: "worker_max_retries".to_string(),
                value: retries,
                expected: "a valid u32".to_string(),
            })?;
        }
        if let Ok(concurrency) = std::env::var("ARQEN_WORKER_CONCURRENCY") {
            self.worker.concurrency =
                concurrency.parse().map_err(|_| ConfigError::InvalidValue {
                    field: "worker_concurrency".to_string(),
                    value: concurrency,
                    expected: "a valid u32".to_string(),
                })?;
        }
        if let Ok(timeout) = std::env::var("ARQEN_HEALTH_CHECK_TIMEOUT") {
            self.health.check_timeout =
                Duration::from_secs(timeout.parse().map_err(|_| ConfigError::InvalidValue {
                    field: "health_check_timeout".to_string(),
                    value: timeout,
                    expected: "a valid u64 (seconds)".to_string(),
                })?);
        }
        if let Ok(delay) = std::env::var("ARQEN_HEALTH_STARTUP_DELAY") {
            self.health.startup_delay =
                Duration::from_secs(delay.parse().map_err(|_| ConfigError::InvalidValue {
                    field: "health_startup_delay".to_string(),
                    value: delay,
                    expected: "a valid u64 (seconds)".to_string(),
                })?);
        }
        if let Ok(timeout) = std::env::var("ARQEN_REQUEST_TIMEOUT") {
            self.server.request_timeout =
                Duration::from_secs(timeout.parse().map_err(|_| ConfigError::InvalidValue {
                    field: "request_timeout".to_string(),
                    value: timeout,
                    expected: "a valid u64 (seconds)".to_string(),
                })?);
        }
        if let Ok(size) = std::env::var("ARQEN_MAX_BODY_SIZE") {
            self.server.max_body_size = size.parse().map_err(|_| ConfigError::InvalidValue {
                field: "max_body_size".to_string(),
                value: size,
                expected: "a valid usize (bytes)".to_string(),
            })?;
        }
        if let Ok(timeout) = std::env::var("ARQEN_SHUTDOWN_TIMEOUT") {
            self.server.shutdown_timeout =
                Duration::from_secs(timeout.parse().map_err(|_| ConfigError::InvalidValue {
                    field: "shutdown_timeout".to_string(),
                    value: timeout,
                    expected: "a valid u64 (seconds)".to_string(),
                })?);
        }
        Ok(self)
    }

    /// Apply CLI overrides (highest precedence).
    fn apply_cli(mut self, cli: CliOverrides) -> Self {
        if let Some(host) = cli.host {
            self.server.host = host;
        }
        if let Some(port) = cli.port {
            self.server.port = port;
        }
        if let Some(level) = cli.log_level {
            self.logging.level = level;
        }
        if let Some(format) = cli.log_format {
            self.logging.format = format;
        }
        if let Some(mode) = cli.storage_mode
            && let Ok(m) = StorageMode::parse_str(&mode)
        {
            self.storage.mode = m;
        }
        self
    }

    /// Load configuration with full precedence chain, allowing a custom config file path.
    pub fn load_with_file(
        cli: CliOverrides,
        path: Option<&std::path::Path>,
    ) -> Result<Self, ConfigError> {
        let mut config = Self::default();

        let file_path = path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("arqen.toml"));

        if file_path.exists() {
            let file_config = Self::from_file_optional(file_path.to_str().unwrap_or("arqen.toml"))?;
            if let Some(file) = file_config {
                config = file;
            }
        }

        config = config.apply_env()?;
        config = config.apply_cli(cli);
        config.validate()?;
        Ok(config)
    }

    /// Load configuration from a TOML file.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, ConfigError> {
        let content =
            std::fs::read_to_string(path.as_ref()).map_err(|e| ConfigError::FileError {
                path: path.as_ref().to_path_buf(),
                source: e,
            })?;
        let config: Self = toml::from_str(&content).map_err(ConfigError::ParseError)?;
        config.validate()?;
        Ok(config)
    }

    /// Load configuration from environment variables (legacy API).
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::load(CliOverrides::default())
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
        if self.storage.mode == StorageMode::Native && self.storage.persistent_path.is_none() {
            return Err(ConfigError::MissingField {
                field: "persistent_path".to_string(),
                context: "required when storage mode is native".to_string(),
            });
        }
        if self.storage.mode == StorageMode::Http && self.storage.http_url.is_none() {
            return Err(ConfigError::MissingField {
                field: "http_url".to_string(),
                context: "required when storage mode is http".to_string(),
            });
        }
        if self.storage.mode == StorageMode::Cloud && self.storage.cloud_url.is_none() {
            return Err(ConfigError::MissingField {
                field: "cloud_url".to_string(),
                context: "required when storage mode is cloud".to_string(),
            });
        }
        if self.sync.enabled {
            if self.sync.mode == ThingdSyncMode::Disabled {
                return Err(ConfigError::InvalidValue {
                    field: "sync.mode".to_string(),
                    value: "disabled".to_string(),
                    expected: "http or native when sync is enabled".to_string(),
                });
            }
            if self.sync.mode == ThingdSyncMode::Native && self.storage.mode != StorageMode::Native
            {
                return Err(ConfigError::InvalidValue {
                    field: "storage.mode".to_string(),
                    value: format!("{:?}", self.storage.mode),
                    expected: "native when sync.mode is native".to_string(),
                });
            }
            if self.sync.target_url.is_none() {
                return Err(ConfigError::MissingField {
                    field: "sync.target_url".to_string(),
                    context: "required when sync is enabled".to_string(),
                });
            }
            if self.sync.source_id.as_deref().is_none_or(str::is_empty) {
                return Err(ConfigError::MissingField {
                    field: "sync.source_id".to_string(),
                    context: "required when sync is enabled".to_string(),
                });
            }
            if self.sync.collections.is_empty() && !self.sync.replicate_all {
                return Err(ConfigError::InvalidValue {
                    field: "sync.collections".to_string(),
                    value: "empty".to_string(),
                    expected: "a non-empty allowlist or sync.replicate_all=true".to_string(),
                });
            }
            if self.sync.batch_size == 0 || self.sync.batch_size > 1000 {
                return Err(ConfigError::InvalidValue {
                    field: "sync.batch_size".to_string(),
                    value: self.sync.batch_size.to_string(),
                    expected: "between 1 and 1000".to_string(),
                });
            }
            if self.sync.poll_interval.is_zero() {
                return Err(ConfigError::InvalidValue {
                    field: "sync.poll_interval".to_string(),
                    value: "0".to_string(),
                    expected: "greater than zero".to_string(),
                });
            }
        }
        if self.worker.lease_seconds == 0 {
            return Err(ConfigError::InvalidValue {
                field: "worker.lease_seconds".to_string(),
                value: "0".to_string(),
                expected: "greater than zero".to_string(),
            });
        }
        if self.worker.max_retries > 1_000_000 {
            return Err(ConfigError::InvalidValue {
                field: "worker.max_retries".to_string(),
                value: self.worker.max_retries.to_string(),
                expected: "a bounded retry count".to_string(),
            });
        }
        Ok(())
    }

    /// Apply guardrails intended for a production deployment.
    ///
    /// This is explicit so libraries can still use the permissive development
    /// defaults without accidentally changing their startup behavior.
    pub fn validate_production(&self) -> Result<(), ConfigError> {
        self.validate_production_with_schema_validation(true)
    }

    /// Apply production guardrails, optionally allowing an application to
    /// defer native schema validation to its own startup workflow.
    pub fn validate_production_with_schema_validation(
        &self,
        require_schema_validation: bool,
    ) -> Result<(), ConfigError> {
        self.validate()?;
        if self.storage.mode == StorageMode::Memory {
            return Err(ConfigError::InvalidValue {
                field: "storage.mode".to_string(),
                value: "memory".to_string(),
                expected: "native, http, or cloud in production".to_string(),
            });
        }
        if !self.auth.enabled {
            return Err(ConfigError::InvalidValue {
                field: "auth.enabled".to_string(),
                value: "false".to_string(),
                expected: "true in production".to_string(),
            });
        }
        if matches!(self.logging.format, LogFormat::Pretty) {
            return Err(ConfigError::InvalidValue {
                field: "logging.format".to_string(),
                value: "pretty".to_string(),
                expected: "json or compact in production".to_string(),
            });
        }
        if require_schema_validation
            && !matches!(self.storage.mode, StorageMode::Http | StorageMode::Cloud)
            && self.storage.schema_path.is_none()
        {
            return Err(ConfigError::MissingField {
                field: "storage.schema_path".to_string(),
                context: "required for native storage; HTTP mode uses the remote thingd schema"
                    .to_string(),
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

/// Configuration error type.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("invalid value for {field}: '{value}' (expected {expected})")]
    InvalidValue {
        field: String,
        value: String,
        expected: String,
    },

    #[error("missing required field '{field}': {context}")]
    MissingField { field: String, context: String },

    #[error("failed to read config file {}: {source}", path.display())]
    FileError {
        path: PathBuf,
        source: std::io::Error,
    },

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
        assert_eq!(config.server.port, 8888);
        assert_eq!(config.storage.mode, StorageMode::Memory);
        assert!(!config.auth.enabled);
        assert_eq!(config.logging.level, "info");
        assert!(!config.worker.enabled);
        assert_eq!(config.worker.queues, vec!["default".to_string()]);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        assert!(config.validate().is_ok());

        config.server.port = 0;
        assert!(config.validate().is_err());

        config.server.port = 8888;
        config.storage.mode = StorageMode::Persistent;
        assert!(config.validate().is_err());

        config.storage.persistent_path = Some(PathBuf::from("/tmp/data"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_storage_mode_from_str() {
        assert_eq!(
            StorageMode::parse_str("memory").unwrap(),
            StorageMode::Memory
        );
        assert_eq!(
            StorageMode::parse_str("persistent").unwrap(),
            StorageMode::Persistent
        );
        assert_eq!(StorageMode::parse_str("http").unwrap(), StorageMode::Http);
        assert_eq!(
            StorageMode::parse_str("MEMORY").unwrap(),
            StorageMode::Memory
        );
        assert!(StorageMode::parse_str("invalid").is_err());
    }

    #[test]
    fn test_address_parsing() {
        let config = AppConfig::default();
        let addr = config.address().unwrap();
        assert_eq!(addr.port(), 8888);
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

[worker]
enabled = true
queues = ["jobs", "events"]
concurrency = 8
"#;
        let config: AppConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.storage.mode, StorageMode::Persistent);
        assert_eq!(config.logging.level, "debug");
        assert!(matches!(config.logging.format, LogFormat::Json));
        assert!(config.worker.enabled);
        assert_eq!(config.worker.queues, vec!["jobs", "events"]);
        assert_eq!(config.worker.concurrency, 8);
    }

    #[test]
    fn test_cli_overrides() {
        let config = AppConfig::default();
        let cli = CliOverrides {
            host: Some("0.0.0.0".to_string()),
            port: Some(8080),
            log_level: Some("debug".to_string()),
            log_format: None,
            storage_mode: None,
        };
        let config = config.apply_cli(cli);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    fn test_worker_config_defaults() {
        let config = WorkerConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.queues, vec!["default".to_string()]);
        assert_eq!(config.poll_interval, Duration::from_secs(1));
        assert_eq!(config.lease_seconds, 30);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.concurrency, 4);
    }

    #[test]
    fn test_health_config_defaults() {
        let config = HealthConfig::default();
        assert_eq!(config.check_timeout, Duration::from_secs(5));
        assert_eq!(config.startup_delay, Duration::ZERO);
    }

    #[test]
    fn test_invalid_port() {
        let config = AppConfig {
            server: ServerConfig {
                port: 0,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_persistent_requires_path() {
        let config = AppConfig {
            storage: StorageConfig {
                mode: StorageMode::Persistent,
                persistent_path: None,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_http_requires_url() {
        let config = AppConfig {
            storage: StorageConfig {
                mode: StorageMode::Http,
                http_url: None,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn sync_mode_defaults_to_disabled() {
        let config = AppConfig::default();
        assert_eq!(config.sync.mode, ThingdSyncMode::Disabled);
        assert!(!config.sync.enabled);
    }

    #[test]
    fn native_sync_requires_embedded_native_storage() {
        let mut config = AppConfig::default();
        config.sync.enabled = true;
        config.sync.mode = ThingdSyncMode::Native;
        config.sync.source_id = Some("local".to_string());
        config.sync.target_url = Some("https://cloud.example".to_string());
        let error = config.validate().unwrap_err();
        assert!(error.to_string().contains("storage.mode"));

        config.storage.mode = StorageMode::Native;
        config.storage.persistent_path = Some("/tmp/thingd".into());
        config.sync.collections = vec!["notes".into()];
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_address_invalid() {
        let config = AppConfig {
            server: ServerConfig {
                host: "not-a-host".to_string(),
                port: 8888,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(config.address().is_err());
    }
}
