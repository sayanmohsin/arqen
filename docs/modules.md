# Modules and application composition

Arqen uses explicit modules to make application wiring visible. A module owns
its routes, tools, jobs, health checks, and lifecycle hooks. It does not use a
hidden dependency-injection container or global service locator.

## The composition model

```text
ModuleBuilder
  ├─ dependencies()  → dependency graph
  ├─ register()      → routes, tools, jobs, checks
  ├─ init()          → startup lifecycle
  └─ shutdown()      → graceful teardown
```

Implement `Module` for a feature boundary and add it to `ModuleBuilder`.
Dependencies are declared by module name and initialized in dependency order.
`ModuleGraphError` and `ModuleError` report invalid graphs and registration
failures without hiding the cause.

```rust
use arqen::{Module, ModuleBuilder};

let builder = ModuleBuilder::new()
    .add_module(MyAccountsModule)
    .add_module(MyJobsModule);
```

`ArqenApp` runs registration, initialization, serving, and best-effort
shutdown. Use `ModuleContext` to register health checks and other public
application capabilities. Keep domain services in your application and pass
them through explicit `AppState` rather than relying on implicit resolution.

## When to use a module

Create a module when a feature has more than one boundary—such as routes plus
a job worker, or tools plus a dependency health check. A small application can
start with one module and split it later. `HttpModule` is available for route-
focused composition.

See [testing](testing.md) for composing modules in an in-process test app.
