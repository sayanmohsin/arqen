# Use cases

<MermaidDiagram type="use-cases" />

Arqen is useful whenever a backend needs explicit operations, durable data,
background work, or clear runtime checks.

## Product backends

Build APIs for web or mobile products with authentication, validation, domain
modules, storage adapters, health endpoints, and OpenAPI helpers.

**Start with:** [Build a backend](./build-a-backend.md) and
[Getting started](./getting-started.md).

## Agent-enabled operations

Expose selected application operations as typed tools. Describe inputs,
outputs, required scopes, read/write effects, and retry behavior in the agent
manifest. The same backend can continue serving normal HTTP clients.

**Start with:** [Typed tools](./typed-tools.md) and
[Agent guide](./agent-guide.md).

## Long-running automation

Move email, indexing, imports, synchronization, and enrichment out of request
handlers. Thingd queues provide leases, retries, idempotency metadata, and
dead-letter states for workers.

**Start with:** [Durable jobs](./durable-jobs.md).

## Local-first development

Use memory mode for fast tests and prototypes, then switch to native Thingd or
an HTTP Thingd service without changing domain repository interfaces.

**Start with:** [Configuration](./configuration.md) and
[Thingd integration](./thingd-integration.md).

## Human and machine operations

Use one HTTP service from dashboards, scripts, CI, applications, and agents.
Health, readiness, structured logs, and metrics make it easier to understand
whether the service and its dependencies are working.

**Start with:** [Health](./health.md), [Observability](./observability.md),
and [Production runbook](./production-runbook.md).
