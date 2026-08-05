# Hot reload

`arqen dev` provides a development server loop:

```bash
arqen dev
```

An external `cargo-watch` process may be used to restart on Rust source,
configuration, and environment-file changes. It should print a fresh startup
banner and distinguish compilation failures from application failures.

The watcher must not be used in production. `arqen start` runs one process with graceful shutdown.

**Current status:** Hot reload is not yet implemented. `arqen dev` currently runs the server without a watcher. Use `cargo-watch` manually for hot reload during development.
