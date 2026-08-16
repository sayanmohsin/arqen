---
title: Feature status
description: Current Arqen features, readiness levels, and application responsibilities.
---

# Feature status

Use this page to decide which Arqen features are ready for your project.

- **Available:** implemented and covered by repository tests.
- **Available / partial:** the API exists, but production policy or validation
  belongs to the application.
- **Experimental:** opt-in and requires testing against the exact service and
  workload you deploy.
- **Future / blocked:** not available in the current package.

<ProjectStatus />

## Application foundation

| Capability                         | Status              | Included                                                                                                                                         |
| ---------------------------------- | ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| HTTP server and router composition | Available           | Axum-based routes, route merging, request limits, timeouts, CORS, and correlation IDs.                                                           |
| Layered configuration              | Available / partial | CLI → environment → config file → defaults, typed validation, secret redaction, and production checks.                                           |
| Authentication and authorization   | Available / partial | JWT, API keys, session adapters, constant-time checks, hashing, and policy combinators. JWKS rotation and key lifecycle remain application work. |
| Request validation                 | Available / partial | Field, enum, pattern, cross-field, and nested validation through `Validate` and `Validated<T>`.                                                  |
| Error contracts                    | Available           | Stable error kinds, redacted responses, correlation IDs, timeout/dependency mappings, and HTTP status conversion.                                |
| Request context and scoping        | Available / early   | Subject, tenant, instance, scopes, roles, and correlation ID. Ownership and isolation rules remain application work.                             |
| Health and readiness               | Available / partial | Parallel checks, liveness/readiness probes, timeouts, startup grace configuration, and HTTP mappings.                                            |

## Tools, jobs, and operations

| Capability                | Status              | Included                                                                                                                                                                                      |
| ------------------------- | ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Typed tools and execution | Contract / partial  | Tool registry, metadata, permissions, invocation, structured inputs/outputs, and inline or queued execution.                                                                                  |
| Manifests and discovery   | Contract / partial  | Runtime metadata, tools, jobs, scopes, effects, idempotency behavior, and machine-readable discovery endpoints.                                                                               |
| Durable jobs              | Available / partial | Thingd-backed queues, leases, retries, idempotency metadata, dead letters, worker shutdown, logs, and metrics.                                                                                |
| Durable scheduler         | Available / partial | Persistent interval, cron, and one-time schedules that enqueue deterministic Thingd jobs; native mode is durable, HTTP mode is limited by the public queue contract, and memory is test-only. |
| Module composition        | Available           | Explicit registration, lifecycle hooks, dependency validation, initialization order, health checks, and HTTP module composition.                                                              |
| Observability             | Available / partial | Structured JSON/pretty logs, bounded request/job/storage/cache metrics, correlation IDs, percentiles, uptime, timeout/dependency counters, and sync metrics. Exporters are not bundled.       |
| OpenAPI                   | Available / partial | OpenAPI 3.0.3 generation, security schemes, CRUD helpers, schemas, tags, and Swagger UI HTML.                                                                                                 |
| Testing utilities         | Available / early   | `TestApp`, mock auth, fixtures, request builders, response readers, assertion macros, adapter contract tests, and benchmarks.                                                                 |
| CLI workflow              | Available / partial | `new`, `dev`, `start`, `up`, `check`, `doctor`, `generate`, `lint`, `format`, `test`, `build`, `doc`, and `thingd`.                                                                           |

## Data and Thingd

| Capability                  | Status                | Included                                                                                                                                                                              |
| --------------------------- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Memory backend              | Available             | Objects, events, search, links, and queues for development and tests.                                                                                                                 |
| Native durable Thingd       | Available / early     | Optional `thingd-native` feature, currently pinned to Thingd 0.83.2. Recovery, backups, resource sizing, and workload testing remain deployment work.                                 |
| HTTP catalog cache          | Available / partial   | Explicit collection-allowlisted cache for safe catalog reads; user-scoped caching remains application-owned.                                                                          |
| Startup bootstrap           | Available / partial   | Bounded retry helper and `arqen thingd seed`; applications still own seed contents and invocation timing.                                                                             |
| HTTP performance controls   | Available / partial   | Compression controls, opt-in ETags/304, JSONL streaming, bounded HTTP query scans, pooled clients, and metrics.                                                                       |
| HTTP Thingd adapter         | Available / partial   | Public `/v1` adapter for objects, events, search, links, queues, and batch operations, with an explicit health/API compatibility probe. Validate failure policy against your service. |
| Shared query/search filters | Available             | Numeric and RFC3339 comparisons, `Contains`, invalid-value errors, filtering before pagination, and consistent memory/native/HTTP semantics.                                          |
| Thingd schema inspection    | Available / partial   | Local `.thingd` loading and hashing plus remote schema and migration-history inspection. Applying migrations is operator-controlled.                                                  |
| Thingd replication          | Experimental / opt-in | Cursor checkpoints, bounded retries, collection allowlists, idempotent replay, conflict reporting, metrics, and stale-cursor snapshot fallback.                                       |
| Native-to-HTTP migration    | Available / early     | Checked, dry-run, resumable JSONL migration for objects, events, queues, indexes, and optional replication records. The source is preserved and the destination must be empty.        |
| Cloud adapter               | Future / blocked      | Requires a versioned public customer contract for identity, tenancy, routing, jobs, and compatibility.                                                                                |

See [Build a backend](./build-a-backend.md) for the recommended sequence and
[Application hardening](./application-hardening.md) for multi-user production
requirements.
