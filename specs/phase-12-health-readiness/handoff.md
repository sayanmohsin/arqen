# Handoff

## Health API
- `HealthCheck` trait for dependency checks
- `HealthRegistry` for registering and executing checks
- `HealthStatus` enum (Healthy, Degraded, Unhealthy)

## Usage

```rust
let mut registry = HealthRegistry::new();
registry.register(Arc::new(StorageHealthCheck::new(storage)));
registry.register(Arc::new(MemoryHealthCheck::new()));

let router = Router::new()
    .route("/health", get(health_handler))
    .route("/ready", get(ready_handler))
    .layer(Extension(Arc::new(registry)));
```

## Built-in Checks
- `StorageHealthCheck` - checks thingd storage connectivity
- `MemoryHealthCheck` - checks available memory
- `DiskHealthCheck` - checks disk space

## Migration Guide
- Replace basic health handler with registry-based checks
- Add dependency checks for critical components
- Configure timeouts for each check
