# Commands

| Command | Purpose |
|---|---|
| `arqen new NAME` | Generate an application from a template. |
| `arqen dev` | Run in development mode with pretty logging. The integrated watcher is not implemented yet. |
| `arqen start` | Run the application without a watcher. |
| `arqen check` | Run the current CLI check command. |
| `arqen doctor` | Inspect Rust, Docker, thingd, and environment setup. |

Useful options include `--host`, `--port`, `--log`, and `--storage`.

The CLI is the binary in the published `arqen` package. From a checkout, use:

```bash
cargo run -p arqen --features cli --bin arqen -- --help
```

Install it from the repository with:

```bash
cargo install --path crates/arqen --features cli
```

The CLI remains a thin process manager and project generator. It does not hide
normal Cargo commands. Commands such as `test`, `routes`, and `agent` are not
part of the current CLI surface. `arqen dev` does not include an integrated file
watcher; use an external `cargo-watch` process if you need automatic restarts.
