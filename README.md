# Arqen

[![CI](https://github.com/sayanmohsin/arqen/actions/workflows/rust.yml/badge.svg)](https://github.com/sayanmohsin/arqen/actions/workflows/rust.yml)
[![Tests](https://github.com/sayanmohsin/arqen/actions/workflows/rust.yml/badge.svg?label=tests)](https://github.com/sayanmohsin/arqen/actions/workflows/rust.yml)
[![Rust](https://img.shields.io/badge/rust-first-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-00d9ff.svg)](https://sayanmohsin.github.io/arqen/)

## Backend infrastructure for agent-ready applications

Arqen is a developer-focused backend toolkit with typed tools, durable jobs,
discoverable APIs, and thingd integration. It is for teams building services
that need a clear HTTP boundary, explicit application state, operational
signals, and automation-friendly contracts.

“Agent-ready” does not mean AI-only. It means an application is discoverable,
typed, permission-aware, auditable, and automation-friendly for people,
programs, and agents alike.

## Status

Arqen is an early-stage framework with a working single-package implementation.
The library, CLI, middleware, configuration, authentication, validation,
health, jobs, observability, OpenAPI helpers, module composition, and testing
utilities are available in the `arqen` package. Durable thingd behavior, public
HTTP parity, security review, and application-specific integration still need
to be validated for each production deployment. Read the [feature status](docs/feature-status.md)
before relying on a feature.

The public product positioning is language-agnostic. Rust is the first
implementation; future Node.js support can arrive through the public HTTP API,
SDKs, templates, and shared manifests without making application users learn
Rust.

## Quickstart

Most applications can start with the single public facade crate:

```toml
[dependencies]
arqen = "0.3"
```

The source remains modular internally, while the public distribution stays
focused on one package.

From a checkout:

```bash
cargo run -p arqen --features cli --bin arqen -- new hello-api --template thingd-app
cd hello-api
cargo run
```

For the workspace server itself:

```bash
cargo run -p arqen --features cli --bin arqen -- dev --storage memory
curl http://127.0.0.1:3000/health
```

The current CLI also supports `start`, `check`, and `doctor`. `dev` currently
starts the server and prints the documented `cargo watch` loop; it does not yet
provide an integrated watcher.

## Architecture

```text
Application, client, or agent
              |
       Axum HTTP boundary
              |
  typed tools · policies · jobs · logs
              |
       Arqen adapter contract
        /         |          \
   memory     native durable   HTTP sidecar
  development   thingd        thingd service
                                  |
                         future optional cloud API
```

The same application-facing contracts are intended to work across four
deployment modes:

| Mode | Purpose | Current posture |
|---|---|---|
| Memory | Fast local development and tests | Available in the Rust workspace |
| Native durable | Embedded durable thingd storage | Implemented; recovery and workload validation remain deployment responsibilities |
| HTTP sidecar | Separate thingd service boundary | Implemented adapter; validate against the current public thingd contract |
| Cloud | Optional hosted thingd service | Future, public-contract dependent |

## What makes Arqen distinct

Arqen is the layer between an ordinary backend and the software that needs to
operate it. It does not replace a web framework, an AI model runtime, a hosted
BaaS, or a standalone workflow engine. It gives those systems a shared,
inspectable contract for capabilities, data, jobs, and operations.

- Compared with a traditional web framework, Arqen makes tools, manifests,
  permissions, jobs, health, and auditability first-class.
- Compared with an agent framework, Arqen is model-agnostic. Agents are
  clients of the application rather than the application’s architecture.
- Compared with a BaaS, Arqen keeps deployment and adapter boundaries visible
  instead of requiring a hosted control plane.
- Compared with a workflow engine, durable jobs are one backend primitive
  alongside HTTP, storage, tools, logs, and readiness.
- Compared with a microservice stack, Arqen starts with one explicit
  application boundary and adds sidecars or cloud adapters only when useful.

See [Why Arqen?](docs/why-arqen.md) for the detailed comparison.

## Explore the docs

- [About Arqen](docs/about.md) · [Why Arqen?](docs/why-arqen.md) · [Use cases](docs/use-cases.md)
- [Feature status](docs/feature-status.md) · [Architecture](docs/architecture.md) · [Roadmap](docs/roadmap.md)
- [Getting started](docs/getting-started.md) · [Commands](docs/commands.md) · [Configuration](docs/configuration.md)
- [Agent guide](docs/agent-guide.md) · [Agent discovery](docs/agent-discovery.md) · [Typed tools](docs/typed-tools.md)
- [Operations](docs/deployment.md) · [Docker](docs/docker.md) · [Security](docs/security.md) · [Release](docs/release.md)
- [Contributing](docs/contributing.md) · [FAQ](docs/faq.md)

The full documentation site is available at
[sayanmohsin.github.io/arqen](https://sayanmohsin.github.io/arqen/).

## License

Arqen is licensed under the [MIT License](LICENSE).
