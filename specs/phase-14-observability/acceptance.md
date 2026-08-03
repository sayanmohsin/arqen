# Acceptance Criteria

1. OpenTelemetry traces are exported to configurable endpoint
2. Prometheus metrics are exposed via /metrics endpoint
3. Request IDs are propagated through traces and logs
4. Tracing spans are structured (method, path, status, duration)
5. Metrics include request count, duration, errors, and in-flight requests
6. Tests verify tracing, metrics, and request ID propagation
