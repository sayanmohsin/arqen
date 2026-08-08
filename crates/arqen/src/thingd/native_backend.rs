use std::path::Path;

use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

use crate::core::{AppError, ErrorKind};
use crate::thingd::NativeThingdStore;
use crate::thingd::{
    FilterOperator, QueryOptions, SearchOptions, SearchResults, ThingdBackend, ThingdEvent,
    ThingdFilter, ThingdJob, ThingdLink, ThingdObject, ThingdOperation, ThingdOperationResult,
};

use thingd::{
    EventLog, LinkDirection, LinkQueryOptions, LinkStore, ListEventsOptions, ListObjectsOptions,
    MemoryEvent, MemoryObject, ObjectStore, QueueClaimOptions, QueueJob, QueueJobStatus,
    QueueNackOptions, QueueStore,
};

pub struct NativeThingdBackend {
    store: NativeThingdStore,
}

impl NativeThingdBackend {
    pub fn memory() -> Self {
        Self {
            store: NativeThingdStore::memory(),
        }
    }

    pub fn persistent(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let store = NativeThingdStore::persistent(path).map_err(|e| {
            AppError::new(
                ErrorKind::Internal,
                format!("failed to open native store: {e}"),
            )
        })?;
        Ok(Self { store })
    }

    /// Execute one synchronous native thingd operation outside Tokio worker
    /// threads. The public `NativeThingdStore` API remains synchronous for
    /// advanced integrations; this boundary is used by the async adapter.
    pub async fn run_blocking<R, F>(&self, operation: F) -> Result<R, AppError>
    where
        R: Send + 'static,
        F: FnOnce(NativeThingdStore) -> Result<R, AppError> + Send + 'static,
    {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || operation(store))
            .await
            .map_err(|error| {
                AppError::new(
                    ErrorKind::Internal,
                    format!("native thingd blocking task failed: {error}"),
                )
            })?
    }
}

fn to_app_error(error: thingd::ThingdError) -> AppError {
    AppError::new(ErrorKind::Internal, error.to_string())
}

fn to_object(obj: &MemoryObject) -> ThingdObject {
    let data = serde_json::from_str(&obj.body).unwrap_or_else(|_| Value::String(obj.body.clone()));
    ThingdObject {
        id: obj.key.id.clone(),
        collection: obj.key.collection.clone(),
        data,
        created_at: obj.created_at.clone(),
        updated_at: obj.updated_at.clone(),
    }
}

fn to_memory_object(collection: &str, id: &str, data: &Value) -> MemoryObject {
    MemoryObject::new(collection, id, data.to_string())
}

fn to_event(event: &MemoryEvent) -> ThingdEvent {
    let data =
        serde_json::from_str(&event.body).unwrap_or_else(|_| Value::String(event.body.clone()));
    ThingdEvent {
        id: event.sequence.to_string(),
        stream: event.stream.clone(),
        event_type: event.event_type.clone(),
        data,
        timestamp: event.created_at.clone(),
    }
}

fn to_job(job: &QueueJob) -> ThingdJob {
    let payload =
        serde_json::from_str(&job.body).unwrap_or_else(|_| Value::String(job.body.clone()));
    let state = match job.status {
        QueueJobStatus::Ready => crate::thingd::JobState::Queued,
        QueueJobStatus::Leased => crate::thingd::JobState::Leased,
        QueueJobStatus::Completed => crate::thingd::JobState::Completed,
        QueueJobStatus::Dead => crate::thingd::JobState::Dead,
    };
    ThingdJob {
        id: job.id.clone(),
        queue: job.queue.clone(),
        payload,
        state,
        attempts: job.attempts,
        max_retries: job.max_attempts,
        lease_expires_at: job
            .lease_expires_at_ms
            .map(|ms| ms.to_string())
            .or_else(|| {
                if job.leased_at_ms.is_some() {
                    Some("".into())
                } else {
                    None
                }
            }),
        created_at: job.created_at.clone(),
        updated_at: String::new(),
    }
}

