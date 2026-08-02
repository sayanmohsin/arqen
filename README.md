# Arqen

[![CI](https://github.com/sayanmohsin/arqen/actions/workflows/rust.yml/badge.svg)](https://github.com/sayanmohsin/arqen/actions/workflows/rust.yml)
[![Tests](https://github.com/sayanmohsin/arqen/actions/workflows/rust.yml/badge.svg?label=tests)](https://github.com/sayanmohsin/arqen/actions/workflows/rust.yml)
[![Rust](https://img.shields.io/badge/rust-first-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-orange.svg)](https://sayanmohsin.github.io/arqen/)

## Backend infrastructure for agent-ready applications

Arqen is a developer-focused backend toolkit with typed tools, durable jobs,
discoverable APIs, and thingd integration. It is for teams building services
that need a clear HTTP boundary, explicit application state, operational
signals, and automation-friendly contracts.

“Agent-ready” does not mean AI-only. It means an application is discoverable,
typed, permission-aware, auditable, and automation-friendly for people,
programs, and agents alike.

## Status

Arqen is early-stage. The repository contains working Rust crates and examples
alongside contracts and planned work. The Rust implementation uses Axum,
Tokio, Tower, tracing, and native thingd adapters. Native thingd migration,
public HTTP parity, and CLI/template completion remain important maturity gates.
Read the [feature status](docs/feature-status.md) before relying on a feature.

The public product positioning is language-agnostic. Rust is the first
implementation; future Node.js support can arrive through the public HTTP API,
SDKs, templates, and shared manifests without making application users learn
Rust.

## Quickstart

From a checkout:

```bash
cargo run -p arqen-cli -- new hello-api --template thingd-app
cd hello-api
cargo run
```

For the workspace server itself:

```bash
cargo run -p arqen-cli -- dev --storage memory
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
| Native durable | Embedded durable thingd storage | In progress / migration gate |
| HTTP sidecar | Separate thingd service boundary | Contract and adapter work exists; parity is still being hardened |
| Cloud | Optional hosted thingd service | Future, public-contract dependent |

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
