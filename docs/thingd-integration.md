# thingd integration

thingd 0.77 is a first-class Arqen dependency. It supplies objects, events,
search, links, durable queues, encryption-aware persistence, and a public
replication contract.

The first stable boundary is thingd's public HTTP API. Arqen should not import private thingd-cloud internals or require a Node.js SDK.

The adapter boundary should support:

- typed object repositories;
- batch writes;
- append-only events;
- queue push, claim, ack, nack, and dead-letter operations;
- full-text and vector search when enabled;
- links for relationships.

Implemented and planned adapter paths:

```text
ThingdBackend
  +-- MemoryThingdBackend (implemented)
  +-- NativeThingdStore (implemented)
  +-- HttpThingdBackend (implemented; public-contract validation required)
  +-- CloudThingdBackend (optional, future)
```

Switching implementations must not change application domain services.

## Hardening boundary

Applications that need local durable thingd during development and hosted
thingd in production should use the same domain repository interfaces across
the storage modes. The reusable gaps to solve in Arqen are documented in
[application-hardening.md](application-hardening.md): scoped access,
conditional writes, idempotency, event cursors, HTTP contract validation, and
the optional public cloud adapter.

Arqen must not implement local/cloud synchronization itself, import private
thingd-cloud modules, or read cloud control-plane databases. The sync engine,
checkpoint semantics, conflict policy, tombstones, and transport belong to
thingd. Arqen should integrate the thingd-provided capability through a
versioned public API and provide configuration, identity, lifecycle, health,
metrics, and capability discovery around it.

## Adapter contract

See [adapter-contract.md](adapter-contract.md) for the full trait definition, data types, and implementation details.

Native durable and HTTP modes should be treated as deployment-specific paths
until recovery, timeout, retry, and compatibility tests have been run against
the target thingd version. Cloud hosting is not implemented by this package.

## Thingd 0.77 encryption, schemas, and sync

Native storage accepts a 32-byte encryption key as 64 hexadecimal characters
through `ARQEN_THINGD_ENCRYPTION_KEY`. Arqen passes this to Thingd's
`PersistentOpenOptions`; an invalid or missing configured key is a startup
error and never falls back to memory. Keys are wrapped in `Secret<T>` and are
not serialized or logged.

Arqen can load a versioned `.thingd` file with `ARQEN_THINGD_SCHEMA_PATH` and
reports a stable source hash. The authoritative parser remains Thingd's
`/v1/schema/validate` endpoint because the standalone `thingd-schema` crate is
not yet a published dependency. Use:

```bash
arqen thingd schema-validate schema.thingd --url http://localhost:8770
arqen thingd schema-remote http://localhost:8770
```

Schema migration application is deliberately not automatic. Operators should
inspect the remote migration history and use Thingd's supported migration
workflow; Arqen will not delete or rewrite data to make a schema fit.

The `arqen::thingd::sync` module is a typed client/worker over Thingd 0.77's
`/v1/replication/events`, `/apply`, `/status`, `/conflicts`, and `/snapshot`
endpoints. It provides cursor checkpoints, bounded retries, collection
allowlists, idempotent replay, stale-cursor snapshot fallback, and graceful
shutdown. Thingd remains responsible for provenance, tombstones, conflict
quarantine, and replication semantics. Sync is opt-in and must be configured
with explicit source/target credentials; Arqen never transmits encryption keys
or provider credentials.
