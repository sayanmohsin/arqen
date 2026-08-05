# Interfaces

## AppConfig

```rust
pub struct AppConfig {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub auth: AuthConfig,
    pub logging: LoggingConfig,
}

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub request_timeout: Duration,
    pub max_body_size: usize,
}

pub struct StorageConfig {
    pub mode: StorageMode,
    pub persistent_path: Option<PathBuf>,
    pub http_url: Option<String>,
}

pub enum StorageMode {
    Memory,
    Persistent,
    Http,
}

pub struct AuthConfig {
    pub enabled: bool,
    pub jwt_secret: Option<Secret<String>>,
    pub api_key_header: String,
}

pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
}

pub enum LogFormat {
    Pretty,
    Json,
    Compact,
}
```

## `Secret<T>`

```rust
pub struct Secret<T>(T);

impl<T: Display> Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl<T: Debug> Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}
```

## AppState

```rust
pub struct AppState {
    pub config: AppConfig,
    pub storage: Arc<dyn ThingdBackend>,
    pub tool_registry: Arc<ToolRegistry>,
}

impl AppState {
    pub fn builder() -> AppStateBuilder { ... }
}

pub struct AppStateBuilder {
    config: AppConfig,
    storage: Option<Arc<dyn ThingdBackend>>,
    tool_registry: Option<Arc<ToolRegistry>>,
}

impl AppStateBuilder {
    pub fn new(config: AppConfig) -> Self { ... }
    pub fn with_storage(mut self, storage: Arc<dyn ThingdBackend>) -> Self { ... }
    pub fn with_tool_registry(mut self, registry: Arc<ToolRegistry>) -> Self { ... }
    pub fn build(self) -> Result<AppState, ConfigError> { ... }
}
```
