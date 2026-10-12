# Application hardening priorities

This document records reusable Arqen improvements for applications that use
local durable thingd during development and hosted thingd/thingd-cloud in
shared or production environments. Application-specific domain models and
business rules remain outside Arqen.

Phase 18 is tracked as four ordered milestones. Request context, scoped
storage, JWT issuer/audience checks, production configuration validation,
HTTP timeouts/retries, and replication cursors already exist. Current work
closes gaps in those foundations: bounded scoped search, conditional writes,
and public queue idempotency. Remaining milestones are listed below; a source
change is not evidence of deployment readiness. See the
[security audit](./security-audit-2026-10.md) for evidence and residual risks.

## Phase 18 checkpoint

| Milestone                 | Status      | Remaining work                                                                                                                                                                            |
| ------------------------- | ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 18.1 Isolation            | In progress | Memory/native shared contracts pass. Run HTTP contracts against Thingd 0.92.3; add batch-failure and adversarial-scope-ID cases.                                                          |
| 18.2 Safe mutations       | In progress | Memory/native CAS contracts pass; HTTP CAS and keyed retry/replay await the live Thingd 0.92.3 contract run. Add concurrent-writer coverage.                                              |
| 18.3 Production boundary  | In progress | Production configuration rejection and HTTP mapping/retry unit tests pass; finish live request contracts and deployment-profile review. JWKS/discovery stays gated on a public Cloud API. |
| 18.4 Scale and operations | Planned     | Keyset pagination, process roles, bounded backpressure, exporter hooks, and recovery exercises.                                                                                           |

## Priority 0: isolation and safe mutation

### 18.1 — Isolation and scoped access

`RequestContext` and `ScopedThingdBackend` are implemented. Tenant and
instance values must continue to come from verified authentication or trusted
server configuration. Scoped search now caps backend scans at 10,000 objects
and returns an unavailable error above that ceiling rather than reading an
unbounded result set. Cross-tenant and cross-instance search coverage is part
of the scoped adapter suite. The shared contract runs for memory and native
adapters. HTTP integration contracts cover the same isolation areas but
require a Thingd 0.92.3 server; batch-failure and adversarial scope-identifier
cases remain open.

### Typed request context

Arqen exposes a first-class `RequestContext` containing:

- authenticated subject;
- tenant and thingd instance identifiers;
- scopes/roles;
- correlation ID.

The context is available to handlers and request logging; applications should
pass it explicitly into repositories and jobs rather than using ambient global
state.
Tenant and instance values should be resolved from verified authentication or
trusted server configuration, never from an untrusted request body.

### Scoped storage helpers

The low-level `ThingdBackend` remains generic. `ScopedThingdBackend` makes the
scope explicit for reads, writes, deletes, queries, and links. Shared
contracts cover object reads and queries, event reads, job claims and replay,
link reads, and batch object visibility across memory and native adapters.
HTTP integration coverage is available for a 0.92.3 server; batch-failure and
adversarial scope-identifier cases remain open.

Tests should continue to prove that two subjects and two tenants cannot read
or overwrite one another's objects, events, links, or jobs across adapters.

### 18.2 — Safe mutations

`ThingdBackend::get_object_versioned` and
`ThingdBackend::put_object_if_version` are additive and return `NotImpl` from
older external implementations until they opt in. Memory, native, HTTP, cache,
and scoped adapters implement versioned reads and conditional writes. The native and HTTP paths
use Thingd's existing create-only version-zero CAS contract; HTTP CAS requests
are sent once because a lost response followed by a retry can turn a committed
write into a misleading stale-version conflict. Queue idempotency is
propagated through the public Thingd HTTP adapter and server route. The Arqen
HTTP queue pushes now propagate the caller's stable `idempotencyKey` and
`delayMs` under Thingd 0.92.3. Unit tests verify stable retry payloads and that
an unkeyed failed push is attempted only once. The live contracts were aimed
at the Cloud MCP host with a project key, but `HttpThingdBackend` targets a
direct REST server under `/v1`; all three returned 401 before assertions. Run
them against a direct Thingd 0.92.3 REST endpoint using that server's
`THINGD_AUTH_TOKEN` (or `auth.token`).

