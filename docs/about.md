# About Arqen

Arqen is backend infrastructure for agent-ready applications: a set of
explicit Rust-first building blocks for HTTP services, typed tools, jobs,
observability, and thingd-backed data access.

The intended audience is developers who want an application boundary that is
easy to inspect and operate. Arqen does not require an AI model, an agent
framework, or a hosted control plane.

## Positioning

- **Public tagline:** Backend infrastructure for agent-ready applications.
- **Public description:** A developer-focused backend toolkit for agent-ready applications, with typed tools, durable jobs, discoverable APIs, and thingd integration.
- **Implementation:** Rust-first, using Arqen’s APIs over Tokio, Tower, tracing,
  and native thingd. The HTTP integration is feature-gated and built on Axum.

Applications can use Arqen’s HTTP helpers and re-exported integration types,
or compose the lower-level HTTP layer directly when they need more control.

- **Compatibility direction:** Node.js through HTTP APIs, SDKs, templates, and shared manifests.

The separation matters: Rust describes today’s implementation, not a language
requirement for every future Arqen application.
