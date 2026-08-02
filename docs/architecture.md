# Architecture

```text
Client or agent
      |
      v
Arqen HTTP API (Axum)
      |
      +-- domain services
      +-- typed agent tools
      +-- auth and policy
      +-- job workers
      +-- observability
      |
      v
Arqen thingd adapter
      |
      +-- MemoryThingdBackend for development
      +-- HttpThingdBackend for durable deployment
      +-- CloudThingdBackend (optional, future)
```

Application code depends on domain repositories and job interfaces. It should not depend directly on HTTP clients, provider SDKs, or private cloud modules.

The first web stack is Axum with Tokio and Tower. Application state is explicit and constructed in `main`, avoiding a general-purpose dependency-injection container.

## Package structure

The public library is one Cargo package with composable internal modules:

```text
arqen/
  crates/
    arqen/            # Single public Cargo package
      src/core/       # Core types, traits, and errors
      src/http/       # Axum HTTP server, middleware, and routes
      src/agent/      # Agent tool definitions and manifest generation
      src/thingd/     # thingd adapters (memory, HTTP, cloud)
      src/jobs/       # Durable job types and worker runtime
      src/logging/    # Tracing setup and redaction
  cli/arqen-cli/      # CLI binary
  templates/          # Project templates
  examples/           # Example applications
  docs/               # Documentation
  specs/              # Phase specifications
```

## Dependency rules

- `arqen::core` must not depend on Axum, thingd, or model providers.
- `arqen::thingd` owns storage and queue adapters.
- `arqen::jobs` depends on core and thingd contracts.
- `arqen::http` depends on core and agent modules.
- The CLI and templates are replaceable without changing application domain code.
