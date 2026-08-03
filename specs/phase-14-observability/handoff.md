# Handoff

## OpenTelemetry Setup
- Configure via `ObservabilityConfig`
- Export traces to configurable endpoint
- Sample rate is configurable

## Prometheus Metrics
- `/metrics` endpoint returns Prometheus format
- Metrics: requests_total, request_duration, requests_in_flight, errors_total

## Request IDs
- Generated per-request (UUID v4)
- Returned in X-Request-Id response header
- Propagated through traces and logs

## Usage
```rust
let config = ObservabilityConfig {
    tracing: TracingConfig {
        enabled: true,
        endpoint: Some("http://localhost:4317".into()),
        sample_rate: 1.0,
    },
    metrics: MetricsConfig {
        enabled: true,
        endpoint: "/metrics".into(),
    },
};

let router = Router::new()
    .route("/metrics", get(metrics_handler))
    .layer(ObservabilityLayer::new(config));
```

## Migration Guide
- Add `ObservabilityConfig` to AppConfig
- Use `ObservabilityLayer` middleware
- Expose `/metrics` endpoint
