# thingd integration

<CurrentVersion kind="thingd" /> is the currently resolved Arqen dependency. It supplies objects, events,
search, links, durable queues, encryption-aware persistence, and a public
replication contract.

The first stable integration is Thingd’s public HTTP API. Arqen should not
import private thingd-cloud internals or require a Node.js SDK.

The adapter contract supports:

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
  +-- NativeThingdStore (embedded, implemented)
  +-- HttpThingdBackend (implemented; public-contract validation required)
  +-- CloudThingdBackend (optional, future)
```

Switching implementations must not change application domain services.

## Production considerations

Applications that need local durable thingd during development and hosted
thingd in production should use the same domain repository interfaces across
the storage modes. The reusable gaps to solve in Arqen are documented in
[application-hardening.md](application-hardening.md): scoped access,
conditional writes, idempotency, event cursors, HTTP contract validation, and
the optional public cloud adapter.

Arqen must not implement local/cloud synchronization itself, import private
thingd-cloud modules, or read cloud control-plane databases. The sync engine,
checkpoint semantics, conflict policy, tombstones, and transport belong to
Thingd. Arqen integrates the public HTTP capability and the native replication
endpoint. The initial production path is embedded native source to an HTTP
Thingd target.

## Adapter contract

See [adapter-contract.md](adapter-contract.md) for the full trait definition, data types, and implementation details.

Native durable and HTTP modes should be treated as deployment-specific paths
until recovery, timeout, retry, and compatibility tests have been run against
the target thingd version. Cloud hosting is not implemented by this package.

## <CurrentVersion kind="thingd" /> encryption, schemas, sync, and migration

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

The `arqen::thingd::sync` module is a typed HTTP client/worker over the current Thingd release's
`/v1/replication/events`, `/apply`, `/status`, `/conflicts`, and `/snapshot`
endpoints. It provides cursor checkpoints, bounded retries, collection
allowlists, idempotent replay, stale-cursor snapshot fallback, and graceful
shutdown. GET/status/snapshot reads are retryable; apply and schema mutation
requests are not retried automatically. Thingd remains responsible for
provenance, tombstones, conflict quarantine, and replication semantics. Sync is
opt-in and must be configured with explicit source/target credentials; Arqen
never transmits encryption keys or provider credentials.

`HttpThingdBackend` reuses pooled connections, applies explicit connect and
request timeouts, retries only safe read/transient failures, and bounds active
requests with `HttpClientPolicy::max_concurrency` (default `16`). Batch writes
group puts and deletes by collection to avoid one remote request per object.

### Supported deployment modes

```text
native local storage + no sync
native local storage + native Thingd source to HTTP target
HTTP Thingd source + HTTP Thingd Cloud replica
memory backend for tests or an explicitly configured cache
```

Native storage means one embedded Thingd engine and one durable data directory
inside the Arqen application process. It does not mean that Arqen starts a
second Thingd server against the same directory.

### What is recorded

The adapter and migration workflow cover these Thingd-owned record families:

- objects, including collection, stable ID, body, version, and timestamps;
- append-only events, including stream, type, payload, sequence, and
  idempotency metadata;
- queue jobs, including queue, payload, retry/lease state, and terminal state;
- links and search indexes through the adapter and destination-owned rebuild;
- replication records when explicitly included in a migration.

Arqen also records operational metadata such as checkpoints, sync results,
latency, retries, conflicts, and snapshot fallbacks through its metrics hooks.
Application audit history, user/tenant ownership, backups, and provider
credentials are intentionally outside Arqen’s storage integration.
