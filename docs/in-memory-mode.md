# In-memory mode

In-memory mode is the default for generated applications:

```toml
[thingd]
mode = "memory"
```

It should provide object storage, events, search, links, and queues without requiring a separate service. This makes prototypes, examples, unit tests, and agent-generated changes fast to run.

In-memory data is process-local and disposable. The startup banner must make this explicit. Applications should be able to reset fixtures and seed deterministic test data.

Production deployments should use a durable thingd service or a deliberately selected embedded durable adapter.
