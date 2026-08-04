//! Application state module for Arqen.
//!
//! Provides `AppState` with explicit wiring via builder pattern.

use std::sync::Arc;

use crate::agent::ToolRegistry;
use crate::config::{AppConfig, ConfigError};
use crate::health::HealthRegistry;
use crate::thingd::{MemoryThingdBackend, ThingdBackend};

/// Application state shared across all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Application configuration.
    pub config: AppConfig,
    /// Storage backend.
    pub storage: Arc<dyn ThingdBackend>,
    /// Tool registry.
    pub tool_registry: Arc<ToolRegistry>,
    /// Storage mode (memory, persistent, http).
    pub storage_mode: String,
    /// Whether thingd is ready.
    pub thingd_ready: bool,
    /// Health registry for dependency checks (optional).
    pub health_registry: Option<Arc<HealthRegistry>>,
}

impl AppState {
    /// Create a builder for `AppState`.
    pub fn builder() -> AppStateBuilder {
        AppStateBuilder::new()
    }
}

/// Builder for `AppState`.
pub struct AppStateBuilder {
    config: Option<AppConfig>,
    storage: Option<Arc<dyn ThingdBackend>>,
    tool_registry: Option<Arc<ToolRegistry>>,
    storage_mode: Option<String>,
    thingd_ready: Option<bool>,
    health_registry: Option<Arc<HealthRegistry>>,
}

impl AppStateBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            config: None,
            storage: None,
            tool_registry: None,
            storage_mode: None,
            thingd_ready: None,
            health_registry: None,
        }
    }

    /// Set the application configuration.
    pub fn with_config(mut self, config: AppConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the storage backend.
    pub fn with_storage(mut self, storage: Arc<dyn ThingdBackend>) -> Self {
        self.storage = Some(storage);
        self
    }

    /// Set the tool registry.
    pub fn with_tool_registry(mut self, registry: ToolRegistry) -> Self {
        self.tool_registry = Some(Arc::new(registry));
        self
    }

    /// Set the storage mode.
    pub fn with_storage_mode(mut self, mode: impl Into<String>) -> Self {
        self.storage_mode = Some(mode.into());
        self
    }

    /// Set whether thingd is ready.
    pub fn with_thingd_ready(mut self, ready: bool) -> Self {
        self.thingd_ready = Some(ready);
        self
    }

    /// Set the health registry for dependency checks.
    pub fn with_health_registry(mut self, registry: HealthRegistry) -> Self {
        self.health_registry = Some(Arc::new(registry));
        self
    }

    /// Set the health registry (pre-wrapped in Arc).
    pub fn with_health_registry_arc(mut self, registry: Arc<HealthRegistry>) -> Self {
        self.health_registry = Some(registry);
        self
    }

    /// Build the `AppState`.
    ///
    /// If no storage is provided, creates a `MemoryThingdBackend`.
    /// If no tool registry is provided, creates a default `ToolRegistry`.
    pub fn build(self) -> Result<AppState, ConfigError> {
        let config = self.config.unwrap_or_default();

        let storage = self
            .storage
            .unwrap_or_else(|| Arc::new(MemoryThingdBackend::new()));

        let tool_registry = self.tool_registry.unwrap_or_else(|| {
            Arc::new(ToolRegistry::new(
                &format!("{}-app", env!("CARGO_PKG_NAME")),
                env!("CARGO_PKG_VERSION"),
                "An Arqen application",
                &format!("{:?}", config.storage.mode),
            ))
        });

        let storage_mode = self
            .storage_mode
            .unwrap_or_else(|| format!("{:?}", config.storage.mode).to_lowercase());

        let thingd_ready = self.thingd_ready.unwrap_or(true);

        let health_registry = self.health_registry;

        Ok(AppState {
            config,
            storage,
            tool_registry,
            storage_mode,
            thingd_ready,
            health_registry,
        })
    }
}

impl Default for AppStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StorageMode;

    #[test]
    fn test_app_state_builder_defaults() {
        let state = AppState::builder().build().unwrap();
        assert_eq!(state.config.server.port, 8888);
        assert_eq!(state.config.storage.mode, StorageMode::Memory);
    }

    #[test]
    fn test_app_state_builder_with_config() {
        let config = AppConfig {
            server: crate::config::ServerConfig {
                port: 8080,
                ..Default::default()
            },
            ..Default::default()
        };

        let state = AppState::builder().with_config(config).build().unwrap();

        assert_eq!(state.config.server.port, 8080);
    }

    #[tokio::test]
    async fn test_app_state_builder_with_storage() {
        let storage = Arc::new(MemoryThingdBackend::new());
        let state = AppState::builder().with_storage(storage).build().unwrap();

        assert!(state.storage.count_objects("test").await.is_ok());
    }

    #[tokio::test]
    async fn test_app_state_builder_with_registry() {
        let registry = ToolRegistry::new("test-app", "1.0.0", "Test", "memory");
        let state = AppState::builder()
            .with_tool_registry(registry)
            .build()
            .unwrap();

        assert_eq!(state.tool_registry.generate_manifest().name, "test-app");
    }
}
