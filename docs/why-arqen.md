# Why Arqen?

Backend systems become difficult to automate when their capabilities are
hidden in untyped routes, implicit dependencies, or operational folklore.
Arqen makes those boundaries explicit.

## The useful property

An agent-ready service is discoverable, typed, permission-aware, auditable, and
automation-friendly. Those properties also improve ordinary developer and
operator workflows.

Arqen combines:

- typed tools and manifests for capability discovery;
- explicit state and adapter contracts for predictable composition;
- jobs with retries, leases, idempotency, and dead-letter handling;
- health, readiness, structured logging, and deployment guidance;
- thingd integration without coupling application code to private cloud APIs.

Arqen is not an AI-only runtime and does not prescribe a model provider.

## How Arqen differs

Arqen is not trying to win by being another application framework, agent
runtime, hosted database, or workflow product. Its distinct layer is the
contract between an application and the people, programs, and agents that need
to discover and operate it.

| If you start with… | The usual center of gravity | Arqen adds or changes |
|---|---|---|
| A web framework | Routes, handlers, and middleware | Typed tools, manifests, permissions, jobs, health, and audit signals are part of the backend contract. |
| An agent framework | Models, prompts, and orchestration | The application remains model-agnostic; agents consume explicit capabilities over HTTP. |
| A BaaS | Hosted data, auth, and dashboards | Deployment modes and adapter boundaries stay visible, with no hosted control plane required by the design. |
| A workflow engine | Durable execution and retries | Jobs sit beside the HTTP API, storage adapter, tool registry, logs, and readiness surface. |
| A microservice stack | Many independently deployed services | Start with one explicit application boundary, then introduce an HTTP sidecar or cloud adapter when the boundary earns it. |

The result is intentionally compositional: Arqen can live inside a normal web
service, expose capabilities to an agent, delegate persistence to thingd, and
run jobs without turning the whole system into an AI runtime or a distributed
workflow graph.

## The honest boundary

This is an architectural distinction, not a claim that every path is mature.
Arqen is still early-stage; native durable thingd migration, public HTTP parity,
and broader CLI/template work remain active gates. Check the
[feature status](feature-status.md) before treating a planned capability as
available.
