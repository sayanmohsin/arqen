# Roadmap

## Phase 1: documentation and contracts (completed)

Define the architecture, configuration, startup output, logging, agent manifest, storage modes, tool metadata, job behavior, and deployment model.

## Phase 2: CLI and template (completed)

Implement `arqen new`, interactive and scripted project generation, `arqen
start`, `arqen dev`, health endpoints, startup output, structured logs, and a
generated public README.

## Phase 3: thingd development mode (completed)

Add in-memory thingd storage, repositories, events, search, links, queues, fixtures, and the HTTP thingd adapter.

## Phase 4: tools and workers (completed)

Add typed tools, manifests, authorization metadata, durable jobs, retries, idempotency, dead letters, and graceful worker shutdown.

## Phase 5: deployment (completed)

Add Docker generation, Compose, deployment guides, readiness checks, and `arqen doctor`.

## Phase 6: reference applications (completed)

Build small examples and a Watchloom-shaped reference backend without coupling Arqen to Watchloom.

## Phase 7: optional cloud adapter (blocked)

Integrate with a future public thingd-cloud customer API while preserving the direct thingd adapter.

## Current framework hardening (implemented, deployment validation ongoing)

The current core package and optional native adapter include layered configuration,
stable error contracts, authentication, request validation, health/readiness,
testing utilities, observability, OpenAPI helpers, and explicit module
composition. These capabilities are tested in the `arqen` package, but
production readiness still depends on the target application, thingd
deployment, security review, recovery testing, and operational controls.

## Phase 17: developer experience, performance, and agent onboarding (completed)

Stable CLI with exit codes, JSON output, config discovery, and integration tests. Compiling project generation. Criterion benchmarks for routing, manifest, validation, in-memory thingd, jobs, and health. Prettier and Markdownlint tooling. Comprehensive documentation for first-time developers and coding agents.

## Phase 18: application hardening and cloud boundary (in progress)

The October 2026 security review fixed JWT issuer validation. Phase 18 closes
gaps in existing foundations in four ordered milestones:

1. **18.1 — Isolation and scoped access:** verify `RequestContext` trust
   sources; cover cross-tenant, cross-instance, and cross-subject reads and
   mutations across all adapters; bound scoped search scans.
2. **18.2 — Safe mutations:** provide additive object CAS with a safe default
   for external backends; implement create-only version zero, matching writes,
   stale conflicts, and queue idempotency across memory, native, and HTTP.
3. **18.3 — Remote and production boundary:** fix confirmed HTTP error mapping
   and retry-safety gaps; reject unsafe production storage, auth, CORS, and
   logging combinations. JWKS rotation and tenant/instance discovery remain an
   optional lane gated on a versioned public customer API.
4. **18.4 — Scale and operations:** add Thingd keyset cursor pagination while
   retaining offsets; propagate its public contract through the Rust, native,
   REST, SDK, CLI/MCP, and Arqen layers; add process roles, bounded
   backpressure, exporter hooks, and backup/restore exercises.

Track each milestone as incomplete until its acceptance checks pass. The sync
protocol and replication engine remain owned by Thingd. Do not couple this
work to private Thingd Cloud internals or Watchloom domain types.

See [application-hardening.md](application-hardening.md) for scope and
boundary rules and [the security audit](security-audit-2026-10.md) for the
review evidence. This phase must not introduce Watchloom-specific domain types
or depend on private thingd-cloud modules.
