---
title: Build an Arqen backend
description: A guided path from a new Arqen project to a backend with modules, tools, jobs, Thingd storage, and a validated schema.
---

# Build an Arqen backend

This is the main path through the docs. Follow it in order when creating a
backend. Each step leaves you with a working application before you add the
next boundary.

<MermaidDiagram type="use-cases" />

## The path

| Step                        | Read                                    | Outcome                                                   |
| --------------------------- | --------------------------------------- | --------------------------------------------------------- |
| 1. Create the app           | [Getting started](./getting-started.md) | A runnable Rust backend with health and agent endpoints   |
| 2. Understand the shape     | [Architecture](./architecture.md)       | Know what belongs in Arqen, your app, and Thingd          |
| 3. Add a feature            | [Modules](./modules.md)                 | Routes, tools, jobs, and checks grouped by domain         |
| 4. Add an agent capability  | [Typed tools](./typed-tools.md)         | Typed inputs, outputs, permissions, and manifest metadata |
| 5. Add background work      | [Durable jobs](./durable-jobs.md)       | Leases, retries, idempotency, and dead letters            |
| 6. Choose storage           | [Configuration](./configuration.md)     | Memory, native Thingd, or HTTP Thingd selected explicitly |
| 7. Define and validate data | [Thingd schema](./schema.md)            | A versioned `.thingd` schema inspected before startup     |
| 8. Prepare production       | [Deployment](./deployment.md)           | Secrets, health, backups, workers, and readiness checks   |

## 1. Create the project

Install or run the CLI from the Arqen checkout:

```bash
cargo install --path crates/arqen --features cli
arqen new catalog-api
cd catalog-api
cargo run
```

The generated app starts with memory storage. Confirm the boundary before
writing application code:

```bash
curl http://127.0.0.1:8888/health
curl http://127.0.0.1:8888/ready
curl http://127.0.0.1:8888/agent/manifest
curl http://127.0.0.1:8888/docs
```

Use `arqen dev` for permissive local development and `arqen start` for the
strict production path.

## 2. Organize the backend by modules

Create a module for a domain boundary such as `catalog`, `accounts`, or
`billing`:

```bash
arqen generate module catalog
```

A module is the explicit place to register its routes, tools, health checks,
and jobs. Declare dependencies by module name so startup order is visible and
validated. Keep domain services and request models in your application; Arqen
provides the composition and lifecycle boundary.

Read [Modules and application composition](./modules.md) before adding a
large module graph.

## 3. Add tools and jobs

Generate the two common agent-facing boundaries:

```bash
arqen generate tool create_product
arqen generate job rebuild_search
```

For each tool, define a stable name, description, JSON input/output schema,
required scopes, read/write effect, and idempotency behavior. If the operation
will outlive the request, enqueue a job instead of doing the work inline.

Read [Typed tools](./typed-tools.md) and [Durable jobs](./durable-jobs.md) for
the contracts and worker rules.

## 4. Choose storage deliberately

Start with memory mode:

```bash
arqen dev --storage memory
```

Move to embedded native Thingd when one process owns the durable store:

```toml
[storage]
mode = "native"
persistent_path = "/var/lib/catalog-api/data"
schema_path = "schema.thingd"
```

Use HTTP Thingd when the data service should be separate from the backend:

```toml
[storage]
mode = "http"
http_url = "https://thingd.internal"
auth_token = "server-side-token"
schema_path = "schema.thingd"
```

The application-facing `ThingdBackend` contract stays the same across these
modes. Read [Thingd integration](./thingd-integration.md) before using events,
search, links, queues, or replication.

## 5. Add a schema before production

Keep the versioned schema file with the application and point Arqen at it:

```bash
export ARQEN_THINGD_SCHEMA_PATH="$PWD/schema.thingd"
arqen thingd schema-validate schema.thingd --url http://127.0.0.1:8770
arqen thingd schema-remote http://127.0.0.1:8770
```

The remote Thingd service is authoritative for compatibility and migration
history. Arqen validates and reports; it does not silently apply migrations or
rewrite data. See [Thingd schema](./schema.md) for the complete workflow.

## 6. Verify the backend

Before deployment, run the application checks and inspect the actual storage
mode:

```bash
arqen check
arqen lint
arqen test
arqen build
arqen start --file arqen.toml
```

Then verify `/health`, `/ready`, `/agent/manifest`, storage connectivity,
queue workers, logs, and the schema report. Use the [production runbook](./production-runbook.md)
as the final checklist.

## What Arqen does not decide for you

Arqen does not invent your domain model, tenant ownership, authorization
policy, backup schedule, provider credentials, or live migration cutover.
Those decisions remain application and operations responsibilities. The
[application hardening](./application-hardening.md) guide explains the
boundaries that become important once multiple users or services share data.
