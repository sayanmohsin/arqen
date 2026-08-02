---
layout: home
title: Arqen — Backend infrastructure for agent-ready applications
description: A developer-focused backend toolkit with typed tools, durable jobs, discoverable APIs, and thingd integration.

hero:
  name: Arqen
  text: Backend infrastructure for agent-ready applications.
  tagline: Typed tools, durable jobs, discoverable APIs, and thingd integration for services that people, programs, and agents can operate.
  actions:
    - theme: brand
      text: Get started →
      link: /getting-started
    - theme: alt
      text: GitHub
      link: https://github.com/sayanmohsin/arqen
    - theme: alt
      text: Feature status
      link: /feature-status
    - theme: alt
      text: thingd.cloud
      link: https://thingd.cloud

features:
  - icon: ◈
    title: Typed tools
    details: Make application capabilities discoverable with structured inputs, outputs, permissions, and audit metadata.
  - icon: ↻
    title: Durable jobs
    details: Model retries, leases, idempotency, and dead letters at a clear application boundary.
  - icon: ◫
    title: Shared manifests
    details: Publish endpoints, tools, jobs, and runtime metadata for clients and agents to inspect.
  - icon: ≋
    title: Useful logging
    details: Start with tracing, request visibility, redaction guidance, health, and readiness signals.
  - icon: ✓
    title: Health by default
    details: Keep liveness and readiness visible to local development, CI, and deployment systems.
  - icon: →
    title: Deployment paths
    details: Move from memory mode toward native durable thingd, an HTTP sidecar, or a future cloud contract.
---

::: warning Early-stage project
Arqen is Rust-first and actively maturing. Native durable thingd migration,
public HTTP parity, and CLI/template completion are still project gates. Check
the [feature status](./feature-status.md) before adopting a capability.
:::

## Rust-first implementation, language-agnostic direction

The current implementation uses Rust, Axum, Tokio, Tower, tracing, and native
thingd adapters. The application positioning stays language-agnostic: future
Node.js support can use the public HTTP API, SDKs, templates, and shared
manifests.

## Start locally

```bash
cargo run -p arqen-cli -- new hello-api --template thingd-app
cd hello-api
cargo run
```

Or run the workspace server directly:

```bash
cargo run -p arqen-cli -- dev --storage memory
curl http://127.0.0.1:3000/health
```

## Architecture

```text
client or agent → Axum API → typed tools / policies / jobs / logs
                              ↓
              memory · native durable thingd · HTTP sidecar · future cloud
```

Application code should depend on domain interfaces and the public thingd
contract, not private cloud modules. See [architecture](./architecture.md) and
[thingd integration](./thingd-integration.md).

## thingd integration

thingd supplies the storage, events, search, links, and queue boundary that
Arqen adapts. Arqen keeps that integration optional and public-contract based;
cloud hosting is a future path rather than a current promise.

Learn more about the thingd ecosystem at
[thingd.cloud](https://thingd.cloud), the home for thingd’s hosted data engine
and related services.

## A different layer

Arqen is not another model runtime or hosted backend. It is the contract layer
between an application and the software that operates it:

- web frameworks get typed tools, manifests, permissions, jobs, and health;
- agent frameworks get a model-agnostic application boundary;
- BaaS products get an explicit adapter and deployment model;
- workflow systems get an HTTP, storage, and observability home;
- microservice stacks get a clear starting boundary before sidecars multiply.

Read [Why Arqen?](./why-arqen.md) for the tradeoffs and the honest maturity
boundary.

## Roadmap

Read the [roadmap](./roadmap.md) and the [Phase 12 specification on GitHub](https://github.com/sayanmohsin/arqen/tree/main/specs/phase-12-documentation-and-public-presence)
for implementation evidence and open work.
