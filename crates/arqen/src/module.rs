//! Module composition for Arqen.
//!
//! Provides a trait-based system for composing application modules with
//! lifecycle hooks and dependency management.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;

use crate::agent::ToolRegistry;
use crate::core::{AppError, ErrorKind};
use crate::health::{HealthRegistry, HealthStatus};
use crate::HealthCheck;

/// Errors from module graph validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleGraphError {
    /// A module with this name is already registered.
    DuplicateModule(String),
    /// A module declares a dependency that is not registered.
    MissingDependency { module: String, dependency: String },
    /// A dependency cycle was detected.
    DependencyCycle(Vec<String>),
}

impl fmt::Display for ModuleGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModuleGraphError::DuplicateModule(name) => {
                write!(f, "duplicate module: {}", name)
            }
            ModuleGraphError::MissingDependency { module, dependency } => {
                write!(
                    f,
                    "module '{}' depends on '{}' which is not registered",
                    module, dependency
                )
            }
            ModuleGraphError::DependencyCycle(cycle) => {
                write!(f, "dependency cycle detected: {}", cycle.join(" -> "))
            }
        }
    }
}

impl std::error::Error for ModuleGraphError {}

/// Trait for application modules.
#[async_trait]
pub trait Module: Send + Sync {
    /// Module name. Must be unique across all registered modules.
    fn name(&self) -> &str;

    /// Module routes (documentation metadata, not HTTP handlers).
    fn routes(&self) -> Option<Vec<RouteEntry>> {
        None
    }

    /// Module dependencies (other module names this depends on).
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }

    /// Register tools and health checks into the provided registries.
    ///
    /// This is the explicit registration point for module capabilities.
    /// Called during app startup, before `init()`.
    fn register(&self, _ctx: &mut ModuleContext<'_>) -> Result<(), AppError> {
        Ok(())
    }

    /// Initialize the module (called once at startup, in dependency order).
    async fn init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    /// Shutdown the module (called once at shutdown, in reverse dependency order).
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

impl From<ModuleHealth> for HealthStatus {
    fn from(m: ModuleHealth) -> Self {
        match m {
            ModuleHealth::Healthy => HealthStatus::Healthy,
            ModuleHealth::Degraded { reason } => HealthStatus::Degraded { reason },
            ModuleHealth::Unhealthy { reason } => HealthStatus::Unhealthy { reason },
        }
    }
}

/// Context passed to `Module::register()` for explicit registration.
pub struct ModuleContext<'a> {
    pub tools: &'a mut ToolRegistry,
    pub health: &'a mut HealthRegistry,
}

/// Adapter that wraps a module's health check for the HealthRegistry.
struct ModuleHealthCheck {
    module: Arc<dyn Module>,
}

#[async_trait]
impl HealthCheck for ModuleHealthCheck {
    fn name(&self) -> &str {
        self.module.name()
    }

