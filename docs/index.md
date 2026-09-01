---
layout: home
title: Arqen — Build and operate a backend
description: Build a readable backend with HTTP routes, modules, jobs, health checks, and Thingd storage.

hero:
  name: Arqen
  text: Build and operate a backend.
  tagline: A Rust toolkit for HTTP services with explicit modules, durable jobs, health checks, and Thingd storage.
  actions:
    - theme: brand
      text: Build a backend →
      link: /build-a-backend
    - theme: alt
      text: GitHub
      link: https://github.com/sayanmohsin/arqen
    - theme: alt
      text: What is included
      link: /feature-status
    - theme: alt
      text: thingd.cloud
      link: https://thingd.cloud

features:
  - icon: →
    title: HTTP services
    details: Compose application routes with health, readiness, authentication, errors, and OpenAPI support.
  - icon: ◫
    title: Application modules
    details: Group related routes, jobs, configuration, and health checks into explicit modules with lifecycle hooks.
  - icon: ≋
    title: Storage adapters
    details: Start in memory, embed native Thingd, or connect to a separate Thingd service as your application grows.
  - icon: ↻
    title: Durable jobs
    details: Model retries, leases, idempotency, and dead letters in durable workers.
  - icon: ✓
    title: Operational signals
    details: Keep structured JSON logs, bounded request metrics, health, readiness, correlation IDs, and redaction guidance close to the service.
  - icon: ◈
    title: Agent tools when useful
    details: Make selected application capabilities discoverable with typed inputs, outputs, permissions, and manifests.
---

::: warning Early-stage project
Arqen is Rust-first and actively maturing. Before production use, validate
durability, Thingd compatibility, security, and operational behavior for your
application. See [Feature status](./feature-status.md) for current readiness.
:::

<ProjectStatus />

<ArqenConsole />

Arqen gives your application one place for HTTP routes, authentication, typed
tools, durable jobs, health, and storage adapters. Agent tools are optional;
the same foundation works for conventional web, mobile, internal, and
automation backends.

## Choose a starting path

| You need                       | Start here                                                      | What you get                                                          |
| ------------------------------ | --------------------------------------------------------------- | --------------------------------------------------------------------- |
| A complete backend path        | [Build a backend](./build-a-backend.md)                         | One guided route from project creation to production                  |
| A quick local prototype        | [Getting started](./getting-started.md)                         | A runnable app with memory storage                                    |
| A configurable starter app     | [CLI generator](./cli-generator.md)                             | Interactive or scripted setup for HTTP, Thingd, examples, and tooling |
| Durable single-process storage | [Deployment](./deployment.md)                                   | Embedded native Thingd with recovery responsibilities clearly stated  |
| A separate data service        | [Thingd integration](./thingd-integration.md)                   | The public HTTP adapter and its compatibility requirements            |
| To move existing data          | [Migration](./migration.md)                                     | A checked, resumable native-to-HTTP JSONL workflow                    |
| A Thingd data contract         | [Thingd schema](./schema.md)                                    | Store, validate, inspect, and operate a `.thingd` schema              |
| Agent-facing capabilities      | [Agent guide](./agent-guide.md)                                 | Discovery, permissions, typed inputs, and invocation                  |
| Production readiness           | [Production runbook](./production-runbook.md)                   | Deployment checks, health, logs, backups, and ownership               |
| Request diagnosis              | [Logging](./logging.md) and [Observability](./observability.md) | Structured logs, correlation, bounded metrics, and collector handoff  |

## Start here

Follow [Build a backend](./build-a-backend.md) for the complete path:
create a project, add a module, add routes/tools/jobs, choose storage, define
a Thingd schema, validate it, and prepare a deployment.

## Start locally

```bash
cargo run -p arqen --features cli --bin arqen -- new hello-api --yes
cd hello-api
cargo run
```

For interactive setup, remove `--yes`. The generator can add native Thingd,
starter guidance, and optional Nice Code CI while keeping Nice Code outside
the application's Rust and runtime dependencies. Read the [CLI project
generator](./cli-generator.md) guide for the full option set.

Or run the workspace server directly:

```bash
cargo run -p arqen --features cli --bin arqen -- dev --storage memory
curl http://127.0.0.1:8888/health
```

## How it fits together

```text
client or agent → Arqen API → typed tools / policies / jobs / logs
                              ↓
              memory · native durable thingd · HTTP sidecar · future cloud
```

Application code owns domain behavior. Arqen provides HTTP, module, job,
health, and adapter integrations. Thingd provides durable data primitives. See
[Architecture](./architecture.md) and [Thingd integration](./thingd-integration.md).

## thingd integration

thingd supplies the object, event, search, link, and queue records that Arqen
adapts. Arqen keeps that integration optional and public-contract based. Cloud
hosting is a future path rather than a current promise.

Learn more about the thingd ecosystem at
[thingd.cloud](https://thingd.cloud), the home for thingd’s hosted data engine
and related services.

## Add agent capabilities when you need them

Typed tools and manifests are optional. Add them when people, programs, or
agents need to discover and invoke selected application operations. Read
[Typed tools](./typed-tools.md) and [Agent guide](./agent-guide.md).

## Roadmap

Read the [roadmap](./roadmap.md) for planned work and the
[Feature status](./feature-status.md) page for what is currently available.

## Performance

Read the [performance guide](./performance.md) for benchmarks, profiling,
and optimization patterns.
