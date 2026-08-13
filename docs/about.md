# About Arqen

Arqen is a Rust toolkit for building backend services with HTTP routes,
explicit application modules, typed tools, durable jobs, health checks, and
Thingd-backed data access.

It is useful for ordinary web and mobile backends, internal services,
automation systems, and products that expose selected operations to agents.
You can use the backend features without using an AI model or an agent
framework.

## What you build with Arqen

An Arqen application usually contains:

- HTTP routes and application state;
- modules that group related routes, tools, jobs, and health checks;
- authentication, authorization, and request validation;
- Thingd objects, events, search, links, and queues;
- workers for work that should continue after a request ends;
- health, readiness, structured logs, and operational metrics.

The application owns its domain models and business rules. Arqen supplies the
composition, runtime, and integration points around them.

## Implementation

The current implementation is Rust-first and uses Tokio, Tower, tracing,
Axum, and Thingd. The HTTP integration is feature-gated. The public data
integration uses Thingd’s HTTP API, while native Thingd can be embedded for
applications that need a local durable store.

Start with [Build an Arqen backend](./build-a-backend.md) to create a project,
add a module, choose storage, define a schema, and prepare a deployment.

## Agent support

Agent tools are one way to expose application capabilities. Arqen can describe
tool names, inputs, outputs, scopes, effects, and idempotency in a manifest so
people and software can discover the same operations. Read the
[Agent guide](./agent-guide.md) when your backend needs that workflow.
