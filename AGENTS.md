# Arqen agent notes

Arqen is an early-stage backend framework for agent-ready applications. It
provides a single Cargo package containing the library and feature-gated CLI
binary. Production use still requires application-specific durability,
security, compatibility, and operational validation.

## Architecture

- **Single crate**: `crates/arqen` (published as `arqen` on crates.io)
- **Edition**: 2024, rust-version 1.96
- **Feature flags**: `default = ["http-server", "thingd-native"]`
- **CLI feature**: `cli = ["http-server", "logging"]`

## Boundaries

- Arqen: reusable Rust backend patterns, CLI, agent tools, jobs, and adapters.
- thingd: object storage, events, search, links, and queues.
- thingd-cloud: hosted identity, tenancy, provisioning, billing, and optional customer APIs.
- Watchloom: reference application only; do not couple Arqen to Watchloom domain types.

## Rules

- Prefer Axum, Tokio, Tower, tracing, and explicit application state.
- Do not create a NestJS-like dependency-injection framework.
- Keep provider and cloud credentials server-side.
- Treat the public thingd HTTP API as the first integration boundary.
- Preserve in-memory and durable adapter parity.
- Add docs and contract tests before implementation features.

## Production Hardening

### Config (B)
- Layered loading: CLI overrides → env vars → config file → defaults
- `CliOverrides`, `WorkerConfig`, `HealthConfig`, `Secret<T>`
- `from_file_optional()`, `apply_env()`, `apply_cli()`, `validate()`

### Error Contracts (C)
- 11 error kinds: Authentication, Authorization, Validation, NotFound, Conflict, RateLimit, Timeout, Dependency, Internal, NotImpl, Unavailable
- `ErrorResponse`, `CorrelationId`, `ErrorContext`
- `should_redact()`, `From<std::io::Error>`, `From<reqwest::Error>`

### Validation (D)
- `Validate` trait, `Validated<T>` extractor
- Enum validation: `one_of()`, `pattern()`
- Cross-field: `fields_match()`, `field_after()`
- Nested validation: `nested()`

### Auth (E)
- Real JWT: `jsonwebtoken` crate (HS256/RS256)
- Constant-time comparison: `subtle` crate
- SHA-256 API key hashing
- Policy combinators: `AllOf`, `AnyOf`, `RequireRole`

### Health (F)
- Parallel check execution
- Liveness/readiness probes
- `required_for_readiness()` trait method
- `to_http_status()` for HealthStatus

### Jobs (H)
- `JobMetrics`: processed, completed, failed, avg_duration_ms
- Structured logging with job_id, worker_id, queue
- Duration tracking per job

### Observability (I)
- Percentiles: p50, p95, p99, max
- Uptime tracking
- Error rate (5xx / total)
- by_status HashMap

### OpenAPI (J)
- Full OpenAPI 3.0.3 spec
- Security schemes (bearer, API key)
- `swagger_ui_html()` for bundled Swagger UI
- Tag support, request/response schemas

### Module Composition (K)
- `async_trait` for Module trait
- Lifecycle hooks: `init()`, `shutdown()`
- Dependencies: `dependencies()` method
- Health checks: `ModuleHealth` enum

### Testing (L)
- `TestApp`, `MockAuth`, `Fixtures`
- Request builders: `get()`, `post_json()`, `put_json()`, `delete()`
- Response readers: `read_body()`
- Macros: `assert_response!`, `assert_error!`, `assert_json_contains!`

## Consumer APIs

Arqen provides convenience APIs for application authors:

- **Router composition**: `create_router_with_state_and_routes(state, app_routes)` and `nest_routes(state, prefix, app_routes)` for merging application routes with built-in routes.
- **Auth middleware**: `auth_middleware` (enforcing) and `optional_auth_middleware` (passthrough) with `Authenticated` extractor for handlers.
- **Health wiring**: Register `HealthCheck` implementations via `ModuleContext` in `Module::register()`. Checks run automatically on `/health` and `/ready`.
- **OpenAPI CRUD**: `OpenApiGenerator` supports `add_get`, `add_post`, `add_put`, `add_delete`, `add_patch`.
- **Module composition**: `Module` trait with `ModuleBuilder` for dependency-ordered registration, initialization, shutdown, and health checks. `HttpModule` for HTTP route modules.
- **Schema generation**: `SchemaGenerator` is a placeholder. Tool input/output schemas must be hand-written as `serde_json::Value`.

## Watchloom integration

Watchloom is the first application built on Arqen. Its backend lives in
`watchloom/backend/` and uses the single `arqen` crate with features
`http-server`, `logging`, `http-client`, and `test-util`.

The Watchloom backend exercises these Arqen APIs:
- `AppState` / `AppStateBuilder` for service wiring
- `ThingdBackend` trait via `MemoryThingdBackend` (dev) and `HttpThingdBackend` (prod)
- `ToolRegistry` / `register_tool!` for agent manifest
- `JobHandler` / `JobWorker` / `Worker` for background jobs
- `AppError` / `ErrorKind` for structured errors
- `Validate` / `Validated<T>` for request validation
- `Authentication` / `AuthContext` for auth (with custom middleware)
- `HealthCheck` / `HealthRegistry` for dependency checks

Watchloom does not add domain types to Arqen. It owns its domain models,
storage service, provider traits, routes, and jobs.

## Dependencies

- `jsonwebtoken` (JWT validation)
- `sha2` (API key hashing)
- `subtle` (constant-time comparison)
- `clap` (CLI argument parsing)
- `anyhow` (error handling)
- `base64` (encoding)
- `hex` (hex encoding)

## Testing

- The current suite contains 186 library/binary tests plus 8 contract tests.
- `cargo test -p arqen --all-features` runs the package tests.
- `cargo clippy -p arqen --all-targets --all-features -- -D warnings` is the lint gate.
- `cargo run -p arqen --features cli --bin arqen -- --help` runs the CLI.
