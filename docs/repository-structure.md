# Repository structure

The planned workspace is:

```text
arqen/
  crates/
    arqen-core/
    arqen-http/
    arqen-agent/
    arqen-thingd/
    arqen-jobs/
    arqen-logging/
  cli/arqen-cli/
  templates/
  examples/
  docs/
```

Keep crates composable. `arqen-core` should not depend on Axum or a model provider. `arqen-thingd` should own storage and queue adapters. The CLI and templates should be replaceable without changing application domain code.
