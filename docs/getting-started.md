# Getting started

The CLI experience is:

```bash
cargo install --path cli/arqen-cli
arqen new hello-api --template thingd-app
cd hello-api
cargo run
```

The generated app starts with no database setup. Development defaults to an in-memory runtime and prints the bound API URL, health URL, storage mode, worker state, and watcher note.

Expected endpoints:

```text
http://127.0.0.1:3000/health
http://127.0.0.1:3000/ready
http://127.0.0.1:3000/docs
http://127.0.0.1:3000/agent
http://127.0.0.1:3000/agent/manifest
```

The generated README also documents the plain Cargo fallback:

```bash
cargo run
cargo test
```
