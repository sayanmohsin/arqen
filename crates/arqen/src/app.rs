//! Convenience application wrapper for Arqen.
//!
//! Provides [`ArqenApp`] as a thin wrapper over [`AppState`], [`Router`],
//! and module composition. This is an optional convenience layer -- the
//! explicit APIs remain first-class.

use std::net::SocketAddr;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::http::{create_router_with_state, start_server};
use crate::module::{Module, ModuleBuilder, ModuleGraphError};
use crate::state::AppState;

/// A convenience wrapper for building and running Arqen applications.
///
/// `ArqenApp` ties together `AppState`, module composition, and the HTTP
/// server. For more control, use `AppState::builder()` and
/// `create_router_with_state()` directly.
///
/// # Example
///
/// ```rust,ignore
/// use arqen::app::ArqenApp;
/// use arqen::module::Module;
///
/// struct UsersModule;
///
/// #[async_trait::async_trait]
/// impl Module for UsersModule {
///     fn name(&self) -> &str { "users" }
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     ArqenApp::builder()
///         .name("my-api")
///         .module(UsersModule)
///         .build()?
///         .start()
///         .await
/// }
/// ```
pub struct ArqenApp {
    state: AppState,
}

impl ArqenApp {
    /// Create a builder for `ArqenApp`.
    pub fn builder() -> ArqenAppBuilder {
        ArqenAppBuilder::new()
    }

    /// Get a reference to the app state.
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Start the server and run until shutdown signal.
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr: SocketAddr = format!(
            "{}:{}",
            self.state.config.server.host, self.state.config.server.port
        )
        .parse()?;

        let router = create_router_with_state(self.state);

        tracing::info!("Starting Arqen app on {}", addr);
        start_server(addr, router).await
    }
}

/// Builder for [`ArqenApp`].
pub struct ArqenAppBuilder {
    name: Option<String>,
    config: Option<AppConfig>,
    state: Option<AppState>,
    modules: Vec<Arc<dyn Module>>,
}

impl ArqenAppBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            name: None,
            config: None,
            state: None,
            modules: Vec::new(),
        }
    }

    /// Set the application name (used in manifest and logs).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the application configuration.
    pub fn config(mut self, config: AppConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set an explicit AppState (escape hatch).
    ///
    /// When set, `config()` and `module()` are ignored.
    pub fn state(mut self, state: AppState) -> Self {
        self.state = Some(state);
        self
    }

    /// Register a module with the application.
    pub fn module<M: Module + 'static>(mut self, m: M) -> Self {
        self.modules.push(Arc::new(m));
        self
    }

    /// Build the `ArqenApp`.
    ///
    /// # Errors
    ///
    /// Returns `ModuleGraphError` if the module graph is invalid.
    pub fn build(self) -> Result<ArqenApp, ModuleGraphError> {
        if let Some(state) = self.state {
            return Ok(ArqenApp { state });
        }

        let mut builder = AppState::builder();

        if let Some(config) = self.config {
            builder = builder.with_config(config);
        }

        // Validate module graph if modules are present
        if !self.modules.is_empty() {
            let mut module_builder = ModuleBuilder::new();
            for module in &self.modules {
                module_builder = module_builder.register_arc(module.clone());
            }
            module_builder.validate()?;

            // Register tools and health from modules
            let mut tools = crate::agent::ToolRegistry::new(
                self.name.as_deref().unwrap_or("arqen-app"),
                env!("CARGO_PKG_VERSION"),
                "An Arqen application",
                "memory",
            );
            let mut health = crate::health::HealthRegistry::new();
            module_builder
                .register_all(&mut tools, &mut health)
                .ok();

            builder = builder
                .with_tool_registry(tools)
                .with_health_registry(health);
        }

        let state = builder
            .build()
            .map_err(|e| ModuleGraphError::DuplicateModule(e.to_string()))?;

        Ok(ArqenApp { state })
    }
}

impl Default for ArqenAppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestModule;

    #[async_trait::async_trait]
    impl Module for TestModule {
        fn name(&self) -> &str {
            "test"
        }
    }

    #[test]
    fn test_arqen_app_builder_no_modules() {
        let app = ArqenApp::builder().build().unwrap();
        assert_eq!(app.state.config.server.port, 8888);
    }

    #[test]
    fn test_arqen_app_builder_with_module() {
        let app = ArqenApp::builder()
            .module(TestModule)
            .build()
            .unwrap();
        assert_eq!(app.state.config.server.port, 8888);
    }

    #[test]
    fn test_arqen_app_builder_with_config() {
        let config = AppConfig {
            server: crate::config::ServerConfig {
                port: 9999,
                ..Default::default()
            },
            ..Default::default()
        };
        let app = ArqenApp::builder().config(config).build().unwrap();
        assert_eq!(app.state.config.server.port, 9999);
    }

    #[test]
    fn test_arqen_app_builder_with_explicit_state() {
        let state = AppState::builder().build().unwrap();
        let app = ArqenApp::builder().state(state).build().unwrap();
        assert_eq!(app.state.config.server.port, 8888);
    }

    #[test]
    fn test_arqen_app_builder_validation_error() {
        struct DepModule;
        #[async_trait::async_trait]
        impl Module for DepModule {
            fn name(&self) -> &str {
                "app"
            }
            fn dependencies(&self) -> Vec<&str> {
                vec!["nonexistent"]
            }
        }

        let result = ArqenApp::builder().module(DepModule).build();
        assert!(result.is_err());
    }
}
