# Hot reload

GoodOne and any Arqen app can get Rust hot reload while keeping Expo HMR and the native single-process Thingd topology.

## Recommended: `arqen up` with `cargo watch`

Define the Rust service in `arqen.toml` as a `cargo watch` command and let `arqen up` supervise it alongside the frontend:

```toml
[[dev.services]]
name = "backend"
command = "cargo"
args = ["watch", "-q", "-x", "run --quiet"]
cwd = "backend"
ready_url = "http://127.0.0.1:8888/ready"
ready_timeout_seconds = 90

[[dev.services]]
name = "frontend"
command = "pnpm"
args = ["dev"]
cwd = "."
```

Run everything through Arqen:

```bash
cargo install cargo-watch   # once
pnpm dev:up                 # or: arqen up
# or: arqen up backend frontend
```

- `cargo watch` restarts the backend on Rust source, `arqen.toml`, or env-file changes; `arqen up` prefixes logs as `[backend]│` and waits for `ready_url`.
- Expo HMR (`pnpm dev` / `expo start`) is untouched - frontend hot reload is independent.
- Native single-process storage is preserved: `scripts/dev-up.sh` strips `worker` when `ARQEN_STORAGE_MODE=native|memory`, so the Thingd file is never opened by two processes.
- Do not use `arqen dev --watch` for this stack - `arqen dev` is a single-process runner; the `arqen up` service table is the correct place for watch.

## Alternative: external `cargo watch` without Arqen supervision

For ad-hoc backend-only work:

```bash
cargo watch -q -x "run --quiet"  # in backend/
```

This bypasses `arqen up` and loses the unified log prefix/ready-url handling.

## Production

The watcher must not be used in production. `arqen start` runs one process with graceful shutdown; production uses JSON logs (`ARQEN_LOG_FORMAT=json`) and the standard `cargo run` path.