/// Evaluate a single arqen filter clause against an object's data.
fn matches_filter(obj: &ThingdObject, filter: &ThingdFilter) -> bool {
    let Some(value) = obj.data.get(&filter.field) else {
        return false;
    };
    match filter.operator {
        FilterOperator::Eq => *value == filter.value,
        FilterOperator::Ne => *value != filter.value,
        FilterOperator::Gt => {
            if let (Some(a), Some(b)) = (value.as_f64(), filter.value.as_f64()) {
                a > b
            } else if let (Some(a), Some(b)) = (value.as_str(), filter.value.as_str()) {
                a > b
            } else {
                false
            }
        }
        FilterOperator::Lt => {
            if let (Some(a), Some(b)) = (value.as_f64(), filter.value.as_f64()) {
                a < b
            } else if let (Some(a), Some(b)) = (value.as_str(), filter.value.as_str()) {
                a < b
            } else {
                false
            }
        }
        FilterOperator::Gte => {
            if let (Some(a), Some(b)) = (value.as_f64(), filter.value.as_f64()) {
                a >= b
            } else if let (Some(a), Some(b)) = (value.as_str(), filter.value.as_str()) {
                a >= b
            } else {
                false
            }
        }
        FilterOperator::Lte => {
            if let (Some(a), Some(b)) = (value.as_f64(), filter.value.as_f64()) {
                a <= b
            } else if let (Some(a), Some(b)) = (value.as_str(), filter.value.as_str()) {
                a <= b
            } else {
                false
            }
        }
        FilterOperator::Contains => {
            if let Some(search_str) = filter.value.as_str() {
                if let Some(value_str) = value.as_str() {
                    value_str.contains(search_str)
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
}

fn applies_filters(obj: &ThingdObject, filters: &[ThingdFilter]) -> bool {
    filters.iter().all(|filter| matches_filter(obj, filter))
}

/// Query the real thingd engine for every object in a collection.
fn list_collection_objects(
    store: &NativeThingdStore,
    collection: &str,
) -> Result<Vec<ThingdObject>, AppError> {
    store.with_engine(|engine| {
        let objects = match engine {
            crate::thingd::NativeThingdEngine::Memory(e) => e.list_objects(
                Some(&[collection.to_string()]),
                &ListObjectsOptions::default(),
            ),
            crate::thingd::NativeThingdEngine::Persistent(e) => e.list_objects(
                Some(&[collection.to_string()]),
                &ListObjectsOptions::default(),
            ),
        }
        .map_err(to_app_error)?;
        Ok(objects.iter().map(to_object).collect())
    })?
}

fn put_object_into_store(
    store: &NativeThingdStore,
    collection: &str,
    id: &str,
    data: &Value,
) -> Result<ThingdObject, AppError> {
    store.with_engine(|engine| {
        let object = to_memory_object(collection, id, data);
        let stored = match engine {
            crate::thingd::NativeThingdEngine::Memory(e) => e.put_object(object),
            crate::thingd::NativeThingdEngine::Persistent(e) => e.put_object(object),
        }
        .map_err(to_app_error)?;
        Ok(to_object(&stored))
    })?
}

#[async_trait]
impl ThingdBackend for NativeThingdBackend {
    async fn get_object(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<Option<ThingdObject>, AppError> {
        let collection = collection.to_string();
        let id = id.to_string();
        self.run_blocking(move |store| {
            store.with_engine(|engine| {
                let found = match engine {
                    crate::thingd::NativeThingdEngine::Memory(e) => e.get_object(&collection, &id),
                    crate::thingd::NativeThingdEngine::Persistent(e) => {
                        e.get_object(&collection, &id)
                    }
                }
                .map_err(to_app_error)?;
                Ok(found.as_ref().map(to_object))
            })
        })
        .await?
    }

    async fn put_object(
        &self,
        collection: &str,
        id: &str,
        data: Value,
    ) -> Result<ThingdObject, AppError> {
        let collection = collection.to_string();
        let id = id.to_string();
        self.run_blocking(move |store| put_object_into_store(&store, &collection, &id, &data))
            .await
    }

    async fn delete_object(&self, collection: &str, id: &str) -> Result<(), AppError> {
        let collection = collection.to_string();
        let id = id.to_string();
        self.run_blocking(move |store| {
            store.with_engine(|engine| {
                let deleted = match engine {
                    crate::thingd::NativeThingdEngine::Memory(e) => {
                        e.delete_object(&collection, &id)
                    }
                    crate::thingd::NativeThingdEngine::Persistent(e) => {
                        e.delete_object(&collection, &id)
                    }
                }
                .map_err(to_app_error)?;
                let _ = deleted;
                Ok(())
            })
        })
        .await?
    }

    async fn query_objects(
        &self,
        collection: &str,
        options: QueryOptions,
    ) -> Result<Vec<ThingdObject>, AppError> {
        let collection = collection.to_string();
        let filters = options.filters.clone();
        let matched = self
            .run_blocking(move |store| list_collection_objects(&store, &collection))
            .await?;
        let mut matched = matched;
        matched.retain(|obj| applies_filters(obj, &filters));
        let items: Vec<ThingdObject> = matched
            .into_iter()
            .skip(options.offset)
            .take(options.limit.unwrap_or(usize::MAX))
            .collect();
        Ok(items)
    }

    async fn count_objects(&self, collection: &str) -> Result<usize, AppError> {
        let collection = collection.to_string();
        self.run_blocking(move |store| {
            store.with_engine(|engine| {
                let count = match engine {
                    crate::thingd::NativeThingdEngine::Memory(e) => {
                        e.count_objects_in_collection(&collection)
                    }
                    crate::thingd::NativeThingdEngine::Persistent(e) => {
                        e.count_objects_in_collection(&collection)
                    }
                }
                .map_err(to_app_error)?;
                Ok(count as usize)
            })
        })
        .await?
    }

    async fn batch_write(
        &self,
        operations: Vec<ThingdOperation>,
    ) -> Result<Vec<ThingdOperationResult>, AppError> {
        let result = self
            .run_blocking(move |store| {
                let mut results = Vec::with_capacity(operations.len());
                for op in operations {
                    let result = match op {
                        ThingdOperation::Put {
                            collection,
                            id,
                            data,
                        } => match put_object_into_store(&store, &collection, &id, &data) {
                            Ok(_) => ThingdOperationResult {
                                success: true,
                                error: None,
                            },
                            Err(e) => ThingdOperationResult {
                                success: false,
                                error: Some(e.to_string()),
                            },
                        },
                        ThingdOperation::Delete { collection, id } => {
                            match store.with_engine(|engine| {
                                match engine {
                                    crate::thingd::NativeThingdEngine::Memory(e) => {
                                        e.delete_object(&collection, &id)
                                    }
                                    crate::thingd::NativeThingdEngine::Persistent(e) => {
                                        e.delete_object(&collection, &id)
                                    }
                                }
                                .map_err(to_app_error)
                            }) {
                                Ok(_) => ThingdOperationResult {
                                    success: true,
                                    error: None,
                                },
                                Err(e) => ThingdOperationResult {
                                    success: false,
                                    error: Some(e.to_string()),
                                },
                            }
                        }
                    };
                    results.push(result);
                }
                Ok(results)
            })
            .await?;
        Ok(result)
    }

    async fn append_event(
        &self,
        stream: &str,
        event_type: &str,
        data: Value,
    ) -> Result<ThingdEvent, AppError> {
        let stream = stream.to_string();
        let event_type = event_type.to_string();
        self.run_blocking(move |store| {
            store.with_engine(|engine| {
                let event = MemoryEvent::new(&stream, &event_type, data.to_string());
                let stored = match engine {
                    crate::thingd::NativeThingdEngine::Memory(e) => e.append_event(event),
                    crate::thingd::NativeThingdEngine::Persistent(e) => e.append_event(event),
                }
                .map_err(to_app_error)?;
                Ok(to_event(&stored))
            })
        })
        .await?
    }

    async fn read_events(
        &self,
        stream: &str,
        from: Option<String>,
        limit: usize,
    ) -> Result<Vec<ThingdEvent>, AppError> {
        let stream = stream.to_string();
        self.run_blocking(move |store| {
            store.with_engine(|engine| {
                let from_sequence = from.as_deref().and_then(|id| id.parse::<u64>().ok());
                let options = ListEventsOptions {
                    from_sequence,
                    limit: Some(limit as u64),
                    since: None,
                };
                let events = match engine {
                    crate::thingd::NativeThingdEngine::Memory(e) => {
                        e.list_events(Some(&stream), options)
                    }
                    crate::thingd::NativeThingdEngine::Persistent(e) => {
                        e.list_events(Some(&stream), options)
                    }
                }
                .map_err(to_app_error)?;
                Ok(events.iter().map(to_event).collect())
            })
        })
        .await?
    }

    async fn push_job(
        &self,
        queue: &str,
        payload: Value,
        max_retries: u32,
    ) -> Result<ThingdJob, AppError> {
        let queue = queue.to_string();
        self.run_blocking(move |store| {
            store.with_engine(|engine| {
                let job = QueueJob::new(
                    &queue,
                    Uuid::new_v4().to_string(),
                    payload.to_string(),
                    max_retries,
                );
                let pushed = match engine {
                    crate::thingd::NativeThingdEngine::Memory(e) => e.push_job(job),
                    crate::thingd::NativeThingdEngine::Persistent(e) => e.push_job(job),
                }
                .map_err(to_app_error)?;
                Ok(to_job(&pushed))
            })
        })
        .await?
    }

    async fn claim_job(
        &self,
        queue: &str,
        _worker_id: &str,
        lease_seconds: u32,
    ) -> Result<Option<ThingdJob>, AppError> {
        let queue = queue.to_string();
        self.run_blocking(move |store| {
            store.with_engine(|engine| {
                let options = QueueClaimOptions::new(lease_seconds.saturating_mul(1000) as u64);
                let claimed = match engine {
                    crate::thingd::NativeThingdEngine::Memory(e) => {
                        e.claim_job_with_options(&queue, options)
                    }
                    crate::thingd::NativeThingdEngine::Persistent(e) => {
                        e.claim_job_with_options(&queue, options)
                    }
                }
                .map_err(to_app_error)?;
                Ok(claimed.as_ref().map(to_job))
            })
        })
        .await?
    }

    async fn complete_job(&self, queue: &str, job_id: &str) -> Result<(), AppError> {
        let queue = queue.to_string();
        let job_id = job_id.to_string();
        self.run_blocking(move |store| {
            store.with_engine(|engine| {
                match engine {
                    crate::thingd::NativeThingdEngine::Memory(e) => e.ack_job(&queue, &job_id),
                    crate::thingd::NativeThingdEngine::Persistent(e) => e.ack_job(&queue, &job_id),
                }
                .map_err(to_app_error)?;
                Ok(())
            })
        })
        .await?
    }

    async fn nack_job(&self, queue: &str, job_id: &str) -> Result<(), AppError> {
        let queue = queue.to_string();
        let job_id = job_id.to_string();
        self.run_blocking(move |store| {
            store.with_engine(|engine| {
                match engine {
                    crate::thingd::NativeThingdEngine::Memory(e) => {
                        e.nack_job_with_options(&queue, &job_id, QueueNackOptions::default())
                    }
                    crate::thingd::NativeThingdEngine::Persistent(e) => {
                        e.nack_job_with_options(&queue, &job_id, QueueNackOptions::default())
                    }
                }
                .map_err(to_app_error)?;
                Ok(())
            })
        })
        .await?
    }

    async fn dead_letter_job(&self, queue: &str, job_id: &str) -> Result<(), AppError> {
        let queue = queue.to_string();
        let job_id = job_id.to_string();
        self.run_blocking(move |store| {
            store.with_engine(|engine| {
                match engine {
                    crate::thingd::NativeThingdEngine::Memory(e) => e.nack_job_with_options(
                        &queue,
                        &job_id,
                        QueueNackOptions::with_error(0, "dead-lettered"),
                    ),
                    crate::thingd::NativeThingdEngine::Persistent(e) => e.nack_job_with_options(
                        &queue,
                        &job_id,
                        QueueNackOptions::with_error(0, "dead-lettered"),
                    ),
                }
                .map_err(to_app_error)?;
                Ok(())
            })
        })
        .await?
    }

    async fn search(&self, query: &str, options: SearchOptions) -> Result<SearchResults, AppError> {
        let query = query.to_lowercase();
        let all: Vec<ThingdObject> = self
            .run_blocking(move |store| {
                store.with_engine(|engine| -> Result<Vec<ThingdObject>, AppError> {
                    let objects = match engine {
                        crate::thingd::NativeThingdEngine::Memory(e) => {
                            e.list_objects(None, &ListObjectsOptions::default())
                        }
                        crate::thingd::NativeThingdEngine::Persistent(e) => {
                            e.list_objects(None, &ListObjectsOptions::default())
                        }
                    }
                    .map_err(to_app_error)?;
                    Ok(objects.iter().map(to_object).collect())
                })?
            })
            .await?;

        let mut matched: Vec<ThingdObject> = all
            .into_iter()
            .filter(|obj| {
                if obj
                    .data
                    .as_str()
                    .is_some_and(|s| s.to_lowercase().contains(&query))
                {
                    return true;
                }
                if let Some(object) = obj.data.as_object() {
                    object.values().any(|value| {
                        value
                            .as_str()
                            .is_some_and(|s| s.to_lowercase().contains(&query))
                    })
                } else {
                    false
                }
            })
            .collect();

        for filter in options.filters {
            matched.retain(|obj| matches_filter(obj, &filter));
        }

        let total = matched.len();
        let items: Vec<ThingdObject> = matched
            .into_iter()
            .skip(options.offset)
            .take(options.limit)
            .collect();

        Ok(SearchResults { items, total })
    }

    async fn create_link(
        &self,
        source_id: &str,
        target_id: &str,
        relation: &str,
    ) -> Result<ThingdLink, AppError> {
        let source_id = source_id.to_string();
        let target_id = target_id.to_string();
        let relation = relation.to_string();
        self.run_blocking(move |store| {
            store.with_engine(|engine| {
                let link = thingd::Link {
                    id: Uuid::new_v4().to_string(),
                    from_ref: source_id,
                    link_type: relation,
                    to_ref: target_id,
                    weight: None,
                    metadata_json: String::new(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                };
                let created = match engine {
                    crate::thingd::NativeThingdEngine::Memory(e) => e.create_link(link),
                    crate::thingd::NativeThingdEngine::Persistent(e) => e.create_link(link),
                }
                .map_err(to_app_error)?;
                Ok(ThingdLink {
                    id: created.id,
                    source_id: created.from_ref,
                    target_id: created.to_ref,
                    relation: created.link_type,
                    created_at: created.created_at,
                })
            })
        })
        .await?
    }

    async fn get_links(
        &self,
        source_id: &str,
        relation: Option<&str>,
    ) -> Result<Vec<ThingdLink>, AppError> {
        let source_id = source_id.to_string();
        let relation = relation.map(ToOwned::to_owned);
        self.run_blocking(move |store| {
            store.with_engine(|engine| {
                let options = LinkQueryOptions {
                    link_type: relation,
                    limit: None,
                };
                let links = match engine {
                    crate::thingd::NativeThingdEngine::Memory(e) => {
                        e.get_neighbors(&source_id, LinkDirection::Outgoing, options)
                    }
                    crate::thingd::NativeThingdEngine::Persistent(e) => {
                        e.get_neighbors(&source_id, LinkDirection::Outgoing, options)
                    }
                }
                .map_err(to_app_error)?;
                Ok(links
                    .into_iter()
                    .map(|link| ThingdLink {
                        id: link.id,
                        source_id: link.from_ref,
                        target_id: link.to_ref,
                        relation: link.link_type,
                        created_at: link.created_at,
                    })
                    .collect())
            })
        })
        .await?
    }

    async fn delete_link(&self, link_id: &str) -> Result<(), AppError> {
        let link_id = link_id.to_string();
        self.run_blocking(move |store| {
            store.with_engine(|engine| {
                match engine {
                    crate::thingd::NativeThingdEngine::Memory(e) => e.delete_link(&link_id),
                    crate::thingd::NativeThingdEngine::Persistent(e) => e.delete_link(&link_id),
                }
                .map_err(to_app_error)?;
                Ok(())
            })
        })
        .await?
    }

    async fn reset(&self) -> Result<(), AppError> {
        self.run_blocking(move |store| {
            store.with_engine(|engine| {
                let collections = match engine {
                    crate::thingd::NativeThingdEngine::Memory(e) => e.list_collections(),
                    crate::thingd::NativeThingdEngine::Persistent(e) => e.list_collections(),
                }
                .map_err(to_app_error)?;
                for collection in collections {
                    let ids = list_collection_ids(&store, &collection)?;
                    for id in ids {
                        match engine {
                            crate::thingd::NativeThingdEngine::Memory(e) => {
                                e.delete_object(&collection, &id)
                            }
                            crate::thingd::NativeThingdEngine::Persistent(e) => {
                                e.delete_object(&collection, &id)
                            }
                        }
                        .map_err(to_app_error)?;
                    }
                }
                Ok(())
            })
        })
        .await?
    }

    async fn seed(&self) -> Result<(), AppError> {
        Ok(())
    }
}

fn list_collection_ids(
    store: &NativeThingdStore,
    collection: &str,
) -> Result<Vec<String>, AppError> {
    let objects = list_collection_objects(store, collection)?;
    Ok(objects.into_iter().map(|o| o.id).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn put_and_get_object() {
        let backend = NativeThingdBackend::memory();
        backend
            .put_object(
                "watchloom_titles",
                "t1",
                serde_json::json!({"titleId": "t1"}),
            )
            .await
            .unwrap();
        let obj = backend.get_object("watchloom_titles", "t1").await.unwrap();
        assert!(obj.is_some());
        assert_eq!(obj.unwrap().data["titleId"], "t1");
    }

    #[tokio::test]
    async fn query_objects_with_filter() {
        let backend = NativeThingdBackend::memory();
        for (id, season) in [("s1", "season1"), ("s2", "season2")] {
            backend
                .put_object(
                    "watchloom_seasons",
                    id,
                    serde_json::json!({ "seasonId": id, "titleId": season }),
                )
                .await
                .unwrap();
        }
        let results = backend
            .query_objects(
                "watchloom_seasons",
                QueryOptions::filtered(vec![ThingdFilter {
                    field: "titleId".to_string(),
                    operator: FilterOperator::Eq,
                    value: Value::String("season2".to_string()),
                }]),
            )
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].data["seasonId"], "s2");
    }

    #[tokio::test]
    async fn append_and_read_events() {
        let backend = NativeThingdBackend::memory();
        let first = backend
            .append_event("library:u1", "library.added", serde_json::json!({}))
            .await
            .unwrap();
        backend
            .append_event("library:u1", "library.updated", serde_json::json!({}))
            .await
            .unwrap();
        let events = backend.read_events("library:u1", None, 10).await.unwrap();
        assert_eq!(events.len(), 2);
        let after = backend
            .read_events("library:u1", Some(first.id), 10)
            .await
            .unwrap();
        assert_eq!(after.len(), 1);
    }

    #[tokio::test]
    async fn queue_claim_complete() {
        let backend = NativeThingdBackend::memory();
        let job = backend
            .push_job("availability_refresh", serde_json::json!({ "t": 1 }), 3)
            .await
            .unwrap();
        let claimed = backend
            .claim_job("availability_refresh", "w1", 30)
            .await
            .unwrap();
        assert!(claimed.is_some());
        assert_eq!(claimed.unwrap().id, job.id);
        backend
            .complete_job("availability_refresh", &job.id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn search_matches_string_fields() {
        let backend = NativeThingdBackend::memory();
        backend
            .put_object(
                "watchloom_titles",
                "t1",
                serde_json::json!({ "titleId": "t1", "title": "The Matrix" }),
            )
            .await
            .unwrap();
        let results = backend
            .search(
                "matrix",
                SearchOptions {
                    limit: 20,
                    offset: 0,
                    filters: Vec::new(),
                },
            )
            .await
            .unwrap();
        assert_eq!(results.total, 1);
        assert_eq!(results.items[0].data["titleId"], "t1");
    }
}
