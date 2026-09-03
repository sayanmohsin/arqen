//! Application state module for Arqen.
//!
//! Provides `AppState` with explicit wiring via builder pattern.

use std::sync::Arc;

use crate::agent::ToolRegistry;
use crate::config::{AppConfig, ConfigError};
use crate::health::HealthRegistry;
#[cfg(feature = "http-server")]
use crate::http::MiddlewareHook;
use crate::module::{Module, ModuleBuilder, ModuleError};
use crate::scheduler::Scheduler;
use crate::thingd::{StorageFactory, ThingdBackend};

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
    /// Durable scheduler backed by the configured Thingd storage.
    pub scheduler: Arc<Scheduler>,
    /// Application request hooks, in registration order.
    #[cfg(feature = "http-server")]
    pub middleware_hooks: Arc<Vec<Arc<dyn MiddlewareHook>>>,
}

impl AppState {
    /// Create a builder for `AppState`.
    pub fn builder() -> AppStateBuilder {
        AppStateBuilder::new()
    }

    /// Start the durable scheduler after application schedules are registered.
    pub async fn start_scheduler(&self) -> Result<(), crate::scheduler::SchedulerError> {
        self.scheduler.start().await
    }

    /// Stop the durable scheduler during graceful application shutdown.
    pub async fn stop_scheduler(&self) -> Result<(), crate::scheduler::SchedulerError> {
        self.scheduler.stop().await
    }

    /// Return a clone with additional request hooks appended in registration
    /// order. This keeps explicit-state applications composable.
    #[cfg(feature = "http-server")]
    pub fn with_middleware_hooks(
        &self,
        hooks: impl IntoIterator<Item = Arc<dyn MiddlewareHook>>,
    ) -> Self {
        let mut combined = self.middleware_hooks.as_ref().clone();
        combined.extend(hooks);
        Self {
            middleware_hooks: Arc::new(combined),
            ..self.clone()
        }
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
    scheduler: Option<Arc<Scheduler>>,
    #[cfg(feature = "http-server")]
    middleware_hooks: Vec<Arc<dyn MiddlewareHook>>,
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
            scheduler: None,
            #[cfg(feature = "http-server")]
            middleware_hooks: Vec::new(),
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

    /// Set an explicitly configured scheduler.
    pub fn with_scheduler(mut self, scheduler: Arc<Scheduler>) -> Self {
        self.scheduler = Some(scheduler);
        self
    }

    /// Register an application request middleware hook.
    #[cfg(feature = "http-server")]
    pub fn with_middleware_hook<H: MiddlewareHook + 'static>(mut self, hook: H) -> Self {
        self.middleware_hooks.push(Arc::new(hook));
        self
    }

    /// Register multiple request middleware hooks in deterministic order.
    #[cfg(feature = "http-server")]
    pub fn with_middleware_hooks(mut self, hooks: Vec<Arc<dyn MiddlewareHook>>) -> Self {
        self.middleware_hooks.extend(hooks);
        self
    }

    /// Register modules and auto-configure tool and health registries.
    ///
    /// This validates the module graph, registers tools and health checks
    /// from each module, and sets up the health registry. Modules are
    /// registered in dependency order.
    ///
    /// # Errors
    ///
    /// Returns `ModuleError` if the module graph has duplicates, missing
    /// dependencies, cycles, or if a module's `register()` call fails.
    pub fn with_modules<M: Module + 'static>(
        mut self,
        modules: Vec<M>,
    ) -> Result<Self, ModuleError> {
        let mut module_builder = ModuleBuilder::new();
        for module in modules {
            module_builder = module_builder.register(module);
        }
        module_builder.validate()?;

        // Create fresh registries and register module capabilities
        let mut tools = ToolRegistry::new(
            &format!("{}-app", env!("CARGO_PKG_NAME")),
            env!("CARGO_PKG_VERSION"),
            "An Arqen application",
            "memory",
        );
        let mut health = HealthRegistry::new();

        module_builder.register_all(&mut tools, &mut health)?;

        self.tool_registry = Some(Arc::new(tools));
        self.health_registry = Some(Arc::new(health));

        Ok(self)
    }

    /// Build the `AppState`.
    ///
    /// If no storage is provided, constructs the backend selected by config.
    /// If no tool registry is provided, creates a default `ToolRegistry`.
    pub fn build(self) -> Result<AppState, ConfigError> {
        let config = self.config.unwrap_or_default();

        let storage = match self.storage {
            Some(storage) => storage,
            None => StorageFactory::build(&config)?,
        };

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

        let health_registry = self.health_registry.map(|mut registry| {
            if let Some(registry) = Arc::get_mut(&mut registry) {
                registry.configure(config.health.check_timeout, config.health.startup_delay);
            }
            registry
        });

        let scheduler = self
            .scheduler
            .unwrap_or_else(|| Scheduler::new(storage.clone()));

        Ok(AppState {
            config,
            storage,
            tool_registry,
            storage_mode,
            thingd_ready,
            health_registry,
            scheduler,
            #[cfg(feature = "http-server")]
            middleware_hooks: Arc::new(self.middleware_hooks),
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
    use crate::thingd::MemoryThingdBackend;

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

    #[test]
    fn test_app_state_builder_with_modules() {
        use crate::agent::{ToolEffect, ToolMetadata};
        use crate::module::{ModuleContext, ModuleHealth};

        struct UsersModule;

        #[async_trait::async_trait]
        impl crate::module::Module for UsersModule {
            fn name(&self) -> &str {
                "users"
            }

            fn register(&self, ctx: &mut ModuleContext<'_>) -> Result<(), crate::core::AppError> {
                ctx.tools.register_tool(ToolMetadata {
                    name: "get_user".to_string(),
                    description: "Get a user by ID".to_string(),
                    input: serde_json::json!({"type": "object"}),
                    output: serde_json::json!({"type": "object"}),
                    scopes: vec!["read:users".to_string()],
                    effect: ToolEffect::Read,
                    idempotent: true,
                    enqueues_job: None,
                    timeout: None,
                });
                Ok(())
            }

            async fn health_check(&self) -> ModuleHealth {
                ModuleHealth::Healthy
            }
        }

        let state = AppState::builder()
            .with_modules(vec![UsersModule])
            .unwrap()
            .build()
            .unwrap();

        // Tool was registered
        assert!(state.tool_registry.get_tool("get_user").is_some());

        // Health registry was created with module's health check
        let health = state.health_registry.unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let report = rt.block_on(health.check_liveness());
        assert_eq!(report.checks.len(), 1);
        assert_eq!(report.checks[0].name, "users");
    }

    #[test]
    fn test_app_state_builder_with_modules_validation_error() {
        struct DepModule;

        #[async_trait::async_trait]
        impl crate::module::Module for DepModule {
            fn name(&self) -> &str {
                "app"
            }
            fn dependencies(&self) -> Vec<&str> {
                vec!["nonexistent"]
            }
        }

        let result = AppState::builder().with_modules(vec![DepModule]);
        assert!(result.is_err());
    }
}
