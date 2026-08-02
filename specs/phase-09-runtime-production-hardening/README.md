# Phase 09: runtime production hardening

Objective: make lifecycle, readiness, request safety, configuration, and
observability safe in deployment. Outcome: truthful health responses, request
IDs, structured logs, bounded requests, and graceful shutdown.

Dependencies: 02, 03, 04. In scope: runtime state, readiness, limits,
timeouts, request IDs, shutdown, secret redaction, config validation, worker
draining. Out of scope: identity-provider implementation and cloud control plane.

Deliverables: typed configuration, operational middleware, health behavior, and tests.

Acceptance: `/health` is liveness-only; `/ready` returns 503 when required
dependencies fail; secrets never appear in logs or manifests; SIGINT/SIGTERM
drains requests and workers within a documented deadline.

Tests: middleware, readiness failure, shutdown, redaction, and config tests.
Docs: update configuration, logging, security, and deployment docs.
Handoff: record signal behavior, health dependency policy, and assumptions.