### Optimistic concurrency

Conditional writes return a stable `Conflict` error when the expected version
is stale. Preserve this contract across adapters and cover create-only version
zero, matching updates, stale writes, and concurrent writers.

### Durable idempotency

Queue insertion uses a stable queue-scoped job ID for repeated idempotency
keys. Broader HTTP mutation idempotency still needs a public contract with
retention and result-replay semantics; do not imply that arbitrary mutations
are idempotent merely because the client retries them.

## 18.3 — Remote and production boundary

### JWKS and cloud identity

Keep rotating JWKS verification, tenant resolution, and instance discovery in
an optional lane that starts only after a versioned public Cloud customer API
exists. Issuer and audience checks already exist for configured JWTs. Do not
couple Arqen to thingd-cloud private modules or control-plane databases.

### Sync capability boundary

The sync protocol itself belongs to thingd. Arqen must not create a second
replication engine or duplicate thingd's conflict, checkpoint, tombstone, or
transport semantics. When thingd provides local-to-cloud synchronization,
Arqen should integrate it through a capability-neutral boundary that provides:

- configuration and secret loading;
- tenant/instance routing and authenticated credentials;
- startup and shutdown lifecycle management;
- readiness and health reporting;
- sync lag, checkpoint, retry, and failure metrics;
- capability/version discovery;
- application hooks for sync status and safe promotion.

The `ThingdBackend` should remain usable whether synchronization is disabled,
embedded in thingd, or provided by a remote thingd/cloud runtime. Any
sync-specific API should be owned and versioned by thingd, then exposed to
Arqen through a documented public contract.

### Cursor-based event consumption

Extend event reads with durable sequence cursors and a response containing
`next_cursor`, `has_more`, and a clear consistency position. Define replay,
retention, tombstone, and ordering behavior. This supports future sync,
activity feeds, projections, and reliable consumers.

### HTTP adapter hardening

The HTTP Thingd adapter provides connection pooling, explicit timeouts, typed
public error mapping, and retries limited to GET and keyed queue pushes.
Remaining work includes:

- API version negotiation or a pinned contract version;
- tenant/instance authentication headers;
- contract tests against the target thingd service;
- observable request latency, retries, and failures.

## 18.4 — Scale and operations

### Separate worker role

Support an explicit API/worker process split while retaining an embedded
worker option for development:

```text
arqen start --role api
arqen start --role worker
```

Worker health should expose queue lag, leases, retries, dead letters, handler
duration, and shutdown state. Worker IDs must be stable enough for diagnosis
but unique enough to avoid lease collisions.

### Production configuration guardrails

Production validation rejects memory storage, disabled or credentialless
authentication, short JWT HMAC secrets, wildcard CORS, pretty logs, and missing
native schemas. It remains configurable for local development. Further
credential and storage transport policy should be driven by the supported
deployment profiles.

### Pagination and backpressure

Add cursor pagination alongside offset pagination for large collections,
events, and activity feeds. Define maximum limits, request budgets, and
backpressure behavior for search, queues, and batch writes.

### Schema/version envelopes

Provide an optional application object envelope containing schema version,
revision, timestamps, and actor metadata. Applications should be able to
evolve JSON objects additively and detect incompatible versions without an ORM
or central migration framework.

## Priority 2: operations and developer experience

- Add Prometheus/OpenTelemetry exporter hooks without requiring a specific
  observability vendor.
- Expose queue lag, dependency latency, storage failures, and sync checkpoints
  through health and metrics APIs.
- Add multi-subject, multi-tenant isolation fixtures to `TestApp`.
- Add native/HTTP compatibility tests for restart recovery, timeouts, retries,
  conditional writes, event cursors, and dead-letter behavior.
- Add capability tests proving Arqen can detect and operate with thingd sync
  enabled or disabled without changing application domain services.
- Document backup, restore, schema evolution, and cloud promotion workflows.

## Boundary rule

These framework improvements must remain domain-neutral. Arqen should not gain
application-specific types such as titles, seasons, ratings, or availability
offers. Applications own those domain contracts and use Arqen for runtime,
auth, validation, jobs, health, and storage boundaries.
