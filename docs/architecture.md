# Architecture

<MermaidDiagram type="architecture" />

Arqen separates the application code you write from the infrastructure it
uses to serve requests, run jobs, and persist data.

## Request flow

1. A person, program, or agent calls an HTTP route or discovers a tool.
2. Arqen applies authentication, authorization, validation, and request
   context.
3. The application module runs its domain service.
4. The service reads or writes through a `ThingdBackend`, or enqueues durable
   work.
5. Health checks, logs, metrics, and job state make the result observable.

Application code owns domain models, business rules, and route handlers. Arqen
provides the application state, module lifecycle, HTTP helpers, storage
contracts, worker runtime, and operational checks.

## HTTP integration

The HTTP integration uses Axum, Tokio, and Tower. Applications can use Arqen’s
router, middleware, state, and lifecycle helpers, or use the re-exported HTTP
types when they need lower-level control.

The common starting point is `arqen::http::{Router, routing}` with Arqen’s
server helpers. See [Getting started](./getting-started.md) for a runnable
project and [OpenAPI](./openapi.md) for route documentation.

## Storage paths

The `ThingdBackend` contract keeps application services independent of the
selected storage mode:

- `MemoryThingdBackend` is for tests and disposable development;
- native Thingd is embedded in the application process;
- `HttpThingdBackend` connects to a separate Thingd service;
- a cloud adapter is future work and requires a public customer contract.

See [Configuration](./configuration.md) and
[Thingd integration](./thingd-integration.md).

## Modules and package structure

Modules group application features and register their routes, tools, jobs, and
health checks. Dependencies and lifecycle order are explicit.

```text
crates/arqen/src/
  core/             # Core types and errors
  http/             # HTTP server, middleware, and routes
  agent/            # Tool definitions and manifests
  auth/             # Authentication adapters and policies
  thingd/           # Memory, native, HTTP, scoped, and cache adapters
  jobs/             # Durable job handlers and workers
  logging/          # Tracing and redaction
  config.rs         # Layered configuration
  health.rs         # Health and readiness
  module.rs         # Module composition
  observability.rs  # Metrics and percentiles
  openapi.rs        # OpenAPI generation helpers
  state.rs          # Explicit application state
  testutil.rs       # Test helpers
```

The public library and feature-gated CLI are published as one Cargo package.
Generated application code is replaceable; your domain services do not need to
depend on generated implementation details.

## Ownership rules

- The application owns domain behavior, authorization policy, tenant/user
  ownership, secrets, backups, and deployment decisions.
- Arqen owns reusable composition, validation, workers, health, metrics, and
  adapter behavior.
- Thingd owns durable data primitives, replication semantics, tombstones, and
  conflict handling.
