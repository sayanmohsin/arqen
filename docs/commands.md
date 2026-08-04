# Commands

| Command | Purpose |
|---|---|
| `arqen new NAME` | Generate an application with the module-based starter structure. |
| `arqen generate module NAME` | Generate a module skeleton under `src/`. |
| `arqen generate tool NAME` | Generate a typed tool metadata skeleton under `src/tools/`. |
| `arqen generate job NAME` | Generate a job handler skeleton under `src/jobs/`. |
| `arqen dev` | Run in development mode with pretty logging. The integrated watcher is not implemented yet. |
| `arqen start` | Run the application without a watcher. |
| `arqen check` | Run the current CLI check command. |
| `arqen doctor` | Inspect Rust, Docker, thingd, and environment setup. |

Useful options include `--host`, `--port`, `--log`, and `--storage`.

`arqen new` currently accepts only the project name. It does not have a
`--template` flag. Generated files are intentionally conservative: review and
register each generated module, tool, or job in the application.

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

Generators refuse to overwrite an existing file or module directory.
