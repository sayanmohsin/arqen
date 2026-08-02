# Use cases

## Tool-enabled product backends

Expose a small, typed capability surface to an agent while keeping domain
logic in ordinary application services. Manifests describe names, inputs,
permissions, and audit expectations.

## Long-running automation

Move email, indexing, synchronization, and other work out of request paths.
Use durable job semantics where retries and idempotency matter.

## Local-first service development

Start with memory mode, health endpoints, structured logs, and an explicit
application state. Move toward durable thingd without rewriting domain code.

## Human and machine operations

Use the same discoverable HTTP surface for dashboards, scripts, CI, and agents.
Readiness and logs make deployment behavior visible to operators.

## Language-diverse clients

Keep the server contracts language-agnostic. Rust is the first implementation;
Node.js clients and templates can consume the HTTP API and shared manifests as
those deliverables mature.
