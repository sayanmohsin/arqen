# Interfaces

## ObservabilityConfig

```rust
pub struct ObservabilityConfig {
    pub tracing: TracingConfig,
    pub metrics: MetricsConfig,
}

pub struct TracingConfig {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub sample_rate: f64,
}

pub struct MetricsConfig {
    pub enabled: bool,
    pub endpoint: String,
}
```

## RequestId

```rust
pub struct RequestId(pub String);

impl RequestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}
```

## Metrics

```rust
pub struct Metrics {
    pub requests_total: Counter,
    pub request_duration: Histogram,
    pub requests_in_flight: Gauge,
    pub errors_total: Counter,
}
```
