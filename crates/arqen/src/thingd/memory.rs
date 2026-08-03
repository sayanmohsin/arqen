use chrono::Utc;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::core::{AppError, ErrorKind};
use crate::thingd::traits::*;

fn lock_mutex<'a, T>(mutex: &'a Mutex<T>, name: &'a str) -> Result<std::sync::MutexGuard<'a, T>, AppError> {
    mutex
        .lock()
        .map_err(|e| AppError::new(ErrorKind::Internal, format!("mutex lock poisoned ({name}): {e}")))
}

pub struct MemoryThingdBackend {
    objects: Mutex<HashMap<String, Vec<ThingdObject>>>,
    events: Mutex<HashMap<String, Vec<ThingdEvent>>>,
    jobs: Mutex<HashMap<String, Vec<ThingdJob>>>,
    links: Mutex<Vec<ThingdLink>>,
}

impl MemoryThingdBackend {
    pub fn new() -> Self {
        Self {
            objects: Mutex::new(HashMap::new()),
            events: Mutex::new(HashMap::new()),
            jobs: Mutex::new(HashMap::new()),
            links: Mutex::new(Vec::new()),
        }
    }
}

impl Default for MemoryThingdBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ThingdBackend for MemoryThingdBackend {
    async fn get_object(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<Option<ThingdObject>, AppError> {
        let objects = lock_mutex(&self.objects, "objects")?;
        let empty_vec = vec![];
        let collection_objects = objects.get(collection).unwrap_or(&empty_vec);
        Ok(collection_objects.iter().find(|o| o.id == id).cloned())
    }

    async fn put_object(
        &self,
        collection: &str,
        id: &str,
        data: serde_json::Value,
    ) -> Result<ThingdObject, AppError> {
        let now = Utc::now().to_rfc3339();
        let object = ThingdObject {
            id: id.to_string(),
            collection: collection.to_string(),
            data,
            created_at: now.clone(),
            updated_at: now,
        };

        let mut objects = lock_mutex(&self.objects, "objects")?;
        let collection_objects = objects.entry(collection.to_string()).or_default();

        if let Some(existing) = collection_objects.iter_mut().find(|o| o.id == id) {
            *existing = object.clone();
        } else {
            collection_objects.push(object.clone());
        }

        Ok(object)
    }

    async fn delete_object(&self, collection: &str, id: &str) -> Result<(), AppError> {
        let mut objects = lock_mutex(&self.objects, "objects")?;
        if let Some(collection_objects) = objects.get_mut(collection) {
            collection_objects.retain(|o| o.id != id);
        }
        Ok(())
    }

    async fn query_objects(
        &self,
        collection: &str,
        filter: Option<ThingdFilter>,
    ) -> Result<Vec<ThingdObject>, AppError> {
        let objects = lock_mutex(&self.objects, "objects")?;
        let collection_objects = objects.get(collection).unwrap_or(&vec![]).clone();

        if let Some(filter) = filter {
            let filtered = collection_objects
                .into_iter()
                .filter(|obj| {
                    if let Some(value) = obj.data.get(&filter.field) {
                        match filter.operator {
                            FilterOperator::Eq => *value == filter.value,
                            FilterOperator::Ne => *value != filter.value,
                            FilterOperator::Gt => {
                                if let (Some(a), Some(b)) = (value.as_f64(), filter.value.as_f64())
                                {
                                    a > b
                                } else if let (Some(a), Some(b)) =
                                    (value.as_str(), filter.value.as_str())
                                {
                                    a > b
                                } else {
                                    false
                                }
                            }
                            FilterOperator::Lt => {
                                if let (Some(a), Some(b)) = (value.as_f64(), filter.value.as_f64())
                                {
                                    a < b
                                } else if let (Some(a), Some(b)) =
                                    (value.as_str(), filter.value.as_str())
                                {
                                    a < b
                                } else {
                                    false
                                }
                            }
                            FilterOperator::Gte => {
                                if let (Some(a), Some(b)) = (value.as_f64(), filter.value.as_f64())
                                {
                                    a >= b
                                } else if let (Some(a), Some(b)) =
                                    (value.as_str(), filter.value.as_str())
                                {
                                    a >= b
                                } else {
                                    false
                                }
                            }
                            FilterOperator::Lte => {
                                if let (Some(a), Some(b)) = (value.as_f64(), filter.value.as_f64())
                                {
                                    a <= b
                                } else if let (Some(a), Some(b)) =
                                    (value.as_str(), filter.value.as_str())
                                {
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
                    } else {
                        false
                    }
                })
                .collect();
            Ok(filtered)
        } else {
            Ok(collection_objects)
        }
    }

    async fn batch_write(
        &self,
        operations: Vec<ThingdOperation>,
    ) -> Result<Vec<ThingdOperationResult>, AppError> {
        let mut results = Vec::new();

        for op in operations {
            let result = match op {
                ThingdOperation::Put {
                    collection,
                    id,
                    data,
                } => match self.put_object(&collection, &id, data).await {
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
                    match self.delete_object(&collection, &id).await {
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
    }

    async fn append_event(
        &self,
        stream: &str,
        event_type: &str,
        data: serde_json::Value,
    ) -> Result<ThingdEvent, AppError> {
        let event = ThingdEvent {
            id: Uuid::new_v4().to_string(),
            stream: stream.to_string(),
            event_type: event_type.to_string(),
            data,
            timestamp: Utc::now().to_rfc3339(),
        };

        let mut events = lock_mutex(&self.events, "events")?;
        events
            .entry(stream.to_string())
            .or_default()
            .push(event.clone());

        Ok(event)
    }

    async fn read_events(
        &self,
        stream: &str,
        from: Option<String>,
        limit: usize,
    ) -> Result<Vec<ThingdEvent>, AppError> {
        let events = lock_mutex(&self.events, "events")?;
        let stream_events = events.get(stream).unwrap_or(&vec![]).clone();

        let filtered = if let Some(from_id) = from {
            let start_index = stream_events
                .iter()
                .position(|e| e.id == from_id)
                .unwrap_or(0);
            stream_events
                .into_iter()
                .skip(start_index + 1)
                .take(limit)
                .collect()
        } else {
            stream_events.into_iter().take(limit).collect()
        };

        Ok(filtered)
    }

    async fn push_job(
        &self,
        queue: &str,
        payload: serde_json::Value,
        max_retries: u32,
    ) -> Result<ThingdJob, AppError> {
        let now = Utc::now().to_rfc3339();
        let job = ThingdJob {
            id: Uuid::new_v4().to_string(),
            queue: queue.to_string(),
            payload,
            state: JobState::Queued,
            attempts: 0,
            max_retries,
            lease_expires_at: None,
            created_at: now.clone(),
            updated_at: now,
        };

        let mut jobs = lock_mutex(&self.jobs, "jobs")?;
        jobs.entry(queue.to_string()).or_default().push(job.clone());

        Ok(job)
    }

    async fn claim_job(
        &self,
        queue: &str,
        _worker_id: &str,
        lease_seconds: u32,
    ) -> Result<Option<ThingdJob>, AppError> {
        let mut jobs = lock_mutex(&self.jobs, "jobs")?;
        let queue_jobs = jobs.entry(queue.to_string()).or_default();

        if let Some(job) = queue_jobs
            .iter_mut()
            .find(|j| j.state == JobState::Queued || j.state == JobState::Retrying)
        {
            job.state = JobState::Leased;
            job.attempts += 1;
            job.lease_expires_at =
                Some((Utc::now() + chrono::Duration::seconds(lease_seconds as i64)).to_rfc3339());
            job.updated_at = Utc::now().to_rfc3339();
            Ok(Some(job.clone()))
        } else {
            Ok(None)
        }
    }

    async fn complete_job(&self, queue: &str, job_id: &str) -> Result<(), AppError> {
        let mut jobs = lock_mutex(&self.jobs, "jobs")?;
        if let Some(queue_jobs) = jobs.get_mut(queue)
            && let Some(job) = queue_jobs.iter_mut().find(|j| j.id == job_id)
        {
            job.state = JobState::Completed;
            job.updated_at = Utc::now().to_rfc3339();
        }
        Ok(())
    }

    async fn nack_job(&self, queue: &str, job_id: &str) -> Result<(), AppError> {
        let mut jobs = lock_mutex(&self.jobs, "jobs")?;
        if let Some(queue_jobs) = jobs.get_mut(queue)
            && let Some(job) = queue_jobs.iter_mut().find(|j| j.id == job_id)
        {
            if job.attempts < job.max_retries {
                job.state = JobState::Retrying;
            } else {
                job.state = JobState::Dead;
            }
            job.updated_at = Utc::now().to_rfc3339();
        }
        Ok(())
    }

    async fn dead_letter_job(&self, queue: &str, job_id: &str) -> Result<(), AppError> {
        let mut jobs = lock_mutex(&self.jobs, "jobs")?;
        if let Some(queue_jobs) = jobs.get_mut(queue)
            && let Some(job) = queue_jobs.iter_mut().find(|j| j.id == job_id)
        {
            job.state = JobState::Dead;
            job.updated_at = Utc::now().to_rfc3339();
        }
        Ok(())
    }

    async fn search(
        &self,
        query: &str,
        options: SearchOptions,
    ) -> Result<SearchResults, AppError> {
        let objects = lock_mutex(&self.objects, "objects")?;
        let mut all_objects: Vec<ThingdObject> = Vec::new();

        for collection_objects in objects.values() {
            all_objects.extend(collection_objects.clone());
        }

        let query_lower = query.to_lowercase();
        let mut matched: Vec<ThingdObject> = all_objects
            .into_iter()
            .filter(|obj| {
                if let Some(data_str) = obj.data.as_str()
                    && data_str.to_lowercase().contains(&query_lower)
                {
                    return true;
                }
                if let Some(obj) = obj.data.as_object() {
                    for (_, value) in obj {
                        if let Some(s) = value.as_str()
                            && s.to_lowercase().contains(&query_lower)
                        {
                            return true;
                        }
                    }
                }
                false
            })
            .collect();

        for filter in options.filters {
            matched.retain(|obj| {
                if let Some(value) = obj.data.get(&filter.field) {
                    match filter.operator {
                        FilterOperator::Eq => *value == filter.value,
                        FilterOperator::Ne => *value != filter.value,
                        FilterOperator::Contains => {
                            if let Some(search_str) = filter.value.as_str() {
                                if let Some(value_str) = value.as_str() {
                                    value_str
                                        .to_lowercase()
                                        .contains(&search_str.to_lowercase())
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        _ => true,
                    }
                } else {
                    false
                }
            });
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
        let link = ThingdLink {
            id: Uuid::new_v4().to_string(),
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            relation: relation.to_string(),
            created_at: Utc::now().to_rfc3339(),
        };

        let mut links = lock_mutex(&self.links, "links")?;
        links.push(link.clone());

        Ok(link)
    }

    async fn get_links(
        &self,
        source_id: &str,
        relation: Option<&str>,
    ) -> Result<Vec<ThingdLink>, AppError> {
        let links = lock_mutex(&self.links, "links")?;
        let filtered = links
            .iter()
            .filter(|l| {
                l.source_id == source_id && relation.map(|r| l.relation == r).unwrap_or(true)
            })
            .cloned()
            .collect();

        Ok(filtered)
    }

    async fn delete_link(&self, link_id: &str) -> Result<(), AppError> {
        let mut links = lock_mutex(&self.links, "links")?;
        links.retain(|l| l.id != link_id);
        Ok(())
    }

    async fn count_objects(&self, collection: &str) -> Result<usize, AppError> {
        let objects = lock_mutex(&self.objects, "objects")?;
        let count = objects.get(collection).map(|v| v.len()).unwrap_or(0);
        Ok(count)
    }

    async fn reset(&self) -> Result<(), AppError> {
        let mut objects = lock_mutex(&self.objects, "objects")?;
        let mut events = lock_mutex(&self.events, "events")?;
        let mut jobs = lock_mutex(&self.jobs, "jobs")?;
        let mut links = lock_mutex(&self.links, "links")?;

        objects.clear();
        events.clear();
        jobs.clear();
        links.clear();

        Ok(())
    }

    async fn seed(&self) -> Result<(), AppError> {
        let sample_users = vec![
            (
                "user1",
                serde_json::json!({"name": "Alice", "email": "alice@example.com"}),
            ),
            (
                "user2",
                serde_json::json!({"name": "Bob", "email": "bob@example.com"}),
            ),
            (
                "user3",
                serde_json::json!({"name": "Charlie", "email": "charlie@example.com"}),
            ),
        ];

        for (id, data) in sample_users {
            self.put_object("users", id, data).await?;
        }

        self.append_event(
            "user_events",
            "user_created",
            serde_json::json!({"user_id": "user1"}),
        )
        .await?;
        self.append_event(
            "user_events",
            "user_created",
            serde_json::json!({"user_id": "user2"}),
        )
        .await?;

        self.push_job(
            "email_queue",
            serde_json::json!({"to": "alice@example.com", "subject": "Welcome"}),
            3,
        )
        .await?;
        self.push_job(
            "email_queue",
            serde_json::json!({"to": "bob@example.com", "subject": "Welcome"}),
            3,
        )
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_backend() -> MemoryThingdBackend {
        MemoryThingdBackend::new()
    }

    #[tokio::test]
    async fn test_put_and_get_object() {
        let backend = create_backend().await;
        let data = serde_json::json!({"name": "Alice", "age": 30});

        let obj = backend.put_object("users", "user1", data.clone()).await.unwrap();
        assert_eq!(obj.id, "user1");
        assert_eq!(obj.collection, "users");

        let retrieved = backend.get_object("users", "user1").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().data, data);
    }

    #[tokio::test]
    async fn test_get_nonexistent_object() {
        let backend = create_backend().await;
        let result = backend.get_object("users", "nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_object() {
        let backend = create_backend().await;
        backend.put_object("users", "user1", serde_json::json!({"name": "Alice"})).await.unwrap();

        backend.delete_object("users", "user1").await.unwrap();
        let result = backend.get_object("users", "user1").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_query_objects_with_filter() {
        let backend = create_backend().await;
        backend.put_object("users", "u1", serde_json::json!({"name": "Alice", "age": 30})).await.unwrap();
        backend.put_object("users", "u2", serde_json::json!({"name": "Bob", "age": 25})).await.unwrap();
        backend.put_object("users", "u3", serde_json::json!({"name": "Charlie", "age": 35})).await.unwrap();

        let filter = ThingdFilter {
            field: "age".to_string(),
            operator: FilterOperator::Gt,
            value: serde_json::json!(27),
        };
        let results = backend.query_objects("users", Some(filter)).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_count_objects() {
        let backend = create_backend().await;
        assert_eq!(backend.count_objects("users").await.unwrap(), 0);

        backend.put_object("users", "u1", serde_json::json!({})).await.unwrap();
        backend.put_object("users", "u2", serde_json::json!({})).await.unwrap();
        assert_eq!(backend.count_objects("users").await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_append_and_read_events() {
        let backend = create_backend().await;

        let event1 = backend.append_event("audit", "user.created", serde_json::json!({"id": "u1"})).await.unwrap();
        let _event2 = backend.append_event("audit", "user.created", serde_json::json!({"id": "u2"})).await.unwrap();

        let events = backend.read_events("audit", None, 10).await.unwrap();
        assert_eq!(events.len(), 2);

        let events_from = backend.read_events("audit", Some(event1.id), 10).await.unwrap();
        assert_eq!(events_from.len(), 1);
    }

    #[tokio::test]
    async fn test_job_lifecycle() {
        let backend = create_backend().await;

        let job = backend.push_job("queue", serde_json::json!({"task": "send_email"}), 3).await.unwrap();
        assert_eq!(job.state, JobState::Queued);

        let claimed = backend.claim_job("queue", "worker1", 60).await.unwrap();
        assert!(claimed.is_some());
        assert_eq!(claimed.unwrap().state, JobState::Leased);

        backend.complete_job("queue", &job.id).await.unwrap();
        // Note: jobs are stored separately from objects, so get_object won't find them
    }

    #[tokio::test]
    async fn test_batch_write() {
        let backend = create_backend().await;

        let ops = vec![
            ThingdOperation::Put {
                collection: "users".to_string(),
                id: "u1".to_string(),
                data: serde_json::json!({"name": "Alice"}),
            },
            ThingdOperation::Put {
                collection: "users".to_string(),
                id: "u2".to_string(),
                data: serde_json::json!({"name": "Bob"}),
            },
            ThingdOperation::Delete {
                collection: "users".to_string(),
                id: "u1".to_string(),
            },
        ];

        let results = backend.batch_write(ops).await.unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.success));

        assert!(backend.get_object("users", "u1").await.unwrap().is_none());
        assert!(backend.get_object("users", "u2").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_create_and_get_links() {
        let backend = create_backend().await;

        let link = backend.create_link("doc1", "doc2", "references").await.unwrap();
        assert_eq!(link.source_id, "doc1");
        assert_eq!(link.target_id, "doc2");
        assert_eq!(link.relation, "references");

        let links = backend.get_links("doc1", None).await.unwrap();
        assert_eq!(links.len(), 1);

        backend.delete_link(&link.id).await.unwrap();
        let links = backend.get_links("doc1", None).await.unwrap();
        assert_eq!(links.len(), 0);
    }

    #[tokio::test]
    async fn test_search() {
        let backend = create_backend().await;
        backend.put_object("docs", "d1", serde_json::json!({"title": "Rust guide"})).await.unwrap();
        backend.put_object("docs", "d2", serde_json::json!({"title": "Python guide"})).await.unwrap();
        backend.put_object("docs", "d3", serde_json::json!({"title": "Rust advanced"})).await.unwrap();

        let results = backend.search("rust", SearchOptions {
            limit: 10,
            offset: 0,
            filters: vec![],
        }).await.unwrap();
        assert_eq!(results.total, 2);
    }

    #[tokio::test]
    async fn test_reset() {
        let backend = create_backend().await;
        backend.put_object("users", "u1", serde_json::json!({})).await.unwrap();
        backend.append_event("audit", "test", serde_json::json!({})).await.unwrap();

        backend.reset().await.unwrap();

        assert_eq!(backend.count_objects("users").await.unwrap(), 0);
        let events = backend.read_events("audit", None, 10).await.unwrap();
        assert_eq!(events.len(), 0);
    }
}
