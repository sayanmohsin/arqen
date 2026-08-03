# Phase 14: Observability

Objective: provide OpenTelemetry traces, Prometheus metrics, and request IDs for production monitoring.

Dependencies: 08, 09.

In scope: OpenTelemetry integration, Prometheus metrics, request ID middleware, and tracing spans.

Out of scope: external monitoring systems, alerting, dashboards.

Deliverables: `observability.rs` module, OpenTelemetry setup, Prometheus metrics, request ID middleware, and tests.

Acceptance: traces are exported to OpenTelemetry; metrics are exposed via Prometheus; request IDs are propagated; tracing spans are structured.

Tests: OpenTelemetry setup, Prometheus metrics, request ID propagation, tracing spans.

Docs: update observability guide.

Handoff: record observability setup, metrics, and request ID strategy.
