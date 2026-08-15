# thingd integration

<CurrentVersion kind="thingd" /> is the currently resolved Arqen dependency, pinned to released
Thingd version `0.83.2`. It supplies objects, events,
search, links, durable queues, encryption-aware persistence, and a public
replication contract. Thingd `0.83.2` replaced the legacy Fjall backend with
embedded RocksDB for production durable storage; the public REST/MCP/native
contracts are unchanged, but existing Fjall directories must be migrated
explicitly with `thingd-migrate` before they can be opened.

The first stable integration is Thingd’s public HTTP API. Arqen should not
import private thingd-cloud internals or require a Node.js SDK.

The adapter contract supports:

- typed object repositories;
- batch writes;
- append-only events;
- queue push, claim, ack, nack, and dead-letter operations;
- full-text and vector search when enabled;
- links for relationships.

The Arqen scheduler persists its records through the object contract and hands
off runs through the queue contract. Native Thingd `0.83.2` supports
deterministic queue IDs and delayed availability. The current public HTTP
queue endpoint exposes neither option, so HTTP scheduling returns an explicit
unsupported error for those operations; it never starts an in-memory timer.

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

### Catalog cache and startup bootstrap

`CachingThingdBackend::new_catalog` is the safe cache constructor for HTTP
deployments. It accepts an explicit collection allowlist and bypasses the
cache for every other collection. Configure it with
`ARQEN_THINGD_CACHE_ENABLED=true` and
`ARQEN_THINGD_CACHE_COLLECTIONS=catalog_titles,catalog_genres`. Never add
user-scoped collections to the allowlist.

Applications that seed data during startup can use
`arqen::seed_with_retry` with `BootstrapPolicy`. It retries transient
unavailable, timeout, and dependency errors with bounded exponential backoff;
seeding remains opt-in and is not started automatically by Arqen.

The HTTP adapter sends equality filters to the Thingd REST API. Range and
contains filters are applied by Arqen after it reads all bounded pages because
the current public REST list contract documents `filter.key=value` equality
parameters only. Arqen never silently drops unsupported filters: the scan is
bounded by `HttpClientPolicy::max_query_scan_objects`, and an exceeded bound
returns an explicit error. Revisit this fallback only when the deployed
Thingd server contract provides a tested range-filter representation.

### Thingd 0.83.2 persistent asynchronous search

Thingd 0.83.2 provides coalesced asynchronous Tantivy indexing while RocksDB
remains the durable source of truth. For an HTTP Thingd deployment, configure
the Thingd service (not Arqen) with:

```env
THINGD_SEARCH_MODE=persistent-async
THINGD_SEARCH_COMMIT_INTERVAL_MS=250
THINGD_SEARCH_COMMIT_BATCH_SIZE=32
THINGD_SEARCH_QUEUE_MAX_KEYS=10000
```

Search is eventually consistent after a successful write. Applications should
retry search-after-write reads with bounded backoff or use the primary object
read until the indexed result appears. Arqen does not run a separate Tantivy
maintenance loop.

Thingd 0.83.2 also adds bounded large-journal recovery for low-memory hosts:
recovery runs in two phases (primary RocksDB recovery/compaction, then Tantivy
search rebuild in bounded batches). During recovery `/ready` and mutation
endpoints return `503 Retry-After: 1`, reads remain available, and compatible
search indexes are reused without a rebuild on normal restarts. Arqen's HTTP
client retries bounded mutations and reads with `ARQEN_THINGD_MAX_RETRIES` and
`ARQEN_THINGD_MAX_RETRY_DURATION`. A `503` with `Retry-After: 1` is retried
for object, batch, event, and queue mutations, catalog bootstrap, and
synchronization. Mutation requests carry a stable per-operation idempotency
key across attempts.

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

### Migrating from Fjall to RocksDB (Thingd 0.83.2)

Thingd 0.83.2 cannot open an existing Fjall directory. Opening one fails closed
with `UnsupportedStorageFormat`. The migration is a logical copy into a new
RocksDB directory and is performed offline, before Arqen starts against the
new path:

1. Stop Arqen and every process that uses the existing database.
2. Identify the configured persistent directory (an old Fjall store).
3. Choose a separate destination path that does not exist and is not inside
   the source directory.
4. Run the Thingd migration utility (built from the thingd repository):

   ```bash
   cargo run -p thingd-migrate -- fjall-to-rocksdb \
     --source <existing-fjall-path> \
     --destination <new-rocksdb-path>
   ```

5. Never reuse the source path as the destination and never modify, delete, or
   overwrite the Fjall source.
6. Point Arqen at the new RocksDB path (for native mode, `ARQEN_PERSISTENT_PATH`
   or `ARQEN_NATIVE_DATA_DIR`), then start Arqen/Thingd and validate: `/healthz`
   and `/ready` return `200`, diagnostics report an idle/healthy maintenance
   state, object counts and IDs are preserved, versions/timestamps are
   preserved, events and sequence state survive, queues/leases/retries/dead
   letters survive, links, schemas, migrations, idempotency, and replication
   state are preserved, search rebuild completes, and representative reads and
   writes succeed after a restart.
7. Keep the original Fjall directory and a backup until the migrated store has
   passed restart and production validation.

Do not seed or write data while Thingd is unavailable during migration; retry
with bounded backoff. In HTTP mode the backend talks to a standalone Thingd
server over REST, so only the sidecar's data directory needs migration. The
`thingd-migrate` binary is an offline utility owned by Thingd; Arqen does not
embed Fjall or perform the conversion itself.

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
