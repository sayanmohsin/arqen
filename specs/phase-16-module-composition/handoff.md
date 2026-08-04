# Handoff

## Module API
- `Module` trait for module composition
- `ModuleBuilder` for registering modules
- `AppBuilder` for composing modules

## Usage

```rust
let app = AppBuilder::new(config)
    .with_module(AuthModule::new(auth))
    .with_module(StorageModule::new(storage))
    .with_module(AgentModule::new(registry))
    .build()?;
```

## Built-in Modules
- `AuthModule` - authentication and authorization
- `StorageModule` - thingd storage adapter
- `AgentModule` - tool registry and agent manifest

## Custom Modules

```rust
pub struct MyModule;

impl Module for MyModule {
    fn name(&self) -> &str { "my-module" }
    fn routes(&self) -> Router { ... }
    fn middleware(&self) -> Vec<Box<dyn Layer<...>>> { ... }
    fn state(&self) -> Option<Box<dyn Any>> { ... }
}
```

## Migration Guide
- Replace manual router composition with ModuleBuilder
- Use built-in modules for common functionality
- Create custom modules for domain-specific logic
