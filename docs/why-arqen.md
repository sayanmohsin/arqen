# Why Arqen?

Backend code becomes harder to operate when routes, storage, background work,
and authorization are scattered across implicit conventions. Arqen puts those
pieces in explicit application modules and gives them consistent runtime
support.

## Common problems

### “The API works, but nobody can discover what it does”

Define typed tools and manifests with names, inputs, outputs, scopes, effects,
and idempotency behavior. A human client, script, or agent can inspect the
same description before calling an operation.

See [Typed tools](./typed-tools.md), [Agent discovery](./agent-discovery.md),
and [Manifest contract](./manifest.md).

### “Background work is mixed into request handlers”

Use Thingd-backed queues and workers for email, indexing, synchronization, and
other work that needs retries, leases, idempotency, or dead-letter handling.

See [Durable jobs](./durable-jobs.md).

### “Changing storage changes application code”

Use the `ThingdBackend` contract so domain services can work with memory,
native Thingd, or an HTTP Thingd service. Start locally with memory mode and
move to durable storage after the application shape is clear.

See [Thingd integration](./thingd-integration.md) and
[Configuration](./configuration.md).

### “Production behavior is hidden in startup scripts”

Use layered configuration, strict production validation, health and readiness
checks, structured logs, and explicit worker settings. The application still
owns deployment policy, secrets, backups, and tenant isolation, but the checks
and integration points are visible in the codebase.

See [Deployment](./deployment.md) and the
[Production runbook](./production-runbook.md).

## When Arqen fits

Arqen fits a backend that needs a clear path from a small local service to a
durable deployment. It can sit alongside a model runtime, a frontend, a
workflow system, or another service without requiring any one of them.

It is not a model provider or a hosted database. Thingd provides the data
engine; the application provides domain behavior; Arqen connects the HTTP,
module, job, health, and storage pieces.

## Current limits

Review [Feature status](./feature-status.md) before relying on a capability in
production. Cloud storage, workload-specific recovery, key lifecycle, tenant
policy, and backup operations require application and deployment decisions.
