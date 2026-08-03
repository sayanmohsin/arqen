# Phase 12: Health & Readiness

Objective: provide registered dependency checks with timeouts and degraded states for health and readiness endpoints.

Dependencies: 08, 09.

In scope: health registry, dependency checks, timeouts, degraded states, and integration with HTTP endpoints.

Out of scope: external monitoring systems, alerting.

Deliverables: `health.rs` module, health registry, dependency check trait, and tests.

Acceptance: `/health` returns liveness only; `/ready` returns 503 when dependencies fail; dependency checks have timeouts; degraded states are reported.

Tests: health registry, dependency checks, timeouts, degraded states.

Docs: update health and readiness guide.

Handoff: record health API, dependency checks, and degraded states.
