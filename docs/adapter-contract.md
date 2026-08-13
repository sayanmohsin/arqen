# Thingd Adapter Contract

The thingd adapter provides a unified interface for storage, events, search, links, and queues.
The public Rust API also includes `reset` and `seed` test helpers. The exact
trait signatures below should be checked against the versioned Rust API when
implementing a third-party adapter.

## Adapter trait

```rust
#[async_trait]
pub trait ThingdBackend: Send + Sync {
    // Object operations
    async fn get_object(&self, collection: &str, id: &str) -> Result<Option<ThingdObject>>;
    async fn put_object(&self, collection: &str, id: &str, data: serde_json::Value) -> Result<ThingdObject>;
    async fn delete_object(&self, collection: &str, id: &str) -> Result<()>;
    async fn query_objects(&self, collection: &str, options: QueryOptions) -> Result<Vec<ThingdObject>>;

    // Batch operations
    async fn batch_write(&self, operations: Vec<ThingdOperation>) -> Result<Vec<ThingdOperationResult>>;

    // Event operations
    async fn append_event(&self, stream: &str, event_type: &str, data: serde_json::Value) -> Result<ThingdEvent>;
    async fn read_events(&self, stream: &str, from: Option<String>, limit: usize) -> Result<Vec<ThingdEvent>>;

    // Queue operations
    async fn push_job(&self, queue: &str, payload: serde_json::Value, max_retries: u32) -> Result<ThingdJob>;
    async fn claim_job(&self, queue: &str, worker_id: &str, lease_seconds: u32) -> Result<Option<ThingdJob>>;
    async fn complete_job(&self, queue: &str, job_id: &str) -> Result<()>;
    async fn nack_job(&self, queue: &str, job_id: &str) -> Result<()>;
    async fn dead_letter_job(&self, queue: &str, job_id: &str) -> Result<()>;

    // Search operations
    async fn search(&self, query: &str, options: SearchOptions) -> Result<SearchResults>;

    // Link operations
    async fn create_link(&self, link: ThingdLink) -> Result<ThingdLink>;
    async fn get_links(&self, source_id: &str, relation: Option<&str>) -> Result<Vec<ThingdLink>>;
    async fn delete_link(&self, link_id: &str) -> Result<()>;
}
```

## Implementations

### MemoryThingdBackend

- In-memory storage using HashMaps
- Process-local and disposable
- Suitable for development and testing
- No external dependencies

### NativeThingdBackend

- Adapts embedded native thingd to the common async `ThingdBackend` contract
- Construct it through `StorageFactory` for configured memory/native storage
- `NativeThingdStore` remains available for advanced full-native APIs

### CachingThingdBackend

- Optional read-through decorator; it is never inserted automatically
- Configurable TTL and capacity
- Invalidates objects on writes, deletes, and batch writes
- Exposes cache hit/miss counters
- `new_catalog` restricts reads to an explicit collection allowlist and is the
  supported path for HTTP production deployments

### Startup bootstrap

`BootstrapPolicy`, `retry_bootstrap`, and `seed_with_retry` provide bounded,
retry-aware startup operations for remote Thingd readiness. Applications own
the seed contents and decide when to invoke them.

HTTP queries have a bounded client-side scan budget for unsupported range
filters. Configure it through `HttpClientPolicy::max_query_scan_objects` or
`ARQEN_THINGD_MAX_QUERY_SCAN_OBJECTS`. Large result producers can use
`jsonl_response` to stream newline-delimited JSON without materializing the
full collection.

### HttpThingdBackend

- Connects to thingd service via HTTP
- Uses thingd's public REST API
- Suitable for production deployments
- Requires network connectivity

### <CurrentVersion kind="thingd" /> synchronization

With the `http-client` feature, `ThingdSyncClient` and `ThingdSyncWorker` wrap
Thingd's public `/v1/replication/events`, `/apply`, `/status`, `/conflicts`, and
`/snapshot` endpoints. `SyncCheckpointStore` lets applications persist the
last applied cursor in their selected backend. The worker supports bounded
retry, collection allowlists, stale-cursor snapshot bootstrap, and graceful
shutdown. It is opt-in and experimental; Thingd remains the owner of
replication semantics, provenance, tombstones, and conflict quarantine.

Native storage is embedded in the Arqen process and does not require a local
sidecar. `NativeThingdSyncEndpoint` uses the current Thingd release's public
`ReplicationService`; it never reads private Thingd internals or falls back to
HTTP or memory. Use Arqen's replication-aware native mutation helpers so
successful object and event writes create source feed records.

### CloudThingdBackend (future)

- Optional hosted thingd-cloud integration
- Uses documented public customer API
- Preserves direct thingd adapter interface
- Feature-gated package

## Switching implementations

Switching between implementations must not change application domain services. The adapter is injected at application startup:

```rust
let thingd = StorageFactory::build(&config)?;

let app_state = AppState::new(thingd);
```

## Data types

### ThingdObject

```json
{
  "id": "string",
  "collection": "string",
  "data": {},
  "created_at": "2026-08-01T00:00:00Z",
  "updated_at": "2026-08-01T00:00:00Z"
}
```

### ThingdEvent

```json
{
  "id": "string",
  "stream": "string",
  "type": "string",
  "data": {},
  "timestamp": "2026-08-01T00:00:00Z"
}
```

### ThingdJob

```json
{
  "id": "string",
  "queue": "string",
  "payload": {},
  "state": "queued | leased | completed | retrying | dead",
  "attempts": 0,
  "max_retries": 3,
  "lease_expires_at": "2026-08-01T00:00:00Z",
  "created_at": "2026-08-01T00:00:00Z",
  "updated_at": "2026-08-01T00:00:00Z"
}
```

### ThingdLink

```json
{
  "id": "string",
  "source_id": "string",
  "target_id": "string",
  "relation": "string",
  "created_at": "2026-08-01T00:00:00Z"
}
```
