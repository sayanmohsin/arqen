# Interfaces

## Module Trait

```rust
pub trait Module: Send + Sync {
    fn name(&self) -> &str;
    fn routes(&self) -> Router;
    fn middleware(&self) -> Vec<Box<dyn Layer<dyn Service<Request>>>>;
    fn state(&self) -> Option<Box<dyn Any>>;
}
```

## ModuleBuilder

```rust
pub struct ModuleBuilder {
    modules: Vec<Box<dyn Module>>,
}

impl ModuleBuilder {
    pub fn new() -> Self { ... }
    pub fn register<M: Module + 'static>(mut self, module: M) -> Self { ... }
    pub fn build(self) -> Router { ... }
}
```

## AppBuilder

```rust
pub struct AppBuilder {
    config: AppConfig,
    modules: Vec<Box<dyn Module>>,
}

impl AppBuilder {
    pub fn new(config: AppConfig) -> Self { ... }
    pub fn with_module<M: Module + 'static>(mut self, module: M) -> Self { ... }
    pub fn build(self) -> Result<AppState, ConfigError> { ... }
}
```
