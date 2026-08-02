# FAQ

## Is Arqen an AI framework?

No. It is backend infrastructure. “Agent-ready” describes discoverable,
typed, permission-aware, auditable, and automation-friendly interfaces that
are useful to humans and software too.

## Is Arqen Rust-only?

The implementation is Rust-first today. The public positioning is
language-agnostic, with a future Node.js path through HTTP APIs, SDKs,
templates, and shared manifests.

## Is cloud hosting available?

Not as a committed Arqen feature. The cloud adapter is future work and depends
on a public thingd-cloud customer contract.

## Which storage mode should I use?

Use memory mode for local development and tests. Evaluate the native durable
and HTTP sidecar paths against the current feature status before production use.

## Does `arqen dev` hot reload?

Not internally yet. It starts the server; the current documented watcher loop
uses `cargo watch` separately.
