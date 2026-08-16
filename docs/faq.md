# FAQ

## What is Arqen?

Arqen is backend infrastructure for HTTP services. It provides application
composition, configuration, authentication helpers, validation, Thingd
adapters, durable jobs, health checks, observability, OpenAPI helpers, and an
optional agent-tool layer.

## Do I need to use AI?

No. Agent tools are optional. You can use Arqen for a conventional web or
mobile backend, internal service, or automation worker.

## Which storage mode should I use?

- Use `memory` for tests, examples, and disposable local development.
- Use `native` only for local development or migration, with Arqen’s optional
  `thingd-native` feature.
- Use `http` when Thingd should run as a separate service with its own memory,
  schema, index, backup, and operational lifecycle.

Native mode is compile-time coupled to the supported Thingd Cargo range.
Compatible patch updates do not require an Arqen release. HTTP mode is coupled
to the public Thingd `v1` API;
call `HttpThingdBackend::check_compatibility()` during startup and fail
readiness if it returns an error.

- Do not use `cloud` as a current production mode; the public cloud adapter is
  not implemented.

See [Configuration](./configuration.md) and
[Deployment](./deployment.md).

## Do I need to use Rust in my client?

The Arqen implementation is Rust-first. Clients can call the HTTP API from
other languages. Machine-readable manifests are available for clients and
agents; broader SDKs and templates are future work.

## What does Thingd store?

Through the adapter, Arqen works with objects, events, search, links, and
queues. Thingd owns its durable storage, replication semantics, tombstones,
and conflict handling. The application owns domain meaning, authorization,
tenant ownership, backups, and audit requirements.

See [Thingd integration](./thingd-integration.md).

## How do I use a schema?

Keep a versioned `.thingd` file with the application, validate it with
`arqen thingd schema-validate`, and inspect the remote schema and migration
history with `arqen thingd schema-remote`. Applying a migration is an explicit
operator action; it does not happen automatically at startup.

See [Thingd schema](./schema.md).

## Is cloud hosting available?

Not through the current Arqen package. Use HTTP Thingd for a separate data
service and evaluate any hosted integration against a versioned public
contract.

## Does `arqen dev` hot reload?

No. It starts the server with development settings. Use an external watcher,
such as `cargo watch`, if the project needs automatic restarts.
