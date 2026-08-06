# Tooling

Arqen includes a complete developer toolchain so you never have to think
about which tools to install or run. Every command works from your project
root with zero configuration.

## Commands

| Command        | What it does                                               |
| -------------- | ---------------------------------------------------------- |
| `arqen lint`   | Check formatting (`cargo fmt --check`) and clippy warnings |
| `arqen format` | Auto-fix formatting (`cargo fmt`)                          |
| `arqen test`   | Run all tests (`cargo test --all-features`)                |
| `arqen build`  | Build the project (`cargo build`)                          |
| `arqen doc`    | Generate documentation (`cargo doc --no-deps`)             |
| `arqen check`  | Validate project structure and configuration               |
| `arqen doctor` | Diagnose Rust, Docker, and environment setup               |

## Quick workflow

```bash
arqen new my-api
cd my-api
arqen dev          # run in development mode
arqen lint         # check code quality
arqen test         # run tests
arqen build        # build for production
```

## Release builds

Pass `--release` to `test` or `build` for optimized output:

```bash
arqen test --release
arqen build --release
```

## JSON output

Every command supports `--json` for CI and scripting:

```bash
arqen --json lint
arqen --json test
arqen --json build
```

JSON output goes to stdout. Errors are also emitted as JSON.

## Exit codes

| Code | Meaning                               |
| ---- | ------------------------------------- |
| 0    | Success                               |
| 2    | Usage error (bad arguments)           |
| 3    | Configuration error                   |
| 4    | Dependency not found (cargo missing)  |
| 5    | Runtime error (check or build failed) |
| 130  | Interrupted (Ctrl+C)                  |

## Generated project tooling

When you run `arqen new`, the project ships with:

- `rustfmt.toml` — arqen default formatting config
- `clippy.toml` — arqen default lint config

These are used automatically by `arqen lint` and `arqen format`. Edit them
to customize formatting and lint rules.

## CI integration

Use the arqen commands directly in your CI pipeline:

```yaml
- name: Lint
  run: cargo run -p arqen --features cli --bin arqen -- lint

- name: Test
  run: cargo run -p arqen --features cli --bin arqen -- test

- name: Build
  run: cargo run -p arqen --features cli --bin arqen -- build
```

Or install the CLI first for faster runs:

```yaml
- name: Install CLI
  run: cargo install --path crates/arqen --features cli

- name: Lint
  run: arqen lint

- name: Test
  run: arqen test
```
