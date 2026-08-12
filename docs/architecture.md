# Architecture

<MermaidDiagram type="architecture" />

The request boundary stays stable while the storage adapter changes underneath
it. The hosted cloud path is shown as future because Arqen currently relies on
public Thingd contracts rather than private cloud modules.

Application code depends on domain repositories and job interfaces. It should not depend directly on HTTP clients, provider SDKs, or private cloud modules.

The first web transport is implemented with Axum, Tokio, and Tower. Applications use Arqen’s router, middleware, state, and lifecycle APIs; Axum remains an internal transport detail unless an application deliberately uses the lower-level HTTP integration.

Application code should prefer `arqen::http::{Router, routing}` and Arqen’s
server helpers. The underlying transport is re-exported only as a compatibility
facade, so application code does not need to name Axum directly.

## Package structure

The public library is one Cargo package with composable internal modules:

```text
arqen/
  crates/
    arqen/            # Single Cargo package, library and CLI
      src/bin/arqen.rs# Feature-gated CLI binary
      src/core/       # Core types, traits, and errors
      src/http/       # Arqen HTTP server, middleware, and routes
      src/agent/      # Agent tool definitions and manifest generation
      src/auth/       # Authentication adapters and policies
      src/thingd/     # thingd adapters (memory, native, HTTP)
      src/jobs/       # Durable job types and worker runtime
      src/logging/    # Tracing setup and redaction
      src/config.rs   # Layered typed configuration
      src/health.rs   # Health and readiness checks
      src/module.rs   # Explicit module composition
      src/openapi.rs  # OpenAPI generation helpers
      src/state.rs    # Explicit application state
      src/testutil.rs # Test application and request helpers
  # CLI-generated project scaffolding is defined in src/bin/arqen.rs
  examples/           # Example applications
  docs/               # Documentation
  specs/              # Phase specifications
```

## Dependency rules

- `arqen::core` must not depend on Axum, thingd, or model providers.
- `arqen::thingd` owns storage and queue adapters.
- `arqen::jobs` depends on core and thingd contracts.
- `arqen::http` depends on core and agent modules.
- The CLI is optional and feature-gated; `arqen new` and `arqen generate`
  create replaceable application scaffolding without changing domain code.
