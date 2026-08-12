# Testing applications

The `test-util` feature provides an in-process `TestApp`, request builders,
fixtures, `MockAuth`, response readers, and assertion macros. It lets an
application exercise routing and error contracts without binding a real port.

```toml
[dev-dependencies]
arqen = { version = "0.9", features = ["test-util"] }
```

Keep tests layered:

1. Unit-test domain services and validation without HTTP.
2. Use `TestApp` for routes, auth, JSON errors, health, and readiness.
3. Use adapter contract tests for memory, native durable, and HTTP thingd
   implementations.
4. Run a small real-process smoke test for deployment, signals, and external
   dependencies.

Framework test utilities do not replace application integration tests. Add
fixtures for your domain and verify that every public tool and route has the
permissions, idempotency, and audit behavior your application promises.
