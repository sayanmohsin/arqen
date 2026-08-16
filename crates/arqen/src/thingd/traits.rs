use async_trait::async_trait;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::core::{AppError, ErrorKind};

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
    /// Unix timestamp in milliseconds when this job becomes claimable.
    pub available_at_ms: Option<i64>,
}

/// Options for deterministic and delayed queue insertion.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PushJobOptions {
    /// Stable job ID used as the idempotency key when supported by Thingd.
    pub idempotency_key: Option<String>,
    /// Delay before the job becomes claimable.
    pub delay_ms: Option<u64>,
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

/// Apply one filter clause using the shared backend contract.
pub(crate) fn matches_filter(
    object: &ThingdObject,
    filter: &ThingdFilter,
) -> Result<bool, AppError> {
    let Some(value) = object.data.get(&filter.field) else {
        return Ok(false);
    };
    match filter.operator {
        FilterOperator::Eq => Ok(*value == filter.value),
        FilterOperator::Ne => Ok(*value != filter.value),
        FilterOperator::Contains => match (value.as_str(), filter.value.as_str()) {
            (Some(value), Some(needle)) => Ok(value.contains(needle)),
            _ => Err(invalid_filter(
                &filter.field,
                "Contains requires string values",
            )),
        },
        FilterOperator::Gt | FilterOperator::Lt | FilterOperator::Gte | FilterOperator::Lte => {
            let ordering = compare_filter_values(value, &filter.value, &filter.field)?;
            Ok(match filter.operator {
                FilterOperator::Gt => ordering == Ordering::Greater,
                FilterOperator::Lt => ordering == Ordering::Less,
                FilterOperator::Gte => ordering != Ordering::Less,
                FilterOperator::Lte => ordering != Ordering::Greater,
                _ => unreachable!(),
            })
        }
    }
}

/// Apply conjunctive filters without silently treating unsupported values as
/// matching every object.
pub(crate) fn filter_objects(
    objects: Vec<ThingdObject>,
    filters: &[ThingdFilter],
) -> Result<Vec<ThingdObject>, AppError> {
    objects
        .into_iter()
        .filter_map(|object| {
            let result = filters.iter().try_fold(true, |matches, filter| {
                if matches {
                    matches_filter(&object, filter)
                } else {
                    Ok(false)
                }
            });
            match result {
                Ok(true) => Some(Ok(object)),
                Ok(false) => None,
                Err(error) => Some(Err(error)),
            }
        })
        .collect()
}

fn compare_filter_values(
    value: &serde_json::Value,
    filter_value: &serde_json::Value,
    field: &str,
) -> Result<Ordering, AppError> {
    if let (Some(left), Some(right)) = (value.as_f64(), filter_value.as_f64()) {
        return left
            .partial_cmp(&right)
            .ok_or_else(|| invalid_filter(field, "numeric values must be finite"));
    }
    if let (Some(left), Some(right)) = (value.as_str(), filter_value.as_str()) {
        if let (Ok(left), Ok(right)) = (
            DateTime::parse_from_rfc3339(left),
            DateTime::parse_from_rfc3339(right),
        ) {
            return Ok(left.cmp(&right));
        }
        return Ok(left.cmp(right));
    }
    Err(invalid_filter(
        field,
        "range comparisons require two numbers or two strings",
    ))
}

fn invalid_filter(field: &str, message: &str) -> AppError {
    AppError::new(
        ErrorKind::Validation,
        format!("invalid filter for field '{field}': {message}"),
    )
}

/// Options for search and query operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Maximum number of results.
    pub limit: usize,
    /// Number of results to skip (pagination offset).
    pub offset: usize,
    /// Additional filters to apply after text matching and before pagination.
    /// Invalid filter values return a validation error.
    pub filters: Vec<ThingdFilter>,
}

/// Options for querying objects within a single collection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryOptions {
    /// Filters to apply conjunctively (all must match).
    pub filters: Vec<ThingdFilter>,
    /// Maximum number of results. `None` returns all matches.
    pub limit: Option<usize>,
    /// Number of results to skip (pagination offset).
    pub offset: usize,
}

impl QueryOptions {
    /// Create options that apply the given conjunctive filters with no pagination.
    pub fn filtered(filters: Vec<ThingdFilter>) -> Self {
        Self {
            filters,
            limit: None,
            offset: 0,
        }
    }
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

/// Result of validating a Thingd backend boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThingdCompatibilityReport {
    /// API or adapter boundary that was validated.
    pub api_version: String,
    /// Health status returned by the boundary.
    pub status: String,
}

/// Async trait for thingd storage backends.
///
/// Implement this trait to provide a custom storage backend. See
/// [`MemoryThingdBackend`](super::MemoryThingdBackend) for an in-memory
/// implementation.
#[async_trait]
pub trait ThingdBackend: Send + Sync {
    /// Validate that this backend is available and compatible.
    ///
    /// Local adapters are compatible once constructed. Remote adapters should
    /// override this to probe their versioned service boundary.
    async fn check_compatibility(
        &self,
    ) -> Result<ThingdCompatibilityReport, crate::core::AppError> {
        Ok(ThingdCompatibilityReport {
            api_version: "local".to_string(),
            status: "ok".to_string(),
        })
    }

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

    /// Query objects in a collection.
    ///
    /// Filters are applied conjunctively (all must match). `Eq` may be sent to
    /// Thingd server-side; backends apply `Ne`, `Gt`, `Lt`, `Gte`, `Lte`, and
    /// `Contains` client-side when the server contract does not support them.
    /// Range comparisons support numbers and strings, including RFC3339
    /// timestamps. Invalid or unsupported filter values return a validation
    /// error rather than returning unfiltered data. `limit`/`offset` paginate
    /// after filtering. Pass [`QueryOptions::default()`] for all objects.
    async fn query_objects(
        &self,
        collection: &str,
        options: QueryOptions,
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

    /// Push a job with deterministic identity and optional delayed visibility.
    async fn push_job_with_options(
        &self,
        queue: &str,
        payload: serde_json::Value,
        max_retries: u32,
        options: PushJobOptions,
    ) -> Result<ThingdJob, crate::core::AppError> {
        let _ = options;
        self.push_job(queue, payload, max_retries).await
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn object(value: serde_json::Value) -> ThingdObject {
        ThingdObject {
            id: "one".into(),
            collection: "items".into(),
            data: value,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn range_filters_compare_rfc3339_and_boundaries() {
        let value = object(json!({"expiresAt": "2026-08-10T12:00:00Z", "price": 10}));
        for (operator, expected) in [
            (FilterOperator::Lt, true),
            (FilterOperator::Lte, true),
            (FilterOperator::Gt, false),
            (FilterOperator::Gte, false),
        ] {
            let filter = ThingdFilter {
                field: "expiresAt".into(),
                operator,
                value: json!("2026-08-11T12:00:00+00:00"),
            };
            assert_eq!(matches_filter(&value, &filter).unwrap(), expected);
        }
        let equal = ThingdFilter {
            field: "price".into(),
            operator: FilterOperator::Lte,
            value: json!(10),
        };
        assert!(matches_filter(&value, &equal).unwrap());
    }

    #[test]
    fn invalid_range_filter_fails_loudly() {
        let filter = ThingdFilter {
            field: "price".into(),
            operator: FilterOperator::Lt,
            value: json!("not-a-number"),
        };
        let error = matches_filter(&object(json!({"price": 10})), &filter).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
    }
}
