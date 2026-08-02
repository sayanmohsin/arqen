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

## Crate structure

The workspace is organized into composable crates:

```text
arqen/
  crates/
    arqen-core/       # Core types, traits, and errors (no Axum dependency)
    arqen-http/       # Axum HTTP server, middleware, and routes
    arqen-agent/      # Agent tool definitions and manifest generation
    arqen-thingd/     # thingd adapters (memory, HTTP, cloud)
    arqen-jobs/       # Durable job types and worker runtime
    arqen-logging/    # Tracing setup and redaction
  cli/arqen-cli/      # CLI binary
  templates/          # Project templates
  examples/           # Example applications
  docs/               # Documentation
  specs/              # Phase specifications
```

## Dependency rules

- `arqen-core` must not depend on Axum, thingd, or model providers.
- `arqen-thingd` owns storage and queue adapters.
- `arqen-jobs` depends on `arqen-core` and optionally on `arqen-thingd`.
- `arqen-http` depends on `arqen-core` and `arqen-agent`.
- The CLI and templates are replaceable without changing application domain code.