    async fn check(&self) -> HealthStatus {
        self.module.health_check().await.into()
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

    /// Get a module by name.
    pub fn get_module(&self, name: &str) -> Option<&Arc<dyn Module>> {
        self.modules.iter().find(|m| m.name() == name)
    }

    /// Validate the module graph.
    ///
    /// Checks for:
    /// - Duplicate module names
    /// - Missing dependencies
    /// - Dependency cycles
    pub fn validate(&self) -> Result<(), ModuleGraphError> {
        let mut seen = HashSet::new();
        for module in &self.modules {
            let name = module.name();
            if !seen.insert(name) {
                return Err(ModuleGraphError::DuplicateModule(name.to_string()));
            }
        }

        for module in &self.modules {
            let name = module.name();
            for dep in module.dependencies() {
                if !seen.contains(dep) {
                    return Err(ModuleGraphError::MissingDependency {
                        module: name.to_string(),
                        dependency: dep.to_string(),
                    });
                }
            }
        }

        if let Some(cycle) = self.detect_cycle() {
            return Err(ModuleGraphError::DependencyCycle(cycle));
        }

        Ok(())
    }

    /// Return module indices in topological order (dependencies first).
    ///
    /// Returns `Err` if the graph has cycles, missing deps, or duplicates.
    fn topological_indices(&self) -> Result<Vec<usize>, ModuleGraphError> {
        self.validate()?;

        let name_to_index: HashMap<&str, usize> = self
            .modules
            .iter()
            .enumerate()
            .map(|(i, m)| (m.name(), i))
            .collect();

        let mut visited = HashSet::new();
        let mut order = Vec::new();

        fn visit(
            idx: usize,
            modules: &[Arc<dyn Module>],
            name_to_index: &HashMap<&str, usize>,
            visited: &mut HashSet<usize>,
            order: &mut Vec<usize>,
        ) {
            if !visited.insert(idx) {
                return;
            }
            for dep_name in modules[idx].dependencies() {
                if let Some(&dep_idx) = name_to_index.get(dep_name) {
                    visit(dep_idx, modules, name_to_index, visited, order);
                }
            }
            order.push(idx);
        }

        for i in 0..self.modules.len() {
            visit(i, &self.modules, &name_to_index, &mut visited, &mut order);
        }

        Ok(order)
    }

    /// Return modules in topological order (dependencies first).
    ///
    /// Returns `Err` if the graph has cycles, missing deps, or duplicates.
    pub fn topological_order(&self) -> Result<Vec<&dyn Module>, ModuleGraphError> {
        self.validate()?;

        let name_to_index: HashMap<&str, usize> = self
            .modules
            .iter()
            .enumerate()
            .map(|(i, m)| (m.name(), i))
            .collect();

        let mut visited = HashSet::new();
        let mut order = Vec::new();

        fn visit<'a>(
            idx: usize,
            modules: &'a [Arc<dyn Module>],
            name_to_index: &HashMap<&str, usize>,
            visited: &mut HashSet<usize>,
            order: &mut Vec<&'a dyn Module>,
        ) {
            if !visited.insert(idx) {
                return;
            }
            for dep_name in modules[idx].dependencies() {
                if let Some(&dep_idx) = name_to_index.get(dep_name) {
                    visit(dep_idx, modules, name_to_index, visited, order);
                }
            }
            order.push(&*modules[idx]);
        }

        for i in 0..self.modules.len() {
            visit(i, &self.modules, &name_to_index, &mut visited, &mut order);
        }

        Ok(order)
    }

