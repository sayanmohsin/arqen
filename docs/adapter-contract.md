# Thingd Adapter Contract

The thingd adapter provides a unified interface for storage, events, search, links, and queues.

## Adapter trait

```rust
#[async_trait]
pub trait ThingdBackend: Send + Sync {
    // Object operations
    async fn get_object(&self, collection: &str, id: &str) -> Result<Option<ThingdObject>>;
    async fn put_object(&self, collection: &str, id: &str, data: serde_json::Value) -> Result<ThingdObject>;
    async fn delete_object(&self, collection: &str, id: &str) -> Result<()>;
    async fn query_objects(&self, collection: &str, filter: ThingdFilter) -> Result<Vec<ThingdObject>>;
    
    // Batch operations
    async fn batch_write(&self, operations: Vec<ThingdOperation>) -> Result<Vec<ThingdOperationResult>>;
    
    // Event operations
    async fn append_event(&self, stream: &str, event: ThingdEvent) -> Result<ThingdEvent>;
    async fn read_events(&self, stream: &str, from: Option<String>, limit: usize) -> Result<Vec<ThingdEvent>>;
    
    // Queue operations
    async fn push_job(&self, queue: &str, job: ThingdJob) -> Result<ThingdJob>;
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

### HttpThingdBackend

- Connects to thingd service via HTTP
- Uses thingd's public REST API
- Suitable for production deployments
- Requires network connectivity

### CloudThingdBackend (future)

- Optional hosted thingd-cloud integration
- Uses documented public customer API
- Preserves direct thingd adapter interface
- Feature-gated package

## Switching implementations

Switching between implementations must not change application domain services. The adapter is injected at application startup:

```rust
let thingd: Box<dyn ThingdBackend> = match config.storage_mode {
    "memory" => Box::new(MemoryThingdBackend::new()),
    "http" => Box::new(HttpThingdBackend::new(&config.thingd_url)?),
    _ => return Err(ConfigError::InvalidStorageMode),
};

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