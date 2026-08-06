# Agent guide

How to implement a scoped change as a coding agent working on Arqen.

## Reading order

1. **README.md** - project overview, build instructions, test commands
2. **specs/** - phase specifications with requirements and acceptance criteria
3. **docs/** - public documentation site (VitePress)
4. **Source code** - implementation in `crates/arqen/src/`

Start with the README to understand the build and test workflow. Then read
the relevant spec for the phase you are working on. Consult docs for
public API contracts and usage patterns.

## Working with tracked files only

Use only public, tracked files:

- `crates/arqen/src/**/*.rs` - library and CLI source
- `docs/**/*.md` - documentation
- `examples/**/*.rs` - example applications
- `specs/**/*.md` - phase specifications
- `Cargo.toml`, `CHANGELOG.md`, `README.md`

Do not read or modify:

- `AGENTS.md` - private agent instructions
- Files under `.opencode/` - private tool configuration
- Internal test fixtures that reference private workflows

## Implementing a scoped change

### 1. Understand the requirement

Read the spec and identify:

- What is being added or changed
- What tests or contract tests are required
- What documentation must be updated

### 2. Find the right location

- **Core types**: `crates/arqen/src/core/`
- **HTTP routes**: `crates/arqen/src/http/`
- **Agent tools**: `crates/arqen/src/agent/`
- **Health checks**: `crates/arqen/src/health.rs`
- **Module system**: `crates/arqen/src/module.rs`
- **Configuration**: `crates/arqen/src/config.rs`
- **Documentation**: `docs/`

### 3. Implement with existing patterns

- Follow the code style of neighboring files
- Use existing types (`AppError`, `HealthStatus`, `ModuleHealth`)
- Add `///` doc comments to all public items
- Gate unstable features behind Cargo feature flags

### 4. Test

```bash
# Run all tests
cargo test -p arqen --all-features

# Run clippy
cargo clippy -p arqen --all-targets --all-features -- -D warnings

# Run the CLI from source
cargo run -p arqen --features cli --bin arqen -- --help
```

### 5. Document

- Update relevant docs in `docs/`
- Update `CHANGELOG.md` for user-facing changes
- Add code examples if the change introduces new public API

## Testing and validation commands

| Command                                                             | Purpose                          |
| ------------------------------------------------------------------- | -------------------------------- |
| `cargo test -p arqen --all-features`                                | Run all library and binary tests |
| `cargo clippy -p arqen --all-targets --all-features -- -D warnings` | Lint check                       |
| `cargo build -p arqen --all-features`                               | Build check                      |
| `cargo run -p arqen --features cli --bin arqen -- --help`           | CLI smoke test                   |
| `cd docs && pnpm dev`                                               | Documentation site               |

## Conventions

- Prefer Axum, Tokio, Tower, tracing, and explicit application state
- Do not create a NestJS-like dependency-injection framework
- Keep provider and cloud credentials server-side
- Treat the public thingd HTTP API as the first integration boundary
- Preserve in-memory and durable adapter parity
- Add docs and contract tests before implementation features
