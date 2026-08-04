# Acceptance Criteria

1. `Module` trait supports lifecycle hooks: `init()`, `shutdown()`, `register()`, `health_check()`
2. `ModuleBuilder` validates module graph: detects duplicates, missing dependencies, and cycles
3. `ModuleBuilder` provides topological ordering for dependency-resolved initialization
4. `ModuleContext` enables explicit tool and health registration
5. `ModuleError` provides structured errors for graph and registration failures
6. `HttpModule` trait (feature-gated) enables HTTP route composition via Axum
7. `merge_module_routes()` combines multiple module routers into a base router
8. `ArqenApp` builder pattern composes modules, config, and state
9. `ArqenApp::start()` runs full lifecycle: init → server → shutdown
10. Shutdown is best-effort: all modules attempted, errors logged, server error preserved
11. `AppStateBuilder::with_modules()` validates and registers modules at state construction time
12. CLI generates module-based applications with `arqen new` and `arqen generate module`
13. No hidden dependency injection — all wiring is explicit via AppState and ModuleContext
14. No automatic handler discovery — routes are explicitly composed via HttpModule
15. No framework coupling — Module trait is framework-neutral, HttpModule is feature-gated
