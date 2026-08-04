# Getting started

The CLI is part of the single published `arqen` package. Install it from a
checkout with:

```bash
cargo install --path crates/arqen --features cli
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

To run the CLI directly from this repository:

```bash
cargo run -p arqen --features cli --bin arqen -- new hello-api --template thingd-app
```
