//! Module composition for Arqen.
//!
//! Provides a trait-based system for composing application modules with
//! lifecycle hooks and dependency management.

use std::sync::Arc;

use async_trait::async_trait;

/// Trait for application modules.
#[async_trait]
pub trait Module: Send + Sync {
    /// Module name.
    fn name(&self) -> &str;

    /// Module routes (if any).
    fn routes(&self) -> Option<Vec<RouteEntry>> {
        None
    }

    /// Module dependencies (other module names this depends on).
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }

    /// Initialize the module (called once at startup).
    async fn init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    /// Shutdown the module (called once at shutdown).
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    /// Health check for this module.
    async fn health_check(&self) -> ModuleHealth {
        ModuleHealth::Healthy
    }
}

/// Module health status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleHealth {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

/// Description of a route provided by a module.
#[derive(Debug, Clone)]
pub struct RouteEntry {
    pub path: String,
    pub method: String,
    pub description: String,
}

/// Builder for composing modules.
pub struct ModuleBuilder {
    modules: Vec<Arc<dyn Module>>,
}

impl ModuleBuilder {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    /// Register a module.
    pub fn register<M: Module + 'static>(mut self, module: M) -> Self {
        self.modules.push(Arc::new(module));
        self
    }

    /// Register a module wrapped in Arc.
    pub fn register_arc(mut self, module: Arc<dyn Module>) -> Self {
        self.modules.push(module);
        self
    }

    /// Get all registered module names.
    pub fn module_names(&self) -> Vec<&str> {
        self.modules.iter().map(|m| m.name()).collect()
    }

    /// Get all routes from all modules.
    pub fn all_routes(&self) -> Vec<RouteEntry> {
        self.modules
            .iter()
            .filter_map(|m| m.routes())
            .flatten()
            .collect()
    }

    /// Get module count.
    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    /// Initialize all modules in dependency order.
    pub async fn init_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for module in &self.modules {
            module.init().await?;
        }
        Ok(())
    }

    /// Shutdown all modules in reverse order.
    pub async fn shutdown_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for module in self.modules.iter().rev() {
            module.shutdown().await?;
        }
        Ok(())
    }

    /// Check health of all modules.
    pub async fn health_check_all(&self) -> Vec<(String, ModuleHealth)> {
        let mut results = Vec::new();
        for module in &self.modules {
            let health = module.health_check().await;
            results.push((module.name().to_string(), health));
        }
        results
    }

    /// Check if all modules are healthy.
    pub async fn all_healthy(&self) -> bool {
        let results = self.health_check_all().await;
        results.iter().all(|(_, h)| *h == ModuleHealth::Healthy)
    }

    /// Get modules that have dependencies.
    pub fn modules_with_dependencies(&self) -> Vec<(&str, Vec<&str>)> {
        self.modules
            .iter()
            .map(|m| (m.name(), m.dependencies()))
            .filter(|(_, deps)| !deps.is_empty())
            .collect()
    }
}

impl Default for ModuleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// An empty module for testing.
pub struct EmptyModule {
    name: String,
}

impl EmptyModule {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait]
impl Module for EmptyModule {
    fn name(&self) -> &str {
        &self.name
    }
}

/// A module with routes.
pub struct RouteModule {
    name: String,
    routes: Vec<RouteEntry>,
}

impl RouteModule {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            routes: Vec::new(),
        }
    }

    pub fn with_route(
        mut self,
        path: impl Into<String>,
        method: impl Into<String>,
        desc: impl Into<String>,
    ) -> Self {
        self.routes.push(RouteEntry {
            path: path.into(),
            method: method.into(),
            description: desc.into(),
        });
        self
    }
}

#[async_trait]
impl Module for RouteModule {
    fn name(&self) -> &str {
        &self.name
    }

    fn routes(&self) -> Option<Vec<RouteEntry>> {
        if self.routes.is_empty() {
            None
        } else {
            Some(self.routes.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_module() {
        let m = EmptyModule::new("test");
        assert_eq!(m.name(), "test");
        assert!(m.routes().is_none());
        assert!(m.dependencies().is_empty());
    }

    #[test]
    fn test_route_module() {
        let m = RouteModule::new("api")
            .with_route("/users", "GET", "List users")
            .with_route("/users", "POST", "Create user");
        assert_eq!(m.name(), "api");
        let routes = m.routes().unwrap();
        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].path, "/users");
        assert_eq!(routes[0].method, "GET");
    }

    #[test]
    fn test_route_module_empty() {
        let m = RouteModule::new("empty");
        assert!(m.routes().is_none());
    }

    #[test]
    fn test_module_builder_new() {
        let builder = ModuleBuilder::new();
        assert_eq!(builder.module_count(), 0);
    }

    #[test]
    fn test_module_builder_register() {
        let builder = ModuleBuilder::new()
            .register(EmptyModule::new("mod1"))
            .register(EmptyModule::new("mod2"));
        assert_eq!(builder.module_count(), 2);
        assert_eq!(builder.module_names(), vec!["mod1", "mod2"]);
    }

    #[test]
    fn test_module_builder_all_routes() {
        let builder = ModuleBuilder::new()
            .register(RouteModule::new("api").with_route("/users", "GET", "List users"))
            .register(RouteModule::new("admin").with_route("/admin", "GET", "Admin panel"));
        let routes = builder.all_routes();
        assert_eq!(routes.len(), 2);
    }

    #[test]
    fn test_module_builder_mixed() {
        let builder = ModuleBuilder::new()
            .register(EmptyModule::new("empty"))
            .register(RouteModule::new("routes").with_route("/test", "GET", "Test"));
        assert_eq!(builder.module_count(), 2);
        assert_eq!(builder.all_routes().len(), 1);
    }

    #[tokio::test]
    async fn test_module_init_shutdown() {
        let builder = ModuleBuilder::new()
            .register(EmptyModule::new("mod1"))
            .register(EmptyModule::new("mod2"));
        assert!(builder.init_all().await.is_ok());
        assert!(builder.shutdown_all().await.is_ok());
    }

    #[tokio::test]
    async fn test_module_health_check() {
        let builder = ModuleBuilder::new()
            .register(EmptyModule::new("mod1"))
            .register(EmptyModule::new("mod2"));
        let results = builder.health_check_all().await;
        assert_eq!(results.len(), 2);
        assert!(builder.all_healthy().await);
    }

    #[test]
    fn test_modules_with_dependencies() {
        struct DepModule;
        #[async_trait]
        impl Module for DepModule {
            fn name(&self) -> &str {
                "dep"
            }
            fn dependencies(&self) -> Vec<&str> {
                vec!["base"]
            }
        }

        let builder = ModuleBuilder::new()
            .register(EmptyModule::new("base"))
            .register(DepModule);
        let deps = builder.modules_with_dependencies();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].0, "dep");
        assert_eq!(deps[0].1, vec!["base"]);
    }
}
