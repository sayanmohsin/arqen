# Commands

Full CLI reference for `arqen`.

## Global flags

| Flag             | Description                                  |
| ---------------- | -------------------------------------------- |
| `--version`      | Print version and exit                       |
| `--verbose`      | Enable verbose output                        |
| `--quiet`        | Suppress non-error output                    |
| `--color <when>` | Color output: `auto`, `always`, `never`      |
| `--json`         | Output as JSON (where applicable)            |
| `--file <path>`  | Config file path for `dev`, `start`, or `up` |

## Available commands

### `arqen new <NAME>`

Generate a new Arqen application with the module-based starter structure.

```bash
arqen new hello-api
cd hello-api
cargo run
```

Options: none beyond the project name. No `--template` flag is available.
Generated files are intentionally conservative; review and register each
generated module, tool, or job in the application.

The CLI refuses to overwrite an existing directory.

### `arqen generate module <NAME>`

Generate a module skeleton under `src/<name>/mod.rs`.

```bash
arqen generate module users
```

Creates `src/users/mod.rs` with a `Module` implementation stub. The CLI
prints instructions for registering the module in your application.

### `arqen generate tool <NAME>`

Generate a typed tool skeleton under `src/tools/<name>.rs`.

```bash
arqen generate tool get_user
```

Creates a `ToolMetadata` function and a `register` function. Call the
register function from your module's `register()` method.

### `arqen generate job <NAME>`

Generate a job handler skeleton under `src/jobs/<name>.rs`.

```bash
arqen generate job send_email
```

Creates a `JobHandler` implementation stub.

Generators refuse to overwrite an existing file or module directory.

### `arqen dev`

Run the application in development mode with pretty logging.

```bash
arqen dev
arqen dev --port 9000
arqen dev --storage memory --log debug
```

Options:

| Flag            | Default      | Description  |
| --------------- | ------------ | ------------ |
| `--host`        | `127.0.0.1`  | Bind address |
| `-p, --port`    | `8888`       | Port         |
| `-l, --log`     | `info`       | Log level    |
| `-s, --storage` | `memory`     | Storage mode |
| `--file`        | `arqen.toml` | Config file  |

`arqen dev` does not include an integrated file watcher. Use an external
`cargo-watch` process if you need automatic restarts.

### `arqen start`

Run the application without pretty logging (production mode).

```bash
arqen start
arqen start --port 3000 --log warn
```

Options: same as `arqen dev`. Uses JSON logging by default. Before binding,
`start` calls `AppConfig::validate_production()` and fails closed for memory
storage, missing durable paths/endpoints/credentials, disabled auth, pretty
logs, and invalid worker settings. Use `arqen dev` for permissive local work.

### `arqen up [SERVICE...]`

Start and supervise long-running dev services defined in `arqen.toml`.

```bash
arqen up                    # start all services
arqen up backend frontend   # start specific services
arqen up --dry-run          # preview what would start
arqen up --file mydev.toml  # use custom config
```

Options:

| Flag        | Default      | Description                |
| ----------- | ------------ | -------------------------- |
| `--file`    | `arqen.toml` | Config file                |
| `--dry-run` | false        | Print plan without running |

Example `arqen.toml` service definitions:

```toml
[[dev.services]]
name = "thingd"
command = "docker"
args = ["compose", "up"]

[[dev.services]]
name = "backend"
command = "cargo"
args = ["run"]
cwd = "backend"
```

Each service has a `name`, `command`, and optional `args`, `cwd`, and `env`.
If any service exits, the rest are shut down and the command exits non-zero
when the exiting service failed.

### `arqen check`

Run validation checks.

```bash
arqen check
```

`check` validates the discovered `arqen.toml` (or the path in
`ARQEN_CONFIG_FILE`) and reports missing Rust/Cargo dependencies. It does not
start the application or validate connectivity to every runtime dependency.

### `arqen lint`

Run lint checks: formatting and clippy warnings.

```bash
arqen lint
```

Checks:

1. `cargo fmt --all -- --check` — formatting
2. `cargo clippy --all-targets --all-features -- -D warnings` — clippy

Exit: `0` pass, `4` cargo missing, `5` a check failed.

### `arqen format`

Auto-fix formatting.

```bash
arqen format
```

Runs `cargo fmt --all`. Exit: `0` success, `4` cargo missing.

### `arqen test`

Run all tests.

```bash
arqen test
arqen test --release
```

Options:

| Flag        | Description                   |
| ----------- | ----------------------------- |
| `--release` | Build and run in release mode |

Exit: `0` pass, `4` cargo missing, `5` tests failed.

### `arqen build`

Build the project.

```bash
arqen build
arqen build --release
```

Options:

| Flag        | Description           |
| ----------- | --------------------- |
| `--release` | Build in release mode |

Exit: `0` success, `4` cargo missing, `5` build failed.

### `arqen doc`

Generate documentation.

```bash
arqen doc
```

Runs `cargo doc --no-deps`. Exit: `0` success, `4` cargo missing, `5` doc failed.

### `arqen doctor`

Diagnose Rust, Docker, thingd, and environment setup.

```bash
arqen doctor
```

Checks:

1. Rust and Cargo installation
2. Docker and Docker Compose
3. thingd connectivity (if `ARQEN_THINGD_URL` is set)
4. Environment variables

## Exit codes

| Code | Meaning                     |
| ---- | --------------------------- |
| 0    | Success                     |
| 2    | Usage error (bad arguments) |
| 3    | Configuration error         |
| 4    | Dependency not found        |
| 5    | Runtime error               |
| 130  | Interrupted (Ctrl+C)        |

## Config discovery

The CLI discovers configuration in this order:

1. `--file` flag (default: `arqen.toml` in current directory)
2. `ARQEN_*` environment variables
3. Compiled defaults

## JSON output

Commands that support `--json` emit structured JSON to stdout. Errors are
also emitted as JSON:

```json
{
  "ok": false,
  "error": {
    "kind": "config",
    "message": "port must be non-zero"
  }
}
```

## Running from source

From a repository checkout:

```bash
cargo run -p arqen --features cli --bin arqen -- --help
```

Install from source:

```bash
cargo install --path crates/arqen --features cli
```

The CLI is a thin process manager, project generator, and dev toolchain.
Commands such as `routes` and `agent` are not part of the current CLI surface.

## Useful project commands

Use these commands when working directly in the repository or a generated
application:

```bash
# Inspect the complete CLI surface
arqen --help
arqen dev --help
arqen generate --help

# Validate configuration and local prerequisites
arqen check
arqen doctor
arqen --json check

# Run the complete quality gate
cargo fmt --all -- --check
cargo check --workspace --all-features
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Generate and inspect API documentation
arqen doc
cargo doc --workspace --all-features --no-deps

# Run Criterion benchmarks when present
cargo bench --bench framework

# Probe a running application
curl -i http://127.0.0.1:8888/health
curl -i http://127.0.0.1:8888/ready
curl -s http://127.0.0.1:8888/agent/manifest | jq .

# Confirm the public framework identity
curl -sI http://127.0.0.1:8888/health | grep -Ei '^(server|x-powered-by):'
```

For a durable local instance, create the directory before starting:

```bash
mkdir -p .arqen/data
ARQEN_STORAGE_MODE=native \
ARQEN_PERSISTENT_PATH="$PWD/.arqen/data" \
arqen dev
```
