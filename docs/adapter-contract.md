# Thingd Adapter Contract

The Thingd adapter provides a unified interface for storage, events, search,
links, and queues. The stable application contract is owned by Arqen and does
not require an adapter to import the Thingd Rust crate. The public trait also
includes compatibility probing, deterministic/delayed queue insertion, and
`reset`/`seed` test helpers.

## Adapter trait

```rust
#[async_trait]
pub trait ThingdBackend: Send + Sync {
    async fn check_compatibility(&self) -> Result<ThingdCompatibilityReport>;

    // Object operations
    async fn get_object(&self, collection: &str, id: &str) -> Result<Option<ThingdObject>>;
    async fn put_object(&self, collection: &str, id: &str, data: serde_json::Value) -> Result<ThingdObject>;
    async fn delete_object(&self, collection: &str, id: &str) -> Result<()>;
    async fn query_objects(&self, collection: &str, options: QueryOptions) -> Result<Vec<ThingdObject>>;
    async fn count_objects(&self, collection: &str) -> Result<usize>;

    // Batch operations
    async fn batch_write(&self, operations: Vec<ThingdOperation>) -> Result<Vec<ThingdOperationResult>>;

    // Event operations
    async fn append_event(&self, stream: &str, event_type: &str, data: serde_json::Value) -> Result<ThingdEvent>;
    async fn read_events(&self, stream: &str, from: Option<String>, limit: usize) -> Result<Vec<ThingdEvent>>;

    // Queue operations
    async fn push_job(&self, queue: &str, payload: serde_json::Value, max_retries: u32) -> Result<ThingdJob>;
    async fn push_job_with_options(&self, queue: &str, payload: serde_json::Value, max_retries: u32, options: PushJobOptions) -> Result<ThingdJob>;
    async fn claim_job(&self, queue: &str, worker_id: &str, lease_seconds: u32) -> Result<Option<ThingdJob>>;
    async fn complete_job(&self, queue: &str, job_id: &str) -> Result<()>;
    async fn nack_job(&self, queue: &str, job_id: &str) -> Result<()>;
    async fn dead_letter_job(&self, queue: &str, job_id: &str) -> Result<()>;

    // Search operations
    async fn search(&self, query: &str, options: SearchOptions) -> Result<SearchResults>;

    // Link operations
    async fn create_link(&self, source_id: &str, target_id: &str, relation: &str) -> Result<ThingdLink>;
    async fn get_links(&self, source_id: &str, relation: Option<&str>) -> Result<Vec<ThingdLink>>;
    async fn delete_link(&self, link_id: &str) -> Result<()>;

    async fn reset(&self) -> Result<()>;
    async fn seed(&self) -> Result<()>;
}
```

## Implementations

### MemoryThingdBackend

- In-memory storage using HashMaps
- Process-local and disposable
- Suitable for development and testing
- No external dependencies

### `thingd-native` / `NativeThingdBackend`

- Adapts embedded native thingd to the common async `ThingdBackend` contract
- Construct it through `StorageFactory` when the `thingd-native` feature is enabled
- `NativeThingdStore` remains available for advanced full-native APIs
- The feature accepts Thingd <CurrentVersion kind="native-thingd" :label="false" />.
- `thingd-maintenance` exposes optional native diagnostics, validation,
  compaction, and bounded search-rebuild operations.
- `thingd-connectors` exposes Thingd's native connector traits and built-in
  connector types. It does not change `ThingdBackend` or the HTTP adapter.

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
- `check_compatibility()` validates the versioned `/v1/health` contract before
  the application marks the remote dependency ready
- It reports API compatibility, not an arbitrary concrete Thingd engine
  version, because the public health response does not expose one

### <CurrentVersion kind="http-api" /> synchronization

With the `http-client` feature, `ThingdSyncClient` and `ThingdSyncWorker` wrap
Thingd's public `/v1/replication/events`, `/apply`, `/status`, `/conflicts`, and
`/snapshot` endpoints. `SyncCheckpointStore` lets applications persist the
last applied cursor in their selected backend. The worker supports bounded
retry, collection allowlists, stale-cursor snapshot bootstrap, and graceful
shutdown. It is opt-in and experimental; Thingd remains the owner of
replication semantics, provenance, tombstones, and conflict quarantine.

Native storage is embedded in the Arqen process and does not require a local
sidecar. Native-only migration and full Thingd APIs are enabled by
`thingd-native` and `thingd-migration`; the default Arqen build does not
compile the Thingd Rust crate. HTTP replication remains the public runtime
integration.

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

For an HTTP deployment, validate the remote contract during startup:

```rust
let backend = HttpThingdBackend::new("https://thingd.internal");
backend.check_compatibility().await?;
let state = AppState::builder()
    .with_storage(std::sync::Arc::new(backend))
    .build()?;
```

Native storage is an explicit optional feature. Its Thingd dependency accepts
the supported compatible range and must pass the native contract suite after
each lockfile update. Incompatible Thingd minor releases require an Arqen
compatibility change.

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
