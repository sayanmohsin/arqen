# Interfaces

## Module Trait

The `Module` trait is the core abstraction for composing application functionality. It is defined in `module.rs` and requires `Send + Sync`.

```rust
#[async_trait]
pub trait Module: Send + Sync {
    fn name(&self) -> &str;
    fn routes(&self) -> Option<Vec<RouteEntry>> { None }
    fn dependencies(&self) -> Vec<&str> { Vec::new() }
    fn register(&self, _ctx: &mut ModuleContext<'_>) -> Result<(), AppError> { Ok(()) }
    async fn init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { Ok(()) }
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { Ok(()) }
    async fn health_check(&self) -> ModuleHealth { ModuleHealth::Healthy }
}
```

## ModuleContext

The context passed to `Module::register` for registering tools and health checks.

```rust
pub struct ModuleContext<'a> {
    pub tools: &'a mut ToolRegistry,
    pub health: &'a mut HealthRegistry,
}
```

## ModuleHealth

Health status reported by each module.

```rust
pub enum ModuleHealth {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}
```

## ModuleGraphError

Errors produced during module graph validation.

```rust
pub enum ModuleGraphError {
    DuplicateModule(String),
    MissingDependency { module: String, dependency: String },
    DependencyCycle(Vec<String>),
}
```

## ModuleError

Top-level error type for module operations.

```rust
pub enum ModuleError {
    Graph(ModuleGraphError),
    Registration { module: String, message: String },
}
```

## ModuleBuilder

Builder for registering modules, validating their dependency graph, and executing lifecycle hooks.

```rust
pub struct ModuleBuilder {
    modules: Vec<Arc<dyn Module>>,
}

impl ModuleBuilder {
    pub fn new() -> Self;
    pub fn register<M: Module + 'static>(self, module: M) -> Self;
    pub fn register_arc(self, module: Arc<dyn Module>) -> Self;
    pub fn validate(&self) -> Result<(), ModuleGraphError>;
    pub fn topological_order(&self) -> Result<Vec<&dyn Module>, ModuleGraphError>;
    pub fn topological_indices(&self) -> Result<Vec<usize>, ModuleGraphError>;
    pub fn register_all(&self, tools: &mut ToolRegistry, health: &mut HealthRegistry) -> Result<(), AppError>;
    pub async fn init_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    pub async fn shutdown_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    pub fn modules(&self) -> &[Arc<dyn Module>];
    pub fn module_count(&self) -> usize;
    pub fn module_names(&self) -> Vec<&str>;
}
```

## HttpModule

Extension trait for modules that expose an Axum router. Feature-gated on `http-server`.

```rust
pub trait HttpModule: Module {
    fn router(&self) -> Router<AppState>;
}

pub fn merge_module_routes(base: Router<AppState>, modules: &[Box<dyn HttpModule>]) -> Router<AppState>;
```

## ArqenApp

Top-level application entry point that wires state and modules.

```rust
pub struct ArqenApp {
    state: AppState,
    module_builder: ModuleBuilder,
}

impl ArqenApp {
    pub fn builder() -> ArqenAppBuilder;
    pub fn state(&self) -> &AppState;
    pub fn module_builder(&self) -> &ModuleBuilder;
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
```

## ArqenAppBuilder

Fluent builder for constructing an `ArqenApp`.

```rust
pub struct ArqenAppBuilder {
    name: Option<String>,
    config: Option<AppConfig>,
    state: Option<AppState>,
    modules: Vec<Arc<dyn Module>>,
}

impl ArqenAppBuilder {
    pub fn new() -> Self;
    pub fn name(self, name: impl Into<String>) -> Self;
    pub fn config(self, config: AppConfig) -> Self;
    pub fn state(self, state: AppState) -> Self;
    pub fn module<M: Module + 'static>(self, m: M) -> Self;
    pub fn build(self) -> Result<ArqenApp, ModuleError>;
}
```

## AppStateBuilder::with_modules

Helper on `AppStateBuilder` to register modules during state construction.

```rust
impl AppStateBuilder {
    pub fn with_modules<M: Module + 'static>(self, modules: Vec<M>) -> Result<Self, ModuleError>;
}
```
