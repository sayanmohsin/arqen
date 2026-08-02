use chrono::Utc;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::thingd::traits::*;

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
    ) -> Result<Option<ThingdObject>, crate::core::AppError> {
        let objects = self.objects.lock().unwrap();
        let empty_vec = vec![];
        let collection_objects = objects.get(collection).unwrap_or(&empty_vec);
        Ok(collection_objects.iter().find(|o| o.id == id).cloned())
    }

    async fn put_object(
        &self,
        collection: &str,
        id: &str,
        data: serde_json::Value,
    ) -> Result<ThingdObject, crate::core::AppError> {
        let now = Utc::now().to_rfc3339();
        let object = ThingdObject {
            id: id.to_string(),
            collection: collection.to_string(),
            data,
            created_at: now.clone(),
            updated_at: now,
        };

        let mut objects = self.objects.lock().unwrap();
        let collection_objects = objects.entry(collection.to_string()).or_default();

        if let Some(existing) = collection_objects.iter_mut().find(|o| o.id == id) {
            *existing = object.clone();
        } else {
            collection_objects.push(object.clone());
        }

        Ok(object)
    }

    async fn delete_object(&self, collection: &str, id: &str) -> Result<(), crate::core::AppError> {
        let mut objects = self.objects.lock().unwrap();
        if let Some(collection_objects) = objects.get_mut(collection) {
            collection_objects.retain(|o| o.id != id);
        }
        Ok(())
    }

    async fn query_objects(
        &self,
        collection: &str,
        filter: Option<ThingdFilter>,
    ) -> Result<Vec<ThingdObject>, crate::core::AppError> {
        let objects = self.objects.lock().unwrap();
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
    ) -> Result<Vec<ThingdOperationResult>, crate::core::AppError> {
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
    ) -> Result<ThingdEvent, crate::core::AppError> {
        let event = ThingdEvent {
            id: Uuid::new_v4().to_string(),
            stream: stream.to_string(),
            event_type: event_type.to_string(),
            data,
            timestamp: Utc::now().to_rfc3339(),
        };

        let mut events = self.events.lock().unwrap();
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
    ) -> Result<Vec<ThingdEvent>, crate::core::AppError> {
        let events = self.events.lock().unwrap();
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
    ) -> Result<ThingdJob, crate::core::AppError> {
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

        let mut jobs = self.jobs.lock().unwrap();
        jobs.entry(queue.to_string()).or_default().push(job.clone());

        Ok(job)
    }

    async fn claim_job(
        &self,
        queue: &str,
        _worker_id: &str,
        lease_seconds: u32,
    ) -> Result<Option<ThingdJob>, crate::core::AppError> {
        let mut jobs = self.jobs.lock().unwrap();
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

    async fn complete_job(&self, queue: &str, job_id: &str) -> Result<(), crate::core::AppError> {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(queue_jobs) = jobs.get_mut(queue)
            && let Some(job) = queue_jobs.iter_mut().find(|j| j.id == job_id)
        {
            job.state = JobState::Completed;
            job.updated_at = Utc::now().to_rfc3339();
        }
        Ok(())
    }

    async fn nack_job(&self, queue: &str, job_id: &str) -> Result<(), crate::core::AppError> {
        let mut jobs = self.jobs.lock().unwrap();
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

    async fn dead_letter_job(
        &self,
        queue: &str,
        job_id: &str,
    ) -> Result<(), crate::core::AppError> {
        let mut jobs = self.jobs.lock().unwrap();
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
    ) -> Result<SearchResults, crate::core::AppError> {
        let objects = self.objects.lock().unwrap();
        let mut all_objects: Vec<ThingdObject> = Vec::new();

        // Collect all objects from all collections
        for collection_objects in objects.values() {
            all_objects.extend(collection_objects.clone());
        }

        // Simple full-text search: check if query appears in any string value
        let query_lower = query.to_lowercase();
        let mut matched: Vec<ThingdObject> = all_objects
            .into_iter()
            .filter(|obj| {
                // Search in data fields
                if let Some(data_str) = obj.data.as_str()
                    && data_str.to_lowercase().contains(&query_lower)
                {
                    return true;
                }
                // Search in object fields
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

        // Apply filters
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

        // Apply pagination
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
    ) -> Result<ThingdLink, crate::core::AppError> {
        let link = ThingdLink {
            id: Uuid::new_v4().to_string(),
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            relation: relation.to_string(),
            created_at: Utc::now().to_rfc3339(),
        };

        let mut links = self.links.lock().unwrap();
        links.push(link.clone());

        Ok(link)
    }

    async fn get_links(
        &self,
        source_id: &str,
        relation: Option<&str>,
    ) -> Result<Vec<ThingdLink>, crate::core::AppError> {
        let links = self.links.lock().unwrap();
        let filtered = links
            .iter()
            .filter(|l| {
                l.source_id == source_id && relation.map(|r| l.relation == r).unwrap_or(true)
            })
            .cloned()
            .collect();

        Ok(filtered)
    }

    async fn delete_link(&self, link_id: &str) -> Result<(), crate::core::AppError> {
        let mut links = self.links.lock().unwrap();
        links.retain(|l| l.id != link_id);
        Ok(())
    }

    async fn count_objects(&self, collection: &str) -> Result<usize, crate::core::AppError> {
        let objects = self.objects.lock().unwrap();
        let count = objects.get(collection).map(|v| v.len()).unwrap_or(0);
        Ok(count)
    }

    async fn reset(&self) -> Result<(), crate::core::AppError> {
        let mut objects = self.objects.lock().unwrap();
        let mut events = self.events.lock().unwrap();
        let mut jobs = self.jobs.lock().unwrap();
        let mut links = self.links.lock().unwrap();

        objects.clear();
        events.clear();
        jobs.clear();
        links.clear();

        Ok(())
    }

    async fn seed(&self) -> Result<(), crate::core::AppError> {
        // Seed sample objects
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

        // Seed sample events
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

        // Seed sample jobs
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
