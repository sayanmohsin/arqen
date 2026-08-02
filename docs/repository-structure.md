# Repository structure

The planned workspace is:

```text
arqen/
  crates/
    arqen/         # Public facade crate for most applications
      src/core/     # Core types and errors
      src/http/     # Axum HTTP server and routes
      src/agent/    # Tools and manifest generation
      src/thingd/   # thingd adapters
      src/jobs/     # Durable job workers
      src/logging/  # Tracing and redaction
  cli/arqen-cli/
  templates/
  examples/
  docs/
```

Keep the modules composable inside the public `arqen` crate. Core types stay
independent of Axum and model providers; the thingd module owns storage and
queue adapters. The CLI and templates should be replaceable without changing
application domain code.
