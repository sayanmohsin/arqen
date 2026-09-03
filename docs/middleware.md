# Middleware and application hooks

Arqen provides a framework-owned request pipeline and application lifecycle.
Most applications only need Arqen's route helpers, middleware functions, and
modules; transport-specific integration remains an advanced escape hatch.

## Request middleware

The built-in pipeline provides:

- request IDs and correlation propagation;
- request and response logging;
- authentication and authorization;
- request timeouts and body limits;
- CORS, compression, and cache headers;
- request metrics and slow-request reporting;
- liveness and readiness endpoints.

Application routes can add a framework-owned hook with `ArqenApp`:

```rust
use arqen::{ArqenApp, MiddlewareHook};

struct AccessPolicy;

#[arqen::async_trait]
impl MiddlewareHook for AccessPolicy {
    fn name(&self) -> &str { "access-policy" }

    async fn before(
        &self,
        context: &arqen::MiddlewareContext,
        _state: &arqen::AppState,
    ) -> Result<(), arqen::AppError> {
        if context.path().starts_with("/admin") {
            return Err(arqen::http::middleware_hooks::reject("admin access required"));
        }
        Ok(())
    }
}

let app = ArqenApp::builder()
    .middleware_hook(AccessPolicy)
    .build()?;
```

Middleware runs before the handler on the request path and after the handler
on the response path. A middleware may return early, for example when
authentication fails. Keep security middleware outside application routes and
register hooks in the intended order. Hooks have state access through the
`MiddlewareContext` request identity and the application state is available to
handlers and modules.

Use `auth_middleware` for required authentication,
`optional_auth_middleware` for public routes that accept an authenticated
context, and `require_auth_middleware` for policy checks such as roles or
scopes.

## Lifecycle hooks

Use `Module` when a feature owns routes, tools, jobs, health checks, or more
than one lifecycle concern. Use `LifecycleHook` for a focused startup or
shutdown action:

```rust
use arqen::{ArqenApp, LifecycleHook};

struct CatalogWarmup;

#[arqen::async_trait]
impl LifecycleHook for CatalogWarmup {
    fn name(&self) -> &str { "catalog-warmup" }

    async fn startup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ArqenApp::builder()
        .hook(CatalogWarmup)
        .build()?
        .run()
}
```

Hooks initialize in dependency order and shut down in reverse order. Startup
failure prevents the server from accepting traffic. Shutdown is best-effort:
all registered hooks are attempted while the first server or shutdown error is
preserved.

## Runtime-neutral boundary

Application code should use Arqen's application, configuration, lifecycle,
health, and HTTP facade APIs. The runtime and transport implementation are
internal details of the Arqen package. Existing transport-specific re-exports
remain available only for compatibility and should not be used in new code.

## Testing hooks

Test middleware with an in-process application and verify both the request and
response paths. Test lifecycle hooks for ordering, startup failure, reverse
shutdown, cancellation, and cleanup after a server error. Never use a startup
hook as a substitute for readiness: dependencies that can become unavailable
must also implement a `HealthCheck`.
