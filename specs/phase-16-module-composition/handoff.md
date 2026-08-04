# Handoff

## Module API
- `Module` trait: framework-neutral, async lifecycle hooks, dependency declaration
- `ModuleBuilder`: validates module graph, topological ordering, lifecycle management
- `ModuleContext<'a>`: explicit tool/health registration
- `ModuleHealth`: Healthy/Degraded/Unhealthy status enum
- `ModuleError`: top-level error combining graph and registration failures
- `HttpModule`: feature-gated HTTP route composition (Axum)
- `ArqenApp`: convenience wrapper with builder pattern and async lifecycle

## Usage Patterns

### Simple module
```rust
struct UsersModule;

#[async_trait]
impl Module for UsersModule {
    fn name(&self) -> &str { "users" }
    async fn health_check(&self) -> ModuleHealth { ModuleHealth::Healthy }
}
```

### Module with dependencies and registration
```rust
struct ApiModule;

#[async_trait]
impl Module for ApiModule {
    fn name(&self) -> &str { "api" }
    fn dependencies(&self) -> Vec<&str> { vec!["db"] }
    fn register(&self, ctx: &mut ModuleContext<'_>) -> Result<(), AppError> {
        ctx.tools.register_tool(ToolMetadata { ... });
        Ok(())
    }
}
```

### HTTP module
```rust
struct UsersModule;

impl Module for UsersModule {
    fn name(&self) -> &str { "users" }
}

impl HttpModule for UsersModule {
    fn router(&self) -> Router<AppState> {
        Router::new().route("/users", get(list_users))
    }
}
```

### App composition
```rust
ArqenApp::builder()
    .name("my-api")
    .module(UsersModule)
    .module(ApiModule)
    .build()?
    .start()
    .await
```

## Architecture Decisions
- Module trait is framework-neutral (no Axum dependency)
- HttpModule is separate and feature-gated on http-server
- No DI container — all wiring explicit via AppState and ModuleContext
- No automatic handler discovery — routes explicitly composed
- Lifecycle: init in dependency order, shutdown in reverse
- Best-effort shutdown: all modules attempted, errors logged

## Known Limitations
- ModuleBuilder is consumed by ArqenApp::builder().build(); cannot add modules after build
- No hot-reload of modules at runtime
- HttpModule requires manual route composition (no auto-discovery)
