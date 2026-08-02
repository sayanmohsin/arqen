# Commands

| Command | Purpose |
|---|---|
| `arqen new NAME` | Generate an application from a template. |
| `arqen dev` | Run in development mode with pretty logging. The integrated watcher is not implemented yet. |
| `arqen start` | Run the application without a watcher. |
| `arqen check` | Run the current CLI check command. |
| `arqen doctor` | Inspect Rust, Docker, thingd, and environment setup. |

Useful options include `--host`, `--port`, `--log`, and `--storage`.

The CLI remains a thin process manager and project generator. It does not hide
normal Cargo commands. Planned commands such as `test`, `routes`, and `agent`
are not part of the current CLI surface.
