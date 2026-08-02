# Hot reload

`arqen dev` should provide an Express-like development loop:

```bash
arqen dev
```

The first implementation may wrap `cargo-watch` rather than building a custom watcher. It should restart on Rust source, configuration, and environment-file changes; print a fresh startup banner; and distinguish compilation failures from application failures.

The watcher must not be used in production. `arqen start` runs one process with graceful shutdown.

**Current status:** Hot reload is not yet implemented. `arqen dev` currently runs the server without a watcher. Use `cargo-watch` manually for hot reload during development.
