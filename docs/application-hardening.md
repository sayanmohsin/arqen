# Application hardening priorities

This document records reusable Arqen improvements for applications that use
local durable thingd during development and hosted thingd/thingd-cloud in
shared or production environments. Application-specific domain models and
business rules remain outside Arqen.

## Priority 0: isolation and safe mutation

### Typed request context

Expose a first-class request context containing:

- authenticated subject;
- tenant and thingd instance identifiers;
- scopes/roles;
- correlation ID;
- request origin or client metadata where appropriate.

The context should be available to handlers, repositories, jobs, and logs.
Tenant and instance values should be resolved from verified authentication or
trusted server configuration, never from an untrusted request body.

### Scoped storage helpers

The low-level `ThingdBackend` should remain generic. Add reusable scoped
helpers or repository decorators that make the scope explicit for reads,
writes, deletes, queries, and links. A caller should be able to construct a
store for a tenant/instance or user and avoid repeating string key/filter
conventions throughout an application.

Required tests should prove that two subjects and two tenants cannot read or
overwrite one another's objects, events, links, or jobs.

### Optimistic concurrency

Add revision or ETag-aware writes, for example `put_if_version`, with a stable
conflict error when the expected revision is stale. This is needed for mobile
edits to library state, ratings, preferences, and progress, and is a
prerequisite for safe local/cloud synchronization.

### Durable idempotency

Provide a reusable idempotency-key boundary for HTTP mutations and jobs. A
retry with the same key should return or reference the original result rather
than applying the mutation twice. The stored result must be scoped by tenant,
subject, route/action, and key, with an explicit retention policy.

## Priority 1: cloud and synchronization boundary

### JWKS and cloud identity

Add rotating JWKS verification, issuer/audience validation, tenant
resolution, and instance discovery as a documented optional cloud adapter.
Do not couple Arqen to thingd-cloud private modules or control-plane
databases. The adapter must target a versioned public customer API.

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

The HTTP thingd adapter should provide:

- explicit connect and request timeouts;
- retry/backoff only for safe or idempotent operations;
- typed public error-envelope parsing;
- API version negotiation or a pinned contract version;
- tenant/instance authentication headers;
- contract tests against the target thingd service;
- observable request latency, retries, and failures.

## Priority 1: production runtime

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

Production validation should reject unsafe combinations such as memory
storage, development authentication, permissive CORS, missing cloud/thingd
credentials, or non-structured logging when the deployment profile requires
structured logs. Validation should remain configurable for local development.

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
