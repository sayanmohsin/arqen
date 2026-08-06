# Troubleshooting

Common issues and their solutions.

## Generated project won't compile

Ensure `Cargo.toml` depends on the correct version and features:

```toml
[dependencies]
arqen = { version = "0.4", features = ["logging", "http-server"] }
```

If you see missing types, check that the feature flags match the APIs you use.
The `cli` feature is for the CLI binary only and should not be added to
application dependencies.

## CLI not found

The CLI is part of the `arqen` crate but is behind a feature flag. Install it
from a checkout:

```bash
cargo install --path crates/arqen --features cli
```

Verify with:

```bash
arqen --help
```

If `arqen` is not on your `PATH`, check `~/.cargo/bin`.

## Port already in use

Arqen binds to `127.0.0.1:8888` by default. If another process is using that
port, either stop the other process or change the port:

```bash
arqen dev --port 9000
# or
ARQEN_PORT=9000 cargo run
```

## Docker not found

The `arqen doctor` command checks for Docker and Docker Compose. If you need
Docker for thingd services:

1. Install [Docker Desktop](https://docs.docker.com/get-docker/) or Docker Engine.
2. Verify with `docker --version`.
3. Run `arqen doctor` to confirm Docker and Docker Compose are detected.

If Docker is installed but not running, start Docker Desktop or the Docker
daemon before running `arqen up`.

## thingd connection refused

When using HTTP storage mode, Arqen connects to a thingd service at the URL
configured via `ARQEN_THINGD_URL` or `storage.http_url` in `arqen.toml`.

Checklist:

1. Verify the thingd service is running.
2. Confirm `ARQEN_THINGD_URL` is set correctly (e.g., `http://localhost:8080`).
3. Run `arqen doctor` to see the current configuration.
4. Test connectivity manually: `curl $ARQEN_THINGD_URL/health`.

If you do not need a remote thingd service, use memory mode (the default):

```bash
arqen dev --storage memory
```

## Module graph errors

Arqen validates the module graph at startup. Common errors:

| Error                                               | Cause                          | Fix                                                  |
| --------------------------------------------------- | ------------------------------ | ---------------------------------------------------- |
| `duplicate module: X`                               | Two modules with the same name | Rename one module                                    |
| `module 'X' depends on 'Y' which is not registered` | Missing dependency             | Register module Y or remove the dependency           |
| `dependency cycle detected: X -> Y -> X`            | Circular dependency            | Break the cycle by restructuring module dependencies |

Ensure every module returned by `dependencies()` is registered in the
`ModuleBuilder` before calling `register_all()`.

## Health check failing

If `/ready` returns 503:

1. Check which dependency is unhealthy in the JSON response.
2. Ensure your module implements `health_check()` to return real dependency
   status, not always `ModuleHealth::Healthy`.
3. If a check is not critical for readiness, implement
   `required_for_readiness() -> false` on your `HealthCheck`.

Example from tests (`health.rs`):

```rust
struct OptionalCheck;

#[async_trait]
impl HealthCheck for OptionalCheck {
    fn name(&self) -> &str { "optional_check" }
    async fn check(&self) -> HealthStatus { HealthStatus::Healthy }
    fn required_for_readiness(&self) -> bool { false }
}
```
