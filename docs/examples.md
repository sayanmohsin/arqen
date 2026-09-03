# Examples

How to run existing examples and code snippets for common patterns.

## Running existing examples

### memory-backend

A minimal application using the in-memory storage adapter:

```bash
cargo run --example memory-backend
```

Source: [`examples/memory-backend/`](https://github.com/sayanmohsin/arqen/tree/main/examples/memory-backend)

### minimal-api

A bare-minimum HTTP API with health and agent endpoints:

```bash
cargo run --example minimal-api
```

Source: [`examples/minimal-api/`](https://github.com/sayanmohsin/arqen/tree/main/examples/minimal-api)

## Code snippets

### Module

```rust
use arqen::module::{Module, ModuleContext, ModuleHealth};
use arqen::core::AppError;

pub struct UsersModule;

#[arqen::async_trait]
impl Module for UsersModule {
    fn name(&self) -> &str {
        "users"
    }

    fn register(&self, ctx: &mut ModuleContext<'_>) -> Result<(), AppError> {
        // Register tools here
        Ok(())
    }

    async fn health_check(&self) -> ModuleHealth {
        ModuleHealth::Healthy
    }
}
```

See: [`crates/arqen/src/module.rs`](https://github.com/sayanmohsin/arqen/blob/main/crates/arqen/src/module.rs)

### Tool

```rust
use arqen::agent::{ToolEffect, ToolMetadata};

pub fn tool_metadata() -> ToolMetadata {
    ToolMetadata {
        name: "get_user".to_string(),
        description: "Get a user by ID".to_string(),
        input: serde_json::json!({
            "type": "object",
            "properties": {
                "user_id": { "type": "string" }
            },
            "required": ["user_id"]
        }),
        output: serde_json::json!({
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "name": { "type": "string" }
            }
        }),
        scopes: vec!["read:users".to_string()],
        effect: ToolEffect::Read,
        idempotent: true,
        enqueues_job: None,
        timeout: None,
    }
}
```

See: [`typed-tools.md`](./typed-tools.md)

### Job handler

```rust
use arqen::core::AppError;
use arqen::jobs::JobHandler;

pub struct SendEmailHandler;

#[arqen::async_trait]
impl JobHandler for SendEmailHandler {
    async fn handle(&self, payload: serde_json::Value) -> Result<(), AppError> {
        tracing::info!(payload = %payload, "Processing email job");
        Ok(())
    }
}
```

See: [`durable-jobs.md`](./durable-jobs.md)

### Weekly OTT release refresh

Register a schedule during application startup and register the
`ott_release_refresh` worker separately. The scheduler only enqueues the
durable job; the worker performs the importer work.

```rust
let scheduler = arqen::Scheduler::new(state.storage.clone());
scheduler
    .schedule(
        "ott-release-refresh",
        arqen::ScheduleOptions {
            expression: Some("0 0 * * 0".into()),
            queue: "imports".into(),
            job_type: "ott_release_refresh".into(),
            payload: serde_json::json!({
                "countries": ["CA", "US"],
                "release_window": "weekly",
                "requested_horizon": 7,
            }),
            ..arqen::ScheduleOptions::new("ott_release_refresh")
        },
    )
    .await?;
scheduler.start().await?;
```

The worker receives `schedule_id`, `scheduled_run_at`, `run_timestamp`, and a
deterministic `idempotency_key` alongside the application payload.

### Auth middleware

```rust
use arqen::auth::Authenticated;
use arqen::http::{body::Body, extract::Extension, http::Request, middleware::Next, response::Response};

pub async fn my_auth_layer(
    Extension(auth): Extension<Authenticated>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // auth.subject contains the authenticated identity
    next.run(req).await
}
```

See: [`authentication.md`](./authentication.md)

### Validation

```rust
use arqen::validation::Validate;

#[derive(serde::Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

impl Validate for CreateUser {
    fn validate(&self) -> Result<(), arqen::core::AppError> {
        if self.name.is_empty() {
            return Err(arqen::core::AppError::new(
                arqen::core::ErrorKind::Validation,
                "name must not be empty",
            ));
        }
        Ok(())
    }
}
```

See: [`validation.md`](./validation.md)

### Health check

```rust
use arqen::health::{HealthCheck, HealthStatus};
use std::time::Duration;

struct DatabaseCheck {
    url: String,
}

#[arqen::async_trait]
impl HealthCheck for DatabaseCheck {
    fn name(&self) -> &str { "database" }
    async fn check(&self) -> HealthStatus {
        // Check database connectivity
        HealthStatus::Healthy
    }
    fn timeout(&self) -> Duration { Duration::from_secs(3) }
    fn required_for_readiness(&self) -> bool { true }
}
```

See: [`health.md`](./health.md)

### OpenAPI

```rust
use arqen::openapi::OpenApiGenerator;

let mut gen = OpenApiGenerator::new("My API", "1.0.0");
gen.add_get("/users", "List users", "users");
gen.add_post("/users", "Create user", "users");
let spec = gen.generate();
```

See: [`openapi.md`](./openapi.md)

### Testing

```rust
use arqen::testutil::{TestApp, MockAuth};

let app = TestApp::new();
let resp = app.get("/health").await;
assert!(resp.status().is_success());
```

See: [`testing.md`](./testing.md)

### Storage modes

```rust
// Memory mode (default, no external deps)
arqen::AppState::builder()
    .with_storage_mode("memory")
    .build()

// Persistent mode (native thingd)
arqen::AppState::builder()
    .with_storage_mode("persistent")
    .build()

// HTTP mode (thingd sidecar)
arqen::AppState::builder()
    .with_storage_mode("http")
    .build()
```

See: [`in-memory-mode.md`](./in-memory-mode.md)
