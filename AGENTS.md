# Arqen agent notes

Arqen is currently documentation-only. Do not add framework implementation until the design documents and public contracts are reviewed.

## Boundaries

- Arqen: reusable Rust backend patterns, CLI, agent tools, jobs, and adapters.
- thingd: object storage, events, search, links, and queues.
- thingd-cloud: hosted identity, tenancy, provisioning, billing, and optional customer APIs.
- Watchloom: reference application only; do not couple Arqen to Watchloom domain types.

## Rules

- Prefer Axum, Tokio, Tower, tracing, and explicit application state.
- Do not create a NestJS-like dependency-injection framework.
- Keep provider and cloud credentials server-side.
- Treat the public thingd HTTP API as the first integration boundary.
- Preserve in-memory and durable adapter parity.
- Add docs and contract tests before implementation features.
