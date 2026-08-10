# Arqen

[![CI](https://github.com/sayanmohsin/arqen/actions/workflows/rust.yml/badge.svg)](https://github.com/sayanmohsin/arqen/actions/workflows/rust.yml)
[![Documentation](https://github.com/sayanmohsin/arqen/actions/workflows/docs.yml/badge.svg)](https://sayanmohsin.github.io/arqen/)
[![Quality](https://github.com/sayanmohsin/arqen/actions/workflows/quality.yml/badge.svg)](https://github.com/sayanmohsin/arqen/actions/workflows/quality.yml)
[![Crates.io](https://img.shields.io/crates/v/arqen.svg)](https://crates.io/crates/arqen)
[![docs.rs](https://docs.rs/arqen/badge.svg)](https://docs.rs/arqen)
[![Rust](https://img.shields.io/badge/rust-first-00bfff.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-00bfff.svg)](LICENSE)

## Backend infrastructure for agent-ready applications

Arqen is a developer-focused backend toolkit for services that people,
programs, and agents can operate. It brings typed tools, durable jobs,
discoverable APIs, explicit modules, health checks, and thingd integration to
one application boundary.

“Agent-ready” does not mean AI-only. It means capabilities are discoverable,
typed, permission-aware, auditable, and automation-friendly.

Arqen is Rust-first internally, built on Tokio, Tower, tracing, and thingd,
with Axum as its current HTTP transport. Its application positioning is language-agnostic: future Node.js
support can use the public HTTP API, SDKs, templates, and shared manifests.

## Project status

Arqen is early-stage and actively maturing. The current single package
contains the library and feature-gated CLI, including configuration,
authentication, validation, jobs, observability, OpenAPI helpers, module
composition, testing utilities, and Thingd 0.79.0 encryption, schema, and
opt-in replication integration. Production adoption still requires
application-specific security review, durability and recovery testing, public
thingd compatibility checks, and operational ownership.

See the [feature status](https://sayanmohsin.github.io/arqen/feature-status)
before depending on a capability. The current implementation is Arqen 0.6.1;
publish that patch before consumers replace their temporary local path pin.

## Quickstart

Add the one public Cargo package:

```toml
[dependencies]
arqen = "0.6"
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
       Arqen HTTP boundary
              |
 tools · policies · jobs · health · logs
              |
       Arqen adapter contract
        /         |          \
   memory     native durable   HTTP Thingd
  local/test  embedded engine  Thingd service
                                  |
                         optional Cloud replica
```

The same application-facing contracts are designed for four deployment modes:

| Mode           | Best for                    | Status                                         |
| -------------- | --------------------------- | ---------------------------------------------- |
| Memory         | Local development and tests | Available                                      |
| Native durable | An embedded Thingd engine in the Arqen process | Available; validate recovery for your workload |
| HTTP Thingd    | A separate Thingd HTTP service                | Available; validate the public contract        |
| Cloud          | Hosted thingd services      | Future integration path                        |

Thingd 0.79.0 adds the supported integration boundary for encrypted native
storage, versioned `.thingd` schema inspection, and opt-in HTTP source-to-
replica sync. Thingd owns encryption format, checkpoints, tombstones, and
conflict semantics; Arqen owns configuration, lifecycle, typed adapters,
health, and metrics around those capabilities. Native storage is embedded in
the Arqen process and does not require a local sidecar. Native-to-Cloud sync
uses Thingd's public native replication service and the existing HTTP target
contract.

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

Arqen is a contract layer between an application and the software that
operates it. It is not a model runtime, BaaS, or workflow engine.

- Traditional web frameworks get typed tools, manifests, policies, jobs, and
  readiness signals.
- Agent frameworks get a model-agnostic application boundary.
- BaaS deployments keep their adapter and operational boundaries visible.
- Workflow systems get a home for HTTP, storage, observability, and durable
  work.

## Explore the documentation

- [Documentation site](https://sayanmohsin.github.io/arqen/)
- [Getting started](https://sayanmohsin.github.io/arqen/getting-started) · [Configuration](https://sayanmohsin.github.io/arqen/configuration) · [Commands](https://sayanmohsin.github.io/arqen/commands)
- [Architecture](https://sayanmohsin.github.io/arqen/architecture) · [Modules](https://github.com/sayanmohsin/arqen/blob/main/docs/modules.md) · [Feature status](https://sayanmohsin.github.io/arqen/feature-status)
- [Authentication](https://github.com/sayanmohsin/arqen/blob/main/docs/authentication.md) · [Validation](https://github.com/sayanmohsin/arqen/blob/main/docs/validation.md) · [OpenAPI](https://github.com/sayanmohsin/arqen/blob/main/docs/openapi.md)
- [Jobs](https://sayanmohsin.github.io/arqen/durable-jobs) · [Observability](https://github.com/sayanmohsin/arqen/blob/main/docs/observability.md) · [Testing](https://github.com/sayanmohsin/arqen/blob/main/docs/testing.md)
- [Agent guide](https://sayanmohsin.github.io/arqen/agent-guide) · [Manifest contract](https://sayanmohsin.github.io/arqen/manifest) · [thingd integration](https://sayanmohsin.github.io/arqen/thingd-integration) · [Thingd 0.79 sync](https://sayanmohsin.github.io/arqen/thingd-integration#thingd-079-encryption-schemas-and-sync)
- [Troubleshooting](https://github.com/sayanmohsin/arqen/blob/main/docs/troubleshooting.md) · [Migration](https://github.com/sayanmohsin/arqen/blob/main/docs/migration.md) · [Standards](https://github.com/sayanmohsin/arqen/blob/main/docs/standards.md)
- [Examples](https://github.com/sayanmohsin/arqen/blob/main/docs/examples.md) · [Health](https://github.com/sayanmohsin/arqen/blob/main/docs/health.md) · [Performance](https://github.com/sayanmohsin/arqen/blob/main/docs/performance.md)
- [Deployment](https://sayanmohsin.github.io/arqen/deployment) · [Docker](https://sayanmohsin.github.io/arqen/docker) · [Security](https://sayanmohsin.github.io/arqen/security)
- [Application hardening](docs/application-hardening.md) · [Logging](docs/logging.md) · [Commands](docs/commands.md)
- [Production runbook](docs/production-runbook.md)
- [Contributing](https://github.com/sayanmohsin/arqen/blob/main/CONTRIBUTING.md) · [Security policy](https://github.com/sayanmohsin/arqen/blob/main/SECURITY.md) · [Changelog](https://github.com/sayanmohsin/arqen/blob/main/CHANGELOG.md)

## License

Arqen is available under the [MIT License](LICENSE).
