---
title: CLI project generator
description: Generate a ready-to-run Arqen application with optional storage, logging, examples, and Nice Code setup.
---

# CLI project generator

`arqen new` creates a complete starter application without adding optional
development tools to the application's runtime dependency graph.

## Quick start

```bash
arqen new hello-api
cd hello-api
cargo run
```

When run in a terminal, the generator asks whether to include:

- the HTTP server and an `/api/hello` route;
- embedded native Thingd storage;
- structured logging;
- starter module, tool, and job guidance;
- optional Nice Code documentation and GitHub Actions CI.

The default choices create an HTTP application with logging and memory
storage. The generated app can run without Node.js or Nice Code.

## Scripted generation

Use `--yes` to accept the defaults without prompts. Set the output directory
independently from the Rust package name:

```bash
arqen new catalog-api --output ./services/catalog --yes
```

Optional capabilities can be selected or disabled explicitly:

```bash
arqen new catalog-api --yes --thingd --examples --nice-code
arqen new worker --yes --no-http --no-logging
```

The command refuses to overwrite an existing directory. Use `--json` when a
script needs the generated file list and selected options as machine-readable
output.

## Generated layout

Every project includes:

```text
Cargo.toml       # Current Arqen release and selected features
arqen.toml      # Memory or native storage configuration
.env.example     # Safe environment-variable template
AGENTS.md        # Portable project guidance
README.md        # Run, test, and extension instructions
rustfmt.toml
clippy.toml
src/main.rs
```

HTTP projects also include `src/app/mod.rs` with an `AppModule` and the
`/api/hello` starter route. With `--examples`, the project includes guidance
for `arqen generate module`, `arqen generate tool`, and
`arqen generate job`.

## Native Thingd

Selecting `--thingd` adds Arqen's `thingd-native` feature and configures a
local `.data/thingd` directory. Review recovery, backup, resource, and schema
requirements before using native storage in production. See
[Thingd integration](./thingd-integration.md).

## Optional Nice Code

Selecting `--nice-code` adds `NICE_CODE.md` and
`.github/workflows/nice-code.yml`. The generated workflow runs:

```bash
npx --yes @sayanmohsin/nice-code@0.1.11 --ci --project . --format sarif
```

Nice Code is not added to `Cargo.toml`, is not required to run the app, and
can be removed without changing Arqen behavior. See
[Tooling](./tooling.md) for repository and consumer usage.

## Extending the project

The individual generators remain available after project creation:

```bash
arqen generate module users
arqen generate tool get_user
arqen generate job send_email
```

Read [Commands](./commands.md) for the complete CLI reference.
