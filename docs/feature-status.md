---
title: Feature status
description: What Arqen provides today, what is partial, and what remains application-owned.
---

# Feature status

This page is the capability map. “Available” means the public API exists and
has repository tests. “Partial” means the framework supplies the boundary but
your application still owns important policy or production validation.
“Experimental” means opt-in and should be tested against the exact Thingd
service and workload you deploy.

<ProjectStatus />

## Application foundation

| Capability                         | Status              | What is included                                                                                                                                                        |
| ---------------------------------- | ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| HTTP server and router composition | Available           | Axum routes, application route merging, request limits, timeouts, CORS, correlation IDs, and response identity headers.                                                 |
| Layered configuration              | Available / partial | CLI overrides → environment → config file → defaults, typed validation, secret redaction, and production checks. Deployment-specific secrets and policy remain yours.   |
| Authentication and authorization   | Available / partial | JWT, API keys, session adapters, constant-time checks, hashing, and `AllOf`/`AnyOf`/role policies. JWKS rotation and key lifecycle remain application responsibilities. |
| Request validation                 | Available / partial | Field, enum, pattern, cross-field, and nested validation through `Validate` and `Validated<T>`. No derive macro is promised.                                            |
| Error contracts                    | Available           | Stable error kinds, redacted responses, correlation IDs, timeout/dependency mappings, and HTTP status conversion.                                                       |
| Typed request context and scoping  | Available / early   | Subject, tenant, instance, scopes, roles, and correlation ID are available. Tenant/user ownership rules must be enforced by the application.                            |
| Health and readiness               | Available / partial | Parallel checks, liveness/readiness probes, timeouts, startup grace configuration, and HTTP mappings. Register your real dependency checks.                             |

## Agents, jobs, and operations

| Capability                 | Status              | What is included                                                                                                                                                                                             |
| -------------------------- | ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Typed tools and execution  | Contract / partial  | Tool registry, typed metadata, permissions, invocation boundaries, structured inputs/outputs, and inline or queued execution. Discovery conventions are still evolving.                                      |
| Manifests and discovery    | Contract / partial  | Runtime metadata, tools, jobs, scopes, effects, idempotency behavior, and machine-readable discovery endpoints. Applications own their public capability policy.                                             |
| Durable jobs               | Available / partial | Thingd-backed queues, leases, retries, idempotency metadata, dead letters, worker shutdown, structured job logging, and metrics. Production failure testing and request idempotency remain application work. |
| Module composition         | Available           | Explicit registration, lifecycle hooks, dependency validation, topological initialization, health checks, and HTTP module composition—without hidden dependency injection.                                   |
| Observability              | Available / partial | Structured logging, request/job/storage/cache metrics, percentiles, uptime, error rates, status breakdowns, sync metrics, and redaction guidance. Prometheus/OpenTelemetry exporters are not bundled.        |
| OpenAPI                    | Available / partial | OpenAPI 3.0.3 generation, security schemes, CRUD operation helpers, schemas, tags, and Swagger UI HTML. Applications must wire and validate their final route document.                                      |
| Testing utilities          | Available / early   | `TestApp`, mock auth, fixtures, request builders, response readers, assertion macros, adapter contract tests, and benchmarks.                                                                                |
| CLI and developer workflow | Available / partial | `new`, `dev`, `start`, `up`, `check`, `doctor`, `generate`, `lint`, `format`, `test`, `build`, `doc`, and `thingd` commands.                                                                                 |

## Data and Thingd

| Capability                  | Status                | What is included                                                                                                                                                                                                                               |
| --------------------------- | --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Memory backend              | Available             | Objects, events, search, links, and queues for development, tests, and prototypes.                                                                                                                                                             |
| Native durable Thingd       | Available / early     | Embedded persistent Thingd with optional encryption and schema validation. Recovery, backups, resource sizing, and workload testing remain deployment responsibilities.                                                                        |
| HTTP Thingd adapter         | Available / partial   | Public `/v1` adapter for objects, events, search, links, queues, and batch operations. Validate compatibility, retries, timeouts, and failure policy against your service.                                                                     |
| Shared query/search filters | Available             | Numeric and RFC3339 comparisons, `Contains`, invalid-value errors, filtering before pagination, and consistent memory/native/HTTP semantics.                                                                                                   |
| Thingd schema inspection    | Available / partial   | Local `.thingd` loading/hashing plus remote current-schema and migration-history inspection. Applying migrations is deliberately operator-controlled.                                                                                          |
| Thingd replication          | Experimental / opt-in | Cursor checkpoints, bounded retries, collection allowlists, idempotent replay, conflict reporting, capability metrics, and stale-cursor snapshot fallback. Thingd owns provenance, tombstones, and conflict semantics.                         |
| Native-to-HTTP migration    | Available / early     | Checked, dry-run, resumable JSONL snapshot migration for objects, events, queues, indexes, and optional replication records. It preserves the source and refuses a non-empty destination. It does not promise a live-write-consistent cutover. |
| Cloud adapter               | Future / blocked      | Requires a versioned public customer contract for identity, tenancy, routing, jobs, and compatibility. Arqen does not read private cloud control-plane data.                                                                                   |

## Boundaries to understand

Arqen records and exposes the data needed by its adapters, jobs, metrics, and
replication workflows. It does not automatically create application audit
history, tenant policy, user ownership, backups, provider credentials, or a
live migration cutover. Those belong in the application and its operations
system. See [application hardening](./application-hardening.md) and the
[production runbook](./production-runbook.md).

Do not infer completion from a roadmap heading. Check the implementation,
tests, and the exact Thingd service version used by your deployment.
