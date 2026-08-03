# Interfaces

## HealthCheck Trait

```rust
#[async_trait]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self) -> HealthStatus;
    fn timeout(&self) -> Duration { Duration::from_secs(5) }
}
```

## HealthStatus

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}
```

## HealthRegistry

```rust
pub struct HealthRegistry {
    checks: Vec<Arc<dyn HealthCheck>>,
}

impl HealthRegistry {
    pub fn new() -> Self { ... }
    pub fn register(&mut self, check: Arc<dyn HealthCheck>) { ... }
    pub async fn check_all(&self) -> HealthReport { ... }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub checks: Vec<CheckResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub status: HealthStatus,
    pub duration_ms: u64,
}
```
