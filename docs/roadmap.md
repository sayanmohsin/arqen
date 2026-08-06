# Roadmap

## Phase 1: documentation and contracts (completed)

Define the architecture, configuration, startup output, logging, agent manifest, storage modes, tool metadata, job behavior, and deployment model.

## Phase 2: CLI and template (completed)

Implement `arqen new`, `arqen start`, `arqen dev`, health endpoints, startup output, structured logs, and a generated public README.

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

The current single-package framework also includes layered configuration,
stable error contracts, authentication, request validation, health/readiness,
testing utilities, observability, OpenAPI helpers, and explicit module
composition. These capabilities are tested in the `arqen` package, but
production readiness still depends on the target application, thingd
deployment, security review, recovery testing, and operational controls.

## Phase 17: developer experience, performance, and agent onboarding (completed)

Stable CLI with exit codes, JSON output, config discovery, and integration tests. Compiling project generation. Criterion benchmarks for routing, manifest, validation, in-memory thingd, jobs, and health. Prettier and Markdownlint tooling. Comprehensive documentation for first-time developers and coding agents.
