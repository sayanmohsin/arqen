use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// An object stored in thingd.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThingdObject {
    /// Unique identifier within the collection.
    pub id: String,
    /// Collection (table/bucket) name.
    pub collection: String,
    /// Arbitrary JSON data.
    pub data: serde_json::Value,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// ISO 8601 last-update timestamp.
    pub updated_at: String,
}

/// An event appended to a thingd event stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThingdEvent {
    /// Unique event identifier.
    pub id: String,
    /// Stream name.
    pub stream: String,
    /// Event type (e.g., `"user.created"`).
    pub event_type: String,
    /// Arbitrary JSON payload.
    pub data: serde_json::Value,
    /// ISO 8601 timestamp.
    pub timestamp: String,
}

/// A job in a thingd queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThingdJob {
    /// Unique job identifier.
    pub id: String,
    /// Queue name.
    pub queue: String,
    /// Arbitrary JSON payload.
    pub payload: serde_json::Value,
    /// Current job state.
    pub state: JobState,
    /// Number of claim attempts so far.
    pub attempts: u32,
    /// Maximum retries before dead-lettering.
    pub max_retries: u32,
    /// ISO 8601 lease expiry (if currently leased).
    pub lease_expires_at: Option<String>,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// ISO 8601 last-update timestamp.
    pub updated_at: String,
}

/// State machine for a queue job.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobState {
    /// Waiting to be claimed.
    Queued,
    /// Currently being processed.
    Leased,
    /// Successfully completed.
    Completed,
    /// Failed, will be retried.
    Retrying,
    /// Permanently failed (dead-lettered).
    Dead,
}

/// A directed link between two objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThingdLink {
    /// Unique link identifier.
    pub id: String,
    /// Source object identifier.
    pub source_id: String,
    /// Target object identifier.
    pub target_id: String,
    /// Relation label (e.g., `"references"`, `"belongs_to"`).
    pub relation: String,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
}

/// A filter clause for object queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThingdFilter {
    /// Field name to filter on.
    pub field: String,
    /// Comparison operator.
    pub operator: FilterOperator,
    /// Value to compare against.
    pub value: serde_json::Value,
}

/// Comparison operator for filters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    /// Equals.
    Eq,
    /// Not equals.
    Ne,
    /// Greater than.
    Gt,
    /// Less than.
    Lt,
    /// Greater than or equal.
    Gte,
    /// Less than or equal.
    Lte,
    /// String contains.
    Contains,
}

/// Options for search and query operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Maximum number of results.
    pub limit: usize,
    /// Number of results to skip (pagination offset).
    pub offset: usize,
    /// Additional filters to apply.
    pub filters: Vec<ThingdFilter>,
}

/// Results from a search or query operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    /// Matching items.
    pub items: Vec<ThingdObject>,
    /// Total number of matching items (before pagination).
    pub total: usize,
}

/// A batch operation (put or delete).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThingdOperation {
    /// Insert or replace an object.
    Put {
        collection: String,
        id: String,
        data: serde_json::Value,
    },
    /// Delete an object.
    Delete { collection: String, id: String },
}

/// Result of a single batch operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThingdOperationResult {
    /// Whether the operation succeeded.
    pub success: bool,
    /// Error message if the operation failed.
    pub error: Option<String>,
}

/// Async trait for thingd storage backends.
///
/// Implement this trait to provide a custom storage backend. See
/// [`MemoryThingdBackend`](super::MemoryThingdBackend) for an in-memory
/// implementation.
#[async_trait]
pub trait ThingdBackend: Send + Sync {
    /// Get an object by collection and ID.
    async fn get_object(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<Option<ThingdObject>, crate::core::AppError>;

    /// Insert or replace an object.
    async fn put_object(
        &self,
        collection: &str,
        id: &str,
        data: serde_json::Value,
    ) -> Result<ThingdObject, crate::core::AppError>;

    /// Delete an object by collection and ID.
    async fn delete_object(&self, collection: &str, id: &str) -> Result<(), crate::core::AppError>;

    /// Query objects in a collection with optional filtering.
    async fn query_objects(
        &self,
        collection: &str,
        filter: Option<ThingdFilter>,
    ) -> Result<Vec<ThingdObject>, crate::core::AppError>;

    /// Count objects in a collection.
    async fn count_objects(&self, collection: &str) -> Result<usize, crate::core::AppError>;

    /// Execute a batch of put/delete operations.
    async fn batch_write(
        &self,
        operations: Vec<ThingdOperation>,
    ) -> Result<Vec<ThingdOperationResult>, crate::core::AppError>;

    /// Append an event to a stream.
    async fn append_event(
        &self,
        stream: &str,
        event_type: &str,
        data: serde_json::Value,
    ) -> Result<ThingdEvent, crate::core::AppError>;

    /// Read events from a stream.
    async fn read_events(
        &self,
        stream: &str,
        from: Option<String>,
        limit: usize,
    ) -> Result<Vec<ThingdEvent>, crate::core::AppError>;

    /// Push a job to a queue.
    async fn push_job(
        &self,
        queue: &str,
        payload: serde_json::Value,
        max_retries: u32,
    ) -> Result<ThingdJob, crate::core::AppError>;

    /// Claim a job from a queue for processing.
    async fn claim_job(
        &self,
        queue: &str,
        worker_id: &str,
        lease_seconds: u32,
    ) -> Result<Option<ThingdJob>, crate::core::AppError>;

    /// Mark a job as completed.
    async fn complete_job(&self, queue: &str, job_id: &str) -> Result<(), crate::core::AppError>;

    /// Negative-acknowledge a job (schedule for retry).
    async fn nack_job(&self, queue: &str, job_id: &str) -> Result<(), crate::core::AppError>;

    /// Dead-letter a job (permanent failure).
    async fn dead_letter_job(&self, queue: &str, job_id: &str)
    -> Result<(), crate::core::AppError>;

    /// Full-text search across all collections.
    async fn search(
        &self,
        query: &str,
        options: SearchOptions,
    ) -> Result<SearchResults, crate::core::AppError>;

    /// Create a directed link between two objects.
    async fn create_link(
        &self,
        source_id: &str,
        target_id: &str,
        relation: &str,
    ) -> Result<ThingdLink, crate::core::AppError>;

    /// Get links from a source object.
    async fn get_links(
        &self,
        source_id: &str,
        relation: Option<&str>,
    ) -> Result<Vec<ThingdLink>, crate::core::AppError>;

    /// Delete a link by ID.
    async fn delete_link(&self, link_id: &str) -> Result<(), crate::core::AppError>;

    /// Clear all data (for testing).
    async fn reset(&self) -> Result<(), crate::core::AppError>;

    /// Seed sample data (for testing).
    async fn seed(&self) -> Result<(), crate::core::AppError>;
}
