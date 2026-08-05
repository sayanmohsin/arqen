# Phase 16 Scope: Module Composition

## What We Are Building

A module composition system for Arqen that lets developers organize an
application into discrete units with explicit dependencies, lifecycle hooks,
and health checks. Modules register tools and health checks into shared
registries. When the `http-server` feature is enabled, HTTP modules contribute
routes. The dependency graph is validated at startup and composition follows a
deterministic topological order.

## Components

### Module Trait

Async-capable trait (`#[async_trait]`, `Send + Sync`) with the following
lifecycle hooks:

- `name()` — unique identifier for the module
- `dependencies()` — list of module names this module depends on
- `register()` — receives `ModuleContext` to register tools and health checks
- `init()` — called once after graph validation, in topological order
- `shutdown()` — called once on application stop, in reverse topological order
- `health_check()` — returns `ModuleHealth` (Healthy, Degraded, Unhealthy)
- `routes()` — returns a list of route descriptors for dependency tracking

### ModuleBuilder

Collects modules, validates the dependency graph, and orchestrates lifecycle:

- Detects duplicate module names
- Detects missing dependencies
- Detects dependency cycles via DFS
- Produces a topological ordering of modules
- Executes `register_all()` to populate tool and health registries
- Executes `init_all()` in topological order
- Executes `shutdown_all()` in reverse topological order

### ModuleContext

A context object passed to `Module::register()` containing mutable borrows of:

- `&mut ToolRegistry` — for registering agent tools
- `&mut HealthRegistry` — for registering health checks

### Error Types

- `ModuleGraphError` — `DuplicateModule`, `MissingDependency`, `DependencyCycle`
- `ModuleError` — `Graph(ModuleGraphError)`, `Registration { module, message }`

### ModuleHealth

Enum representing module health status:

- `Healthy`
- `Degraded { reason: String }`
- `Unhealthy { reason: String }`

### HttpModule Trait

Feature-gated on `http-server`. Extends or complements `Module` for modules
that contribute HTTP routes:

- `fn router(&self) -> Router<AppState>` — returns an Axum router

### merge_module_routes()

Collects routers from all `HttpModule` implementors and merges them into a
single `Router` for mounting on the application.

### ArqenApp / ArqenAppBuilder

Builder pattern wrapping `ModuleBuilder` for full application lifecycle:

- Stores the module graph
- `async init()` — validates graph, runs `register_all()`, runs `init_all()`
- Starts the Axum server (when `http-server` feature is enabled)
- `async shutdown()` — runs `shutdown_all()` in reverse order

### AppStateBuilder::with_modules()

Integration point with the existing `AppStateBuilder`:

- Accepts a `ModuleBuilder`
- Validates the module graph
- Registers tools and health checks from all modules
- Returns `Result<Self, ModuleError>`

### CLI Generators

Command-line scaffolding for new modules, tools, and jobs:

- `arqen new` — scaffold a new Arqen application
- `arqen generate module` — generate a module skeleton
- `arqen generate tool` — generate a tool skeleton
- `arqen generate job` — generate a job handler skeleton

## What We Are Not Building

- **Dependency injection container** — no service locator, no automatic
  resolution. Modules receive explicit context, not injected services.
- **Automatic handler discovery** — no annotation scanning, no macro-based
  registration. All wiring is explicit code.
- **Framework coupling** — Axum is used only when the `http-server` feature is
  enabled. The core module system has no framework dependency.
- **Hot-reloading** — modules are composed once at startup.
- **Inter-module messaging** — no event bus, no message passing between modules.
- **Version compatibility** — modules are compiled together; no runtime version
  negotiation.

## Composition Rules

1. Every module must have a unique name.
2. A module's dependencies must exist in the graph.
3. The dependency graph must be acyclic.
4. `register()` is called before `init()`.
5. `init()` follows topological order.
6. `shutdown()` follows reverse topological order.
7. HTTP routes are merged only when the `http-server` feature is active.

## Testing Strategy

- Unit tests for graph validation (duplicates, missing deps, cycles)
- Unit tests for topological ordering (linear, diamond, complex graphs)
- Unit tests for lifecycle ordering (init order, shutdown reverse order)
- Unit tests for `ModuleContext` tool and health registration
- Unit tests for `ModuleHealth` variant construction
- Integration tests for `merge_module_routes()`
- Integration tests for `AppStateBuilder::with_modules()`
- CLI generator output compilation tests

## Success Criteria

- Cycles, missing deps, and duplicate names produce clear errors at build time
- Lifecycle hooks execute in the correct order
- `ModuleContext` allows tools and health checks to be registered
- `HttpModule` routes merge correctly
- `AppStateBuilder::with_modules()` returns a fully wired `AppState`
- CLI generators produce valid, compilable Rust code
- All tests pass
- All lints pass
