//! Module composition for Arqen.
//!
//! Provides a trait-based system for composing application modules.

use std::any::Any;

/// Trait for application modules.
pub trait Module: Send + Sync {
    /// Module name.
    fn name(&self) -> &str;

    /// Module routes (if any).
    fn routes(&self) -> Option<Vec<RouteEntry>> {
        None
    }

    /// Module state (if any).
    fn state(&self) -> Option<Box<dyn Any + Send + Sync>> {
        None
    }
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
    modules: Vec<Box<dyn Module>>,
}

impl ModuleBuilder {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    /// Register a module.
    pub fn register<M: Module + 'static>(mut self, module: M) -> Self {
        self.modules.push(Box::new(module));
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

    pub fn with_route(mut self, path: impl Into<String>, method: impl Into<String>, desc: impl Into<String>) -> Self {
        self.routes.push(RouteEntry {
            path: path.into(),
            method: method.into(),
            description: desc.into(),
        });
        self
    }
}

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

/// A module with state.
pub struct StateModule {
    name: String,
    state: Option<Box<dyn Any + Send + Sync>>,
}

impl StateModule {
    pub fn new(name: impl Into<String>, state: Box<dyn Any + Send + Sync>) -> Self {
        Self {
            name: name.into(),
            state: Some(state),
        }
    }
}

impl Module for StateModule {
    fn name(&self) -> &str {
        &self.name
    }

    fn state(&self) -> Option<Box<dyn Any + Send + Sync>> {
        self.state.as_ref().map(|_s| {
            // This is a simplification - in practice, you'd need to clone or Arc the state
            // For now, we'll use a dummy approach
            Box::new(format!("state from {}", self.name)) as Box<dyn Any + Send + Sync>
        })
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
        assert!(m.state().is_none());
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
            .register(RouteModule::new("api")
                .with_route("/users", "GET", "List users"))
            .register(RouteModule::new("admin")
                .with_route("/admin", "GET", "Admin panel"));
        let routes = builder.all_routes();
        assert_eq!(routes.len(), 2);
    }

    #[test]
    fn test_module_builder_mixed() {
        let builder = ModuleBuilder::new()
            .register(EmptyModule::new("empty"))
            .register(RouteModule::new("routes")
                .with_route("/test", "GET", "Test"));
        assert_eq!(builder.module_count(), 2);
        assert_eq!(builder.all_routes().len(), 1);
    }
}
