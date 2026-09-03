# Health and readiness

Health checks let liveness and readiness probes observe dependency state.

`ARQEN_HEALTH_CHECK_TIMEOUT` configures the default check timeout (5 seconds
by default), and `ARQEN_HEALTH_STARTUP_DELAY` (disabled by default) keeps
readiness unhealthy during the dependency warm-up window while liveness remains available. Checks taking
more than three seconds emit a structured `slow health check` warning. The
startup grace period is applied when an application registers its checks
through `AppState`.
Arqen provides a registry-based system with parallel execution, timeouts,
and degraded states.

## Concepts

### HealthStatus

Every check returns one of three states:

| Status                 | HTTP code | Meaning                               |
| ---------------------- | --------- | ------------------------------------- |
| `Healthy`              | 200       | Dependency is operating normally      |
| `Degraded { reason }`  | 200       | Dependency is functional but impaired |
| `Unhealthy { reason }` | 503       | Dependency is not functioning         |

### Liveness vs readiness

- **Liveness** (`/health`): Is the application alive? Runs all registered
  checks. Used by orchestrators to decide whether to restart the process.
- **Readiness** (`/ready`): Is the application ready to serve traffic? Runs
  only checks where `required_for_readiness()` returns `true`. Used to decide
  whether to route traffic to the instance.

## Types

### HealthCheck trait

```rust
#[arqen::async_trait]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self) -> HealthStatus;
    fn timeout(&self) -> Duration { Duration::from_secs(5) }
    fn required_for_readiness(&self) -> bool { true }
}
```

Implement this trait for each dependency you want to monitor. The `timeout`
method defaults to 5 seconds. Set `required_for_readiness()` to `false` for
non-critical checks.

### HealthRegistry

Collects health checks and runs them in parallel:

```rust
use arqen::health::{HealthRegistry, AlwaysHealthy};
use std::sync::Arc;

let mut registry = HealthRegistry::new();
registry.register(Arc::new(AlwaysHealthy));

let report = registry.check_liveness().await;
// report.status == HealthStatus::Healthy
// report.checks.len() == 1
```

### HealthReport

The result of running all checks:

```rust
pub struct HealthReport {
    pub status: HealthStatus,
    pub checks: Vec<CheckResult>,
    pub timestamp: String,
    pub probe_type: ProbeType, // Liveness or Readiness
}
```

### ModuleHealth

Module-level health status that converts into `HealthStatus`:

```rust
pub enum ModuleHealth {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}
```

Modules return this from `health_check()`. It is automatically registered
with the health registry when `ModuleBuilder::register_all()` is called.

## HTTP endpoints

| Endpoint      | Probe type | Behavior                                 |
| ------------- | ---------- | ---------------------------------------- |
| `GET /health` | Liveness   | Runs all checks, returns 200 or 503      |
| `GET /ready`  | Readiness  | Runs required checks, returns 200 or 503 |

Both endpoints return JSON:

```json
{
  "status": "healthy",
  "checks": [
    {
      "name": "database",
      "status": "healthy",
      "duration_ms": 2
    }
  ],
  "timestamp": "2026-08-05T12:00:00Z",
  "probe_type": "liveness"
}
```

## Registering checks

### From a module

Use `ModuleContext` in `Module::register()` to register checks:

```rust
fn register(&self, ctx: &mut ModuleContext<'_>) -> Result<(), AppError> {
    ctx.health.register(Arc::new(MyDatabaseCheck { /* ... */ }));
    Ok(())
}
```

Module health checks are also auto-registered from the `health_check()`
method.

### Directly on the registry

```rust
let mut registry = HealthRegistry::new();
registry.register(Arc::new(DatabaseCheck::new("postgres://...")));
registry.register(Arc::new(RedisCheck::new("redis://...")));
registry.register(Arc::new(ExternalApiCheck::new("https://api.example.com")));
```

## Code example

From the health module tests (`health.rs`):

```rust
use arqen::health::{HealthRegistry, AlwaysHealthy, AlwaysDegraded, OptionalCheck};
use std::sync::Arc;

fn main() {
    arqen::run(async {
        let mut registry = HealthRegistry::new();
        registry.register(Arc::new(AlwaysHealthy));
        registry.register(Arc::new(OptionalCheck));

        let liveness = registry.check_liveness().await;
        assert_eq!(liveness.checks.len(), 2); // runs all

        let readiness = registry.check_readiness().await;
        assert_eq!(readiness.checks.len(), 1); // skips optional
    });
}
```

See: [`crates/arqen/src/health.rs`](https://github.com/sayanmohsin/arqen/blob/main/crates/arqen/src/health.rs)
