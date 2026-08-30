# Arqen

[![CI](https://github.com/sayanmohsin/arqen/actions/workflows/rust.yml/badge.svg)](https://github.com/sayanmohsin/arqen/actions/workflows/rust.yml)
[![Documentation](https://github.com/sayanmohsin/arqen/actions/workflows/docs.yml/badge.svg)](https://sayanmohsin.github.io/arqen/)
[![Quality](https://github.com/sayanmohsin/arqen/actions/workflows/quality.yml/badge.svg)](https://github.com/sayanmohsin/arqen/actions/workflows/quality.yml)
[![Crates.io](https://img.shields.io/crates/v/arqen.svg)](https://crates.io/crates/arqen)
[![docs.rs](https://docs.rs/arqen/badge.svg)](https://docs.rs/arqen)
[![Rust](https://img.shields.io/badge/rust-first-00bfff.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-00bfff.svg)](LICENSE)

## Backend infrastructure for agent-ready applications

Arqen is a Rust-first backend toolkit for HTTP services, built with Tokio,
Axum, Tower, tracing, Serde, and optional Reqwest and Clap integrations. It
brings typed tools, durable jobs, discoverable APIs, explicit modules, health
checks, and an Arqen-owned Thingd adapter contract to one readable project
structure.

“Agent-ready” does not mean AI-only. It means capabilities are discoverable,
typed, permission-aware, auditable, and automation-friendly.

Thingd storage is reached through an Arqen-owned adapter contract; Arqen is not
Thingd itself. Memory support is available in the core package. HTTP Thingd,
native Thingd, migration, and maintenance capabilities are feature-gated, and
native Thingd is included only when the optional `thingd-native` feature is
enabled. The optional `cli` feature provides the Clap-based command-line tool.
Clients in other languages can use the HTTP API and machine-readable manifests.

## Project status

Arqen is early-stage and actively maturing. The current package
contains the library and feature-gated Clap CLI, including configuration,
authentication, validation, jobs, observability, OpenAPI helpers, module
composition, testing utilities, and Thingd encryption, schema, migration, and
opt-in replication integration. Production adoption still requires
application-specific security review, durability and recovery testing, public
Thingd compatibility checks, and operational ownership.

See the [feature status](https://sayanmohsin.github.io/arqen/feature-status)
before depending on a capability. The current release is shown dynamically in
the documentation site and on [crates.io](https://crates.io/crates/arqen).

Read the live documentation on [GitHub Pages](https://sayanmohsin.github.io/arqen/).

## Quickstart

Add the core package for production HTTP deployments:

```toml
[dependencies]
arqen = "0.14"
```

Create a starter application from a checkout:

```bash
cargo run -p arqen --features cli --bin arqen -- new hello-api
cd hello-api
cargo run
```

Run the example server:

```bash
cargo run -p arqen --features cli --bin arqen -- dev --storage memory
curl http://127.0.0.1:8888/health
```

Install the CLI locally when working from a checkout:

```bash
cargo install --path crates/arqen --features cli
arqen --version
arqen --help
```

## Architecture

```text
Application, client, or agent
              |
          Arqen core
              |
   Arqen-owned ThingdBackend
       /         |          \
   memory      HTTP       native adapter
                  |             |
            Thingd server   Thingd Rust crate
```

The same application-facing contract is designed for these deployment modes:

| Mode           | Best for                                       | Status                                         |
| -------------- | ---------------------------------------------- | ---------------------------------------------- |
| Memory         | Local development and tests                    | Available                                      |
| Native durable | Local development and migration | Available with `thingd-native`; validate the compatible Thingd range and recovery |
| HTTP Thingd    | A separate Thingd HTTP service                         | Available with `http-client`; validate the public `v1` contract |
| Cloud          | Hosted thingd services                         | Future integration path                        |

Thingd supplies the durable records and replication primitives. Arqen owns
configuration, lifecycle, the stable `ThingdBackend` contract, health,
metrics, and operator workflows. Production applications should use the HTTP
adapter. Enable `thingd-native` only for local tooling that must embed the
engine, `thingd-maintenance` for native diagnostics and repair operations, and
`thingd-connectors` for optional native connector APIs. Use
`thingd-migration` for native-to-HTTP migration. See [migration](https://sayanmohsin.github.io/arqen/migration)
for the safe native-to-HTTP data movement workflow.

### Compatibility policy

| Boundary | Compatibility source | Failure mode |
| --- | --- | --- |
| Arqen core | Arqen SemVer and `ThingdBackend` contract | Cargo/API compatibility |
| Native adapter | Optional Arqen feature and compatible Thingd Cargo range | Compile-time failure or native contract test failure |
| HTTP adapter | Public Thingd REST API `v1` and required endpoint behavior | `check_compatibility()` returns a dependency error |

The native adapter currently supports Thingd `>=0.85.0, <0.86.0`. The optional
`thingd-maintenance` and `thingd-connectors` features use Thingd's public native
APIs without changing the backend-neutral contract. The public Thingd
health endpoint does not expose a stable engine version, so HTTP compatibility
is checked at the API/capability boundary rather than inferred from an
arbitrary server version string.

### Observability by default

Arqen emits readable pretty logs locally and structured JSON logs to stderr in
production. Requests carry bounded, sanitized correlation IDs and structured
fields for route, outcome, status, duration, service identity, and applicable
authentication or tenant context. Request metrics use bounded latency samples
and route labels, with timeout and dependency-error counters. See the
[Logging](https://sayanmohsin.github.io/arqen/logging) and
[Observability](https://sayanmohsin.github.io/arqen/observability) guides for
redaction rules, `RUST_LOG` precedence, Docker/journald collection, and future
external collector integration.

## For coding agents

Arqen is designed to be understood from tracked public files alone. To implement a scoped change:

1. Read `README.md` (this file) for purpose, status, and quickstart.
2. Read `specs/README.md` and `specs/STATUS.md` for phase status.
3. Read the relevant phase specification in `specs/`.
4. Read `docs/standards.md` for coding conventions.
5. Read `docs/repository-structure.md` for file locations.
6. Read source files in `crates/arqen/src/` for implementation details.
7. Run tests: `cargo test --workspace --all-features`
8. Run lints: `cargo clippy --workspace --all-targets --all-features -- -D warnings`

Do not rely on `AGENTS.md`, `.opencode/`, or other local AI instruction files. The versioned README, documentation site, and specifications are the public project contract.

## Why Arqen?

Arqen gives a backend a clear place for HTTP routes, application modules,
storage, durable work, health, and observability. It can be used with a model
runtime, frontend, BaaS, or workflow system, but does not require any of them.

## Explore the documentation

- [Documentation site on GitHub Pages](https://sayanmohsin.github.io/arqen/)
- [Getting started](https://sayanmohsin.github.io/arqen/getting-started) · [Configuration](https://sayanmohsin.github.io/arqen/configuration) · [Commands](https://sayanmohsin.github.io/arqen/commands)
- [Architecture](https://sayanmohsin.github.io/arqen/architecture) · [Modules](https://github.com/sayanmohsin/arqen/blob/main/docs/modules.md) · [Feature status](https://sayanmohsin.github.io/arqen/feature-status)
- [Authentication](https://github.com/sayanmohsin/arqen/blob/main/docs/authentication.md) · [Validation](https://github.com/sayanmohsin/arqen/blob/main/docs/validation.md) · [OpenAPI](https://github.com/sayanmohsin/arqen/blob/main/docs/openapi.md)
- [Jobs](https://sayanmohsin.github.io/arqen/durable-jobs) · [Logging](https://sayanmohsin.github.io/arqen/logging) · [Observability](https://sayanmohsin.github.io/arqen/observability) · [Testing](https://sayanmohsin.github.io/arqen/testing)
- [Durable scheduler](https://sayanmohsin.github.io/arqen/durable-jobs#durable-scheduler) · [Thingd integration](https://sayanmohsin.github.io/arqen/thingd-integration)
- [Agent guide](https://sayanmohsin.github.io/arqen/agent-guide) · [Manifest contract](https://sayanmohsin.github.io/arqen/manifest) · [thingd integration](https://sayanmohsin.github.io/arqen/thingd-integration) · [Thingd sync](https://sayanmohsin.github.io/arqen/thingd-integration)
- [Troubleshooting](https://github.com/sayanmohsin/arqen/blob/main/docs/troubleshooting.md) · [Migration](https://github.com/sayanmohsin/arqen/blob/main/docs/migration.md) · [Standards](https://github.com/sayanmohsin/arqen/blob/main/docs/standards.md)
- [Examples](https://github.com/sayanmohsin/arqen/blob/main/docs/examples.md) · [Health](https://github.com/sayanmohsin/arqen/blob/main/docs/health.md) · [Performance](https://github.com/sayanmohsin/arqen/blob/main/docs/performance.md)
- [Deployment](https://sayanmohsin.github.io/arqen/deployment) · [Docker](https://sayanmohsin.github.io/arqen/docker) · [Security](https://sayanmohsin.github.io/arqen/security)
- [Application hardening](docs/application-hardening.md) · [Logging](docs/logging.md) · [Commands](docs/commands.md)
- [Thingd bootstrap](docs/bootstrap.md) · [Thingd adapter contract](docs/adapter-contract.md)
- [HTTP caching](docs/http-caching.md) · [Streaming](docs/streaming.md) · [Performance](docs/performance.md)
- [Production runbook](docs/production-runbook.md)
- [Contributing](https://github.com/sayanmohsin/arqen/blob/main/CONTRIBUTING.md) · [Security policy](https://github.com/sayanmohsin/arqen/blob/main/SECURITY.md) · [Changelog](https://github.com/sayanmohsin/arqen/blob/main/CHANGELOG.md)

## License

Arqen is available under the [MIT License](LICENSE).
