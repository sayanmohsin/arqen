# Hot reload

`arqen dev` provides a development server loop:

```bash
arqen dev
```

Install the optional watcher and run:

```bash
cargo install cargo-watch
arqen dev --watch
```

The reload loop restarts after Rust source, `arqen.toml`, or environment-file
changes. It distinguishes compilation failures from application failures and
returns the child exit status to the shell.

The watcher must not be used in production. `arqen start` runs one process with graceful shutdown.

`arqen dev` without `--watch` remains a single process. The watcher must not
be used in production; use `arqen start` behind the deployment supervisor.
