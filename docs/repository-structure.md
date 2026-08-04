# Repository structure

The workspace contains one Cargo package:

```text
arqen/
  crates/
    arqen/         # Single published package: library plus CLI binary
      src/bin/arqen.rs
      src/core/     # Core types and errors
      src/http/     # Axum HTTP server and routes
      src/agent/    # Tools and manifest generation
      src/auth/     # Authentication adapters and policies
      src/thingd/   # thingd adapters
      src/jobs/     # Durable job workers
      src/logging/  # Tracing and redaction
      src/config.rs
      src/health.rs
      src/module.rs
      src/observability.rs
      src/openapi.rs
      src/state.rs
      src/testutil.rs
  # `arqen new` writes the starter structure directly; there is no checked-in
  # template directory.
  examples/
  docs/
```

Keep the modules composable inside the public `arqen` crate. Core types stay
independent of Axum and model providers; the thingd module owns storage and
queue adapters. The CLI is enabled with the `cli` feature and is not a second
published package. Templates should be replaceable without changing
application domain code. Generated scaffolding is replaceable without changing
application domain code.
