# Feature status

This table is deliberately conservative. “Documented” means the contract or
design exists; it does not mean every production path is complete.

| Area | Status | Notes |
|---|---|---|
| Rust workspace and Axum HTTP server | Available | One `arqen` package provides the library and feature-gated CLI binary, with request limits, timeouts, CORS, correlation IDs, health, readiness, docs, and agent routes. |
| Typed configuration | Available / early | Layered CLI → environment → file → defaults loading, validation, and secret redaction are implemented. |
| Authentication and authorization | Available / partial | JWT, API-key, session adapters, constant-time API-key checks, hashing, and policy combinators exist; route integration and deployment key lifecycle remain application responsibilities. |
| Request validation | Available / partial | `Validate` and `Validated<T>` support field, enum, cross-field, and nested checks; no derive procedural macro is promised. |
| Stable error contracts | Available | Error codes, correlation IDs, redaction, timeout, dependency, auth, validation, and internal mappings exist. |
| Health and readiness | Available / partial | Parallel checks, liveness/readiness probes, statuses, timeouts, and HTTP mappings exist; applications must register real dependency checks. |
| In-memory storage mode | Available | Intended for local development, tests, and prototypes. |
| Native durable thingd | Available / early | Embedded thingd integration exists; recovery, backups, and workload validation remain deployment responsibilities. |
| HTTP thingd adapter | Available / partial | Adapter and contract tests exist; validate against the current public thingd service and failure policy. |
| Typed tools and manifests | Contract / partial | Types, registries, schemas, permissions, and manifests exist; broader discovery parity is still evolving. |
| Durable jobs and workers | Available / partial | Queue semantics, retries, leases, idempotency, dead letters, shutdown hooks, structured job logging, and metrics exist; production workloads need failure testing. |
| Observability | Available / partial | Structured logging, correlation IDs, request metrics, percentiles, uptime, error rate, and status breakdowns exist. OpenTelemetry/Prometheus exporters are not included. |
| OpenAPI helpers | Available / partial | OpenAPI 3.0.3 generation, security schemes, schemas, and Swagger HTML generation exist; applications must wire and validate their public route document. |
| Module composition | Available / early | Explicit module registration, dependency ordering, lifecycle hooks, and health hooks exist; Arqen intentionally does not provide hidden automatic DI. |
| Testing utilities | Available / early | `TestApp`, mock auth, fixtures, request builders, response readers, and assertion macros exist. |
| Cloud adapter | Future | Depends on a public thingd-cloud customer contract. |
| CLI `new`, `dev`, `start`, `check`, `doctor` | Available / early | Commands live in the `arqen` package; `dev` does not include an integrated watcher. |
| Node.js support | Future direction | Planned through HTTP, SDKs, templates, and manifests; no Node package is promised yet. |
| GitHub Pages docs | Available | Built and deployed from `main`; content follows the active repository documentation. |

Do not infer completion from a roadmap heading. Check the implementation,
tests, and phase acceptance evidence for the feature you need.