    /// Initialize all modules in dependency order.
    pub async fn init_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let indices = self
            .topological_indices()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        for idx in indices {
            self.modules[idx].init().await?;
        }
        Ok(())
    }

    /// Shutdown all modules in reverse dependency order.
    pub async fn shutdown_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let indices = self
            .topological_indices()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        for idx in indices.into_iter().rev() {
            self.modules[idx].shutdown().await?;
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

    /// Get the registered modules.
    pub fn modules(&self) -> &[Arc<dyn Module>] {
        &self.modules
    }

    /// Register all modules' tools and health checks into the provided registries.
    ///
    /// Calls `Module::register()` on each module in dependency order.
    pub fn register_all(
        &self,
        tools: &mut ToolRegistry,
        health: &mut HealthRegistry,
    ) -> Result<(), AppError> {
        let indices = self.topological_indices().map_err(|e| {
            AppError::new(ErrorKind::Internal, format!("module graph error: {}", e))
        })?;

        for &idx in &indices {
            let module = &self.modules[idx];
            let mut ctx = ModuleContext { tools, health };
            module.register(&mut ctx)?;

            // Auto-register module health check
            health.register(Arc::new(ModuleHealthCheck {
                module: module.clone(),
            }));
        }

        Ok(())
    }

    /// Detect dependency cycles using DFS.
    fn detect_cycle(&self) -> Option<Vec<String>> {
        let name_to_index: HashMap<&str, usize> = self
            .modules
            .iter()
            .enumerate()
            .map(|(i, m)| (m.name(), i))
            .collect();

        #[derive(Clone, Copy, PartialEq)]
        enum State {
            White,
            Gray,
            Black,
        }

        let mut state = vec![State::White; self.modules.len()];
        let mut parent = vec![None::<usize>; self.modules.len()];

        for start in 0..self.modules.len() {
            if state[start] != State::White {
                continue;
            }

            let mut stack = vec![start];
            while let Some(&idx) = stack.last() {
                if state[idx] == State::White {
                    state[idx] = State::Gray;
                    for dep_name in self.modules[idx].dependencies() {
                        if let Some(&dep_idx) = name_to_index.get(dep_name) {
                            if state[dep_idx] == State::Gray {
                                // Found cycle — reconstruct path
                                let mut cycle = vec![self.modules[dep_idx].name().to_string()];
                                let mut cur = idx;
                                loop {
                                    cycle.push(self.modules[cur].name().to_string());
                                    if cur == dep_idx {
                                        break;
                                    }
                                    cur = parent[cur].unwrap();
                                }
                                cycle.reverse();
                                return Some(cycle);
                            }
                            if state[dep_idx] == State::White {
                                parent[dep_idx] = Some(idx);
                                stack.push(dep_idx);
                            }
                        }
                    }
                    // If we pushed children, we'll revisit this node.
                    // If not, mark as done.
                    if stack.last() == Some(&idx) {
                        state[idx] = State::Black;
                        stack.pop();
                    }
                } else if state[idx] == State::Gray {
                    state[idx] = State::Black;
                    stack.pop();
                } else {
                    stack.pop();
                }
            }
        }

        None
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
    use std::sync::Mutex;

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

    // --- Phase 1 tests: validation and topological ordering ---

    #[test]
    fn test_validate_ok() {
        let builder = ModuleBuilder::new()
            .register(EmptyModule::new("base"))
            .register(EmptyModule::new("app"));
        assert!(builder.validate().is_ok());
    }

    #[test]
    fn test_validate_duplicate_module() {
        let builder = ModuleBuilder::new()
            .register(EmptyModule::new("dup"))
            .register(EmptyModule::new("dup"));
        let err = builder.validate().unwrap_err();
        assert_eq!(err, ModuleGraphError::DuplicateModule("dup".to_string()));
    }

    #[test]
    fn test_validate_missing_dependency() {
        struct DepModule;
        #[async_trait]
        impl Module for DepModule {
            fn name(&self) -> &str {
                "app"
            }
            fn dependencies(&self) -> Vec<&str> {
                vec!["missing"]
            }
        }

        let builder = ModuleBuilder::new().register(DepModule);
        let err = builder.validate().unwrap_err();
        assert_eq!(
            err,
            ModuleGraphError::MissingDependency {
                module: "app".to_string(),
                dependency: "missing".to_string(),
            }
        );
    }

    #[test]
    fn test_validate_dependency_cycle() {
        struct ModuleA;
        #[async_trait]
        impl Module for ModuleA {
            fn name(&self) -> &str {
                "a"
            }
            fn dependencies(&self) -> Vec<&str> {
                vec!["b"]
            }
        }

        struct ModuleB;
        #[async_trait]
        impl Module for ModuleB {
            fn name(&self) -> &str {
                "b"
            }
            fn dependencies(&self) -> Vec<&str> {
                vec!["a"]
            }
        }

        let builder = ModuleBuilder::new().register(ModuleA).register(ModuleB);
        let err = builder.validate().unwrap_err();
        assert!(matches!(err, ModuleGraphError::DependencyCycle(_)));
    }

    #[test]
    fn test_topological_order_linear() {
        struct DepModule {
            dep: &'static str,
        }
        #[async_trait]
        impl Module for DepModule {
            fn name(&self) -> &str {
                "dep"
            }
            fn dependencies(&self) -> Vec<&str> {
                vec![self.dep]
            }
        }

        // Register in reverse order: dep depends on base, but registered first
        let builder = ModuleBuilder::new()
            .register(DepModule { dep: "base" })
            .register(EmptyModule::new("base"));

        let order = builder.topological_order().unwrap();
        let names: Vec<&str> = order.iter().map(|m| m.name()).collect();
        assert_eq!(names, vec!["base", "dep"]);
    }

    #[test]
    fn test_topological_order_diamond() {
        //     app
        //    /   \
        //   b     c
        //    \   /
        //     base
        struct DiamondModule {
            deps: Vec<&'static str>,
        }
        #[async_trait]
        impl Module for DiamondModule {
            fn name(&self) -> &str {
                "app"
            }
            fn dependencies(&self) -> Vec<&str> {
                self.deps.clone()
            }
        }

        struct MidModule {
            name_val: &'static str,
            dep: &'static str,
        }
        #[async_trait]
        impl Module for MidModule {
            fn name(&self) -> &str {
                self.name_val
            }
            fn dependencies(&self) -> Vec<&str> {
                vec![self.dep]
            }
        }

        let builder = ModuleBuilder::new()
            .register(DiamondModule {
                deps: vec!["b", "c"],
            })
            .register(MidModule {
                name_val: "b",
                dep: "base",
            })
            .register(MidModule {
                name_val: "c",
                dep: "base",
            })
            .register(EmptyModule::new("base"));

        let order = builder.topological_order().unwrap();
        let names: Vec<&str> = order.iter().map(|m| m.name()).collect();

        // base must come before b and c, b and c must come before app
        let base_pos = names.iter().position(|&n| n == "base").unwrap();
        let b_pos = names.iter().position(|&n| n == "b").unwrap();
        let c_pos = names.iter().position(|&n| n == "c").unwrap();
        let app_pos = names.iter().position(|&n| n == "app").unwrap();

        assert!(base_pos < b_pos);
        assert!(base_pos < c_pos);
        assert!(b_pos < app_pos);
        assert!(c_pos < app_pos);
    }

    #[tokio::test]
    async fn test_shutdown_reverse_order() {
        struct OrderTracker {
            name_val: String,
            init_order: Arc<Mutex<Vec<String>>>,
            shutdown_order: Arc<Mutex<Vec<String>>>,
        }

        #[async_trait]
        impl Module for OrderTracker {
            fn name(&self) -> &str {
                &self.name_val
            }
            async fn init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                self.init_order.lock().unwrap().push(self.name_val.clone());
                Ok(())
            }
            async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                self.shutdown_order
                    .lock()
                    .unwrap()
                    .push(self.name_val.clone());
                Ok(())
            }
        }

        let init_order = Arc::new(Mutex::new(Vec::<String>::new()));
        let shutdown_order = Arc::new(Mutex::new(Vec::<String>::new()));

        let builder = ModuleBuilder::new()
            .register(OrderTracker {
                name_val: "first".to_string(),
                init_order: init_order.clone(),
                shutdown_order: shutdown_order.clone(),
            })
            .register(OrderTracker {
                name_val: "second".to_string(),
                init_order: init_order.clone(),
                shutdown_order: shutdown_order.clone(),
            })
            .register(OrderTracker {
                name_val: "third".to_string(),
                init_order: init_order.clone(),
                shutdown_order: shutdown_order.clone(),
            });

        builder.init_all().await.unwrap();
        builder.shutdown_all().await.unwrap();

        let init = init_order.lock().unwrap();
        let shutdown = shutdown_order.lock().unwrap();

        assert_eq!(*init, vec!["first", "second", "third"]);
        assert_eq!(*shutdown, vec!["third", "second", "first"]);
    }

    #[test]
    fn test_detect_cycle_three_nodes() {
        struct Cyclical;
        #[async_trait]
        impl Module for Cyclical {
            fn name(&self) -> &str {
                "x"
            }
            fn dependencies(&self) -> Vec<&str> {
                vec!["y"]
            }
        }

        struct Cyclical2;
        #[async_trait]
        impl Module for Cyclical2 {
            fn name(&self) -> &str {
                "y"
            }
            fn dependencies(&self) -> Vec<&str> {
                vec!["z"]
            }
        }

        struct Cyclical3;
        #[async_trait]
        impl Module for Cyclical3 {
            fn name(&self) -> &str {
                "z"
            }
            fn dependencies(&self) -> Vec<&str> {
                vec!["x"]
            }
        }

        let builder = ModuleBuilder::new()
            .register(Cyclical)
            .register(Cyclical2)
            .register(Cyclical3);

        let err = builder.validate().unwrap_err();
        assert!(matches!(err, ModuleGraphError::DependencyCycle(_)));
    }

    #[test]
    fn test_get_module() {
        let builder = ModuleBuilder::new()
            .register(EmptyModule::new("a"))
            .register(EmptyModule::new("b"));

        assert!(builder.get_module("a").is_some());
        assert!(builder.get_module("b").is_some());
        assert!(builder.get_module("c").is_none());
    }

    #[test]
    fn test_module_graph_error_display() {
        let e1 = ModuleGraphError::DuplicateModule("dup".to_string());
        assert!(format!("{}", e1).contains("dup"));

        let e2 = ModuleGraphError::MissingDependency {
            module: "app".to_string(),
            dependency: "lib".to_string(),
        };
        assert!(format!("{}", e2).contains("app"));
        assert!(format!("{}", e2).contains("lib"));

        let e3 = ModuleGraphError::DependencyCycle(vec!["a".into(), "b".into(), "a".into()]);
        assert!(format!("{}", e3).contains("a -> b -> a"));
    }

    // --- Phase 2 tests: ModuleContext and register() ---

    #[test]
    fn test_module_register_tools() {
        use crate::agent::{ToolEffect, ToolMetadata};
        use crate::health::HealthRegistry;

        struct ToolModule;
        #[async_trait]
        impl Module for ToolModule {
            fn name(&self) -> &str {
                "tools"
            }
            fn register(&self, ctx: &mut ModuleContext<'_>) -> Result<(), AppError> {
                ctx.tools.register_tool(ToolMetadata {
                    name: "get_user".to_string(),
                    description: "Get a user".to_string(),
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
        }

        let mut tools = ToolRegistry::new("test", "0.1.0", "test", "memory");
        let mut health = HealthRegistry::new();

        let builder = ModuleBuilder::new().register(ToolModule);
        builder.register_all(&mut tools, &mut health).unwrap();

        assert!(tools.get_tool("get_user").is_some());
        assert_eq!(tools.list_tools().len(), 1);
    }

    #[test]
    fn test_module_register_health_checks() {
        use crate::health::HealthRegistry;

        struct HealthModule;
        #[async_trait]
        impl Module for HealthModule {
            fn name(&self) -> &str {
                "health"
            }
            fn register(&self, ctx: &mut ModuleContext<'_>) -> Result<(), AppError> {
                // Register a tool to verify registration works
                ctx.tools.register_tool(crate::agent::ToolMetadata {
                    name: "health_tool".to_string(),
                    description: "Health tool".to_string(),
                    input: serde_json::json!({}),
                    output: serde_json::json!({}),
                    scopes: vec![],
                    effect: crate::agent::ToolEffect::Read,
                    idempotent: true,
                    enqueues_job: None,
                    timeout: None,
                });
                Ok(())
            }
        }

        let mut tools = ToolRegistry::new("test", "0.1.0", "test", "memory");
        let mut health = HealthRegistry::new();

        let builder = ModuleBuilder::new().register(HealthModule);
        builder.register_all(&mut tools, &mut health).unwrap();

        // Verify the module's register was called
        assert!(tools.get_tool("health_tool").is_some());
        // Health registry should have the module's health check auto-registered
        let rt = tokio::runtime::Runtime::new().unwrap();
        let report = rt.block_on(health.check_liveness());
        assert_eq!(report.checks.len(), 1);
        assert_eq!(report.checks[0].name, "health");
    }

    #[test]
    fn test_module_health_conversion() {
        let healthy: HealthStatus = ModuleHealth::Healthy.into();
        assert_eq!(healthy, HealthStatus::Healthy);

        let degraded: HealthStatus = ModuleHealth::Degraded {
            reason: "slow".to_string(),
        }
        .into();
        assert!(degraded.is_degraded());

        let unhealthy: HealthStatus = ModuleHealth::Unhealthy {
            reason: "down".to_string(),
        }
        .into();
        assert!(unhealthy.is_unhealthy());
    }

    #[test]
    fn test_register_all_respects_dependency_order() {
        use std::sync::Mutex;

        struct OrderedModule {
            name_val: String,
            dep: Option<String>,
            order: Arc<Mutex<Vec<String>>>,
        }

        #[async_trait]
        impl Module for OrderedModule {
            fn name(&self) -> &str {
                &self.name_val
            }
            fn dependencies(&self) -> Vec<&str> {
                match &self.dep {
                    Some(d) => vec![d.as_str()],
                    None => vec![],
                }
            }
            fn register(&self, _ctx: &mut ModuleContext<'_>) -> Result<(), AppError> {
                self.order.lock().unwrap().push(self.name_val.clone());
                Ok(())
            }
        }

        let order = Arc::new(Mutex::new(Vec::<String>::new()));

        // Register in reverse order: app depends on db, but registered first
        let builder = ModuleBuilder::new()
            .register(OrderedModule {
                name_val: "app".to_string(),
                dep: Some("db".to_string()),
                order: order.clone(),
            })
            .register(OrderedModule {
                name_val: "db".to_string(),
                dep: None,
                order: order.clone(),
            });

        let mut tools = ToolRegistry::new("test", "0.1.0", "test", "memory");
        let mut health = HealthRegistry::new();
        builder.register_all(&mut tools, &mut health).unwrap();

        let registered = order.lock().unwrap();
        assert_eq!(*registered, vec!["db", "app"]);
    }

    #[test]
    fn test_register_all_validation_error() {
        struct MissingDep;
        #[async_trait]
        impl Module for MissingDep {
            fn name(&self) -> &str {
                "app"
            }
            fn dependencies(&self) -> Vec<&str> {
                vec!["nonexistent"]
            }
        }

        let mut tools = ToolRegistry::new("test", "0.1.0", "test", "memory");
        let mut health = HealthRegistry::new();

        let builder = ModuleBuilder::new().register(MissingDep);
        let result = builder.register_all(&mut tools, &mut health);
        assert!(result.is_err());
    }
}
