# Phase 16: Module Composition

## Objective

Provide a lightweight module composition system for Arqen that lets developers
split an application into logical units with explicit dependencies, lifecycle
hooks, and health checks. Modules register tools and health checks into shared
registries. HTTP modules contribute routes when the `http-server` feature is
enabled. The system enforces a valid dependency graph at startup and composes
the application in deterministic order.

## Dependencies

- Axum (feature-gated behind `http-server`)
- `async_trait` for the async-capable `Module` trait
- `ToolRegistry` and `HealthRegistry` from Arqen core
- `AppState` as the composition root

## In Scope

| Component | Description |
|---|---|
| `Module` trait | `name()`, `routes()`, `dependencies()`, `register()`, `init()`, `shutdown()`, `health_check()`. Uses `#[async_trait]`, requires `Send + Sync`. |
| `ModuleBuilder` | Collects modules, validates the dependency graph (duplicates, missing deps, cycles via DFS), produces topological ordering, and executes `register_all()`, `init_all()`, `shutdown_all()`. |
| `ModuleContext<'a>` | Mutable borrow of `ToolRegistry` and `HealthRegistry` passed to `Module::register()`. |
| `ModuleHealth` | Enum: `Healthy`, `Degraded { reason }`, `Unhealthy { reason }`. |
| `ModuleGraphError` | `DuplicateModule`, `MissingDependency`, `DependencyCycle`. |
| `ModuleError` | `Graph(ModuleGraphError)`, `Registration { module, message }`. |
| `HttpModule` trait | Feature-gated on `http-server`. Adds `fn router(&self) -> Router<AppState>` for HTTP route modules. |
| `merge_module_routes()` | Collects routers from all `HttpModule` implementors and merges them into a single `Router`. |
| `ArqenApp` / `ArqenAppBuilder` | Builder pattern wrapping `ModuleBuilder`. Async lifecycle: `init()` → server start → `shutdown()`. |
| `AppStateBuilder::with_modules()` | Validates the module graph, registers tools and health checks, returns `Result<Self, ModuleError>`. |
| CLI generators | `arqen new`, `arqen generate module`, `arqen generate tool`, `arqen generate job`. |

## Out of Scope

- Dependency-injection container or service locator
- Automatic handler discovery or annotation-based routing
- Framework-specific coupling (Axum is feature-gated; no hard dependency)
- Hot-reloading or dynamic module loading at runtime
- Inter-module message passing or event bus
- Module versioning or compatibility checking

## Deliverables

1. `Module` trait with full lifecycle hooks
2. `ModuleBuilder` with graph validation and topological sort
3. `ModuleContext` for tool and health registration
4. `ModuleError` / `ModuleGraphError` error types
5. `ModuleHealth` enum
6. `HttpModule` trait (feature-gated)
7. `merge_module_routes()` function
8. `ArqenApp` / `ArqenAppBuilder` with async lifecycle
9. `AppStateBuilder::with_modules()` integration
10. CLI generators for modules, tools, and jobs
11. Unit tests covering graph validation, cycle detection, lifecycle ordering
12. Integration tests for end-to-end module composition

## Acceptance Criteria

- `Module::init()` is called in topological order; `shutdown()` in reverse
- Cycles, missing dependencies, and duplicate names are caught at build time with clear errors
- `register()` receives a valid `ModuleContext` and can register tools and health checks
- `HttpModule` routes merge correctly when the `http-server` feature is enabled
- `AppStateBuilder::with_modules()` returns a properly wired `AppState` or a descriptive `ModuleError`
- CLI generators produce compilable boilerplate
- All tests pass with `cargo test -p arqen --all-features`
- Lints pass with `cargo clippy -p arqen --all-targets --all-features -- -D warnings`

## Tests

- Graph validation: duplicate module names, missing dependencies, single and multi-node cycles
- Topological ordering correctness for diamond and linear dependency graphs
- Lifecycle ordering: init order matches topological sort, shutdown is reverse
- `ModuleContext` tool and health registration
- `ModuleHealth` variant construction
- `merge_module_routes()` combines routers from multiple `HttpModule` implementors
- `AppStateBuilder::with_modules()` integration with full wiring
- CLI generator output compilation

## Documentation

- Module composition guide with examples for creating, registering, and composing modules
- `Module` trait API reference
- `HttpModule` trait API reference
- `ArqenApp` builder usage
- CLI generator reference
- Migration notes for applications adopting the module system

## Handoff

The module composition system is ready for application authors once:
- Tests and lints pass
- CLI generators produce valid code
- `AppStateBuilder::with_modules()` wires everything end to end
- Documentation covers the full workflow from module creation to application startup
