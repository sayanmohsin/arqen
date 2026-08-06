# Getting started

10-minute path from zero to a running Arqen application.

## Prerequisites

- **Rust 1.96+** (edition 2024): install via [rustup](https://rustup.rs/)
- **pnpm** (for building docs only): install via `npm install -g pnpm`
- **Docker** (optional): needed for thingd HTTP mode and `arqen up` services

Verify your Rust installation:

```bash
rustc --version   # should show 1.96 or newer
cargo --version
```

## Install the CLI

From a repository checkout:

```bash
cargo install --path crates/arqen --features cli
```

Or run directly without installing:

```bash
cargo run -p arqen --features cli --bin arqen -- --help
```

## Create a project

```bash
arqen new hello-api
cd hello-api
```

This generates:

```text
hello-api/
  Cargo.toml          # depends on arqen 0.4 with logging + http-server
  README.md
  rustfmt.toml        # formatting config
  clippy.toml         # lint config
  src/
    main.rs           # entry point
    app/mod.rs        # AppModule (Module trait)
```

## Run the project

```bash
cargo run
```

Expected output:

```text
Arqen v0.4.0
API:    http://127.0.0.1:8888
Health: http://127.0.0.1:8888/health
Docs:   http://127.0.0.1:8888/docs
Agent:  http://127.0.0.1:8888/agent
Storage: memory
```

## Test the endpoints

```bash
curl http://127.0.0.1:8888/health
curl http://127.0.0.1:8888/ready
curl http://127.0.0.1:8888/agent
curl http://127.0.0.1:8888/agent/manifest
curl http://127.0.0.1:8888/docs
```

## Lint and test

```bash
arqen lint         # check formatting + clippy
arqen test         # run all tests
```

## Add your first module

```bash
arqen generate module users
```

This creates `src/users/mod.rs`. Register it in `src/app/mod.rs`:

```rust
mod users;

// In AppModule:
fn dependencies(&self) -> Vec<&str> {
    vec!["users"]
}
```

## Add your first tool

```bash
arqen generate tool get_user
```

This creates `src/tools/get_user.rs`. Register it in your module's
`register()` method:

```rust
fn register(&self, ctx: &mut ModuleContext<'_>) -> Result<(), AppError> {
    crate::tools::get_user::register(ctx)?;
    Ok(())
}
```

## Add your first job

```bash
arqen generate job send_email
```

This creates `src/jobs/send_email.rs` with a `JobHandler` stub.

## Run from the workspace

If you prefer not to install the CLI:

```bash
cargo run -p arqen --features cli --bin arqen -- new hello-api
```

## Next steps

- [Commands](./commands.md) - full CLI reference
- [Configuration](./configuration.md) - environment variables and config files
- [Modules](./modules.md) - module composition and lifecycle
- [Typed tools](./typed-tools.md) - structured tool definitions
- [Durable jobs](./durable-jobs.md) - background job processing
- [Authentication](./authentication.md) - JWT and API key auth
- [Validation](./validation.md) - request validation
- [Health](./health.md) - health and readiness checks
- [Deployment](./deployment.md) - production deployment
- [Examples](./examples.md) - code snippets and examples
