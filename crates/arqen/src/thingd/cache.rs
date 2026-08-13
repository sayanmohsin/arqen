//! Optional read-through caching for any [`ThingdBackend`].

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde_json::Value;

use crate::core::AppError;
use crate::observability::{NoopMetricsSink, SharedMetricsSink};
use crate::thingd::{
    QueryOptions, SearchOptions, SearchResults, ThingdBackend, ThingdEvent, ThingdJob, ThingdLink,
    ThingdObject, ThingdOperation, ThingdOperationResult,
};

/// Limits and expiry settings for [`CachingThingdBackend`].
#[derive(Debug, Clone)]
pub struct CachePolicy {
    pub ttl: Duration,
    pub capacity: usize,
}

impl Default for CachePolicy {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(30),
            capacity: 1_024,
        }
    }
}

struct CacheState {
    expires: Mutex<HashMap<String, Instant>>,
    gates: Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>,
}

/// A backend decorator that caches object reads while delegating all other
/// operations to `source`. Writes invalidate the cached object first.
///
/// Prefer [`CachingThingdBackend::new_catalog`] for production configuration.
pub struct CachingThingdBackend {
    source: Arc<dyn ThingdBackend>,
    cache: Arc<dyn ThingdBackend>,
    policy: CachePolicy,
    state: CacheState,
    hits: AtomicU64,
    misses: AtomicU64,
    metrics: SharedMetricsSink,
    allowed_collections: Option<Arc<HashSet<String>>>,
}

impl CachingThingdBackend {
    pub fn new(
        source: Arc<dyn ThingdBackend>,
        cache: Arc<dyn ThingdBackend>,
        policy: CachePolicy,
    ) -> Self {
        Self::new_with_metrics(source, cache, policy, Arc::new(NoopMetricsSink))
    }

    pub fn new_with_metrics(
        source: Arc<dyn ThingdBackend>,
        cache: Arc<dyn ThingdBackend>,
        policy: CachePolicy,
        metrics: SharedMetricsSink,
    ) -> Self {
        Self {
            source,
            cache,
            policy,
            state: CacheState {
                expires: Mutex::new(HashMap::new()),
                gates: Mutex::new(HashMap::new()),
            },
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            metrics,
            allowed_collections: None,
        }
    }

    /// Construct a cache restricted to an explicit catalog collection allowlist.
    ///
    /// This is the safe variant for HTTP and multi-user deployments. A cache
    /// must never be enabled for user-scoped collections unless the caller has
    /// separately guaranteed that the collection contains no tenant data.
    pub fn new_catalog(
        source: Arc<dyn ThingdBackend>,
        cache: Arc<dyn ThingdBackend>,
        policy: CachePolicy,
        collections: impl IntoIterator<Item = String>,
    ) -> Self {
        let mut backend = Self::new(source, cache, policy);
        backend.allowed_collections = Some(Arc::new(collections.into_iter().collect()));
        backend
    }

    pub fn cache_hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }
    pub fn cache_misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    fn key(collection: &str, id: &str) -> String {
        format!("{collection}\0{id}")
    }

    fn cache_collection(&self, collection: &str) -> bool {
        self.allowed_collections
            .as_ref()
            .is_none_or(|collections| collections.contains(collection))
    }

    async fn gate(&self, key: &str) -> Arc<tokio::sync::Mutex<()>> {
        let mut gates = self.state.gates.lock().expect("cache gate mutex poisoned");
        gates
            .entry(key.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .clone()
    }

    fn fresh(&self, key: &str) -> bool {
        self.state
            .expires
            .lock()
            .expect("cache expiry mutex poisoned")
            .get(key)
            .is_some_and(|expiry| *expiry > Instant::now())
    }

    async fn invalidate(&self, collection: &str, id: &str) -> Result<(), AppError> {
        let key = Self::key(collection, id);
        self.cache.delete_object(collection, id).await?;
        self.state
            .expires
            .lock()
            .expect("cache expiry mutex poisoned")
            .remove(&key);
        Ok(())
    }

    async fn cache_object(&self, object: &ThingdObject) -> Result<(), AppError> {
        if self.policy.capacity == 0 {
            return Ok(());
        }
        let key = Self::key(&object.collection, &object.id);
        self.cache
            .put_object(&object.collection, &object.id, object.data.clone())
            .await?;
        let evicted = {
            let expires = self
                .state
                .expires
                .lock()
                .expect("cache expiry mutex poisoned");
            if expires.len() >= self.policy.capacity && !expires.contains_key(&key) {
                expires
                    .iter()
                    .min_by_key(|(_, expiry)| **expiry)
                    .map(|(k, _)| k.clone())
            } else {
                None
            }
        };
        if let Some(oldest) = &evicted {
            if let Some((collection, id)) = oldest.split_once('\0') {
                let _ = self.cache.delete_object(collection, id).await;
            }
            self.state
                .expires
                .lock()
                .expect("cache expiry mutex poisoned")
                .remove(oldest);
            self.metrics
                .record_cache(crate::observability::CacheMetric {
                    operation: "eviction".to_string(),
                    hit: false,
                    duration_ms: 0,
                });
        }
        self.state
            .expires
            .lock()
            .expect("cache expiry mutex poisoned")
            .insert(key, Instant::now() + self.policy.ttl);
        Ok(())
    }
}

#[async_trait]
impl ThingdBackend for CachingThingdBackend {
    async fn get_object(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<Option<ThingdObject>, AppError> {
        if !self.cache_collection(collection) {
            return self.source.get_object(collection, id).await;
        }
        let started = Instant::now();
        let key = Self::key(collection, id);
        if self.fresh(&key)
            && let Some(object) = self.cache.get_object(collection, id).await?
        {
            self.hits.fetch_add(1, Ordering::Relaxed);
            self.metrics
                .record_cache(crate::observability::CacheMetric {
                    operation: "get".to_string(),
                    hit: true,
                    duration_ms: started.elapsed().as_millis() as u64,
                });
            return Ok(Some(object));
        }
        let gate = self.gate(&key).await;
        let _guard = gate.lock().await;
        if self.fresh(&key)
            && let Some(object) = self.cache.get_object(collection, id).await?
        {
            self.hits.fetch_add(1, Ordering::Relaxed);
            self.metrics
                .record_cache(crate::observability::CacheMetric {
                    operation: "get".to_string(),
                    hit: true,
                    duration_ms: started.elapsed().as_millis() as u64,
                });
            return Ok(Some(object));
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        self.metrics
            .record_cache(crate::observability::CacheMetric {
                operation: "get".to_string(),
                hit: false,
                duration_ms: started.elapsed().as_millis() as u64,
            });
        let object = self.source.get_object(collection, id).await?;
        if let Some(object) = &object {
            self.cache_object(object).await?;
        }
        Ok(object)
    }

    async fn put_object(
        &self,
        collection: &str,
        id: &str,
        data: Value,
    ) -> Result<ThingdObject, AppError> {
        self.invalidate(collection, id).await?;
        self.source.put_object(collection, id, data).await
    }
    async fn delete_object(&self, collection: &str, id: &str) -> Result<(), AppError> {
        self.invalidate(collection, id).await?;
        self.source.delete_object(collection, id).await
    }
    async fn query_objects(
        &self,
        collection: &str,
        options: QueryOptions,
    ) -> Result<Vec<ThingdObject>, AppError> {
        self.source.query_objects(collection, options).await
    }
    async fn count_objects(&self, collection: &str) -> Result<usize, AppError> {
        self.source.count_objects(collection).await
    }
    async fn batch_write(
        &self,
        operations: Vec<ThingdOperation>,
    ) -> Result<Vec<ThingdOperationResult>, AppError> {
        for operation in &operations {
            match operation {
                ThingdOperation::Put { collection, id, .. }
                | ThingdOperation::Delete { collection, id } => {
                    self.invalidate(collection, id).await?
                }
            }
        }
        self.source.batch_write(operations).await
    }
    async fn append_event(
        &self,
        stream: &str,
        event_type: &str,
        data: Value,
    ) -> Result<ThingdEvent, AppError> {
        self.source.append_event(stream, event_type, data).await
    }
    async fn read_events(
        &self,
        stream: &str,
        from: Option<String>,
        limit: usize,
    ) -> Result<Vec<ThingdEvent>, AppError> {
        self.source.read_events(stream, from, limit).await
    }
    async fn push_job(
        &self,
        queue: &str,
        payload: Value,
        max_retries: u32,
    ) -> Result<ThingdJob, AppError> {
        self.source.push_job(queue, payload, max_retries).await
    }
    async fn claim_job(
        &self,
        queue: &str,
        worker_id: &str,
        lease_seconds: u32,
    ) -> Result<Option<ThingdJob>, AppError> {
        self.source.claim_job(queue, worker_id, lease_seconds).await
    }
    async fn complete_job(&self, queue: &str, job_id: &str) -> Result<(), AppError> {
        self.source.complete_job(queue, job_id).await
    }
    async fn nack_job(&self, queue: &str, job_id: &str) -> Result<(), AppError> {
        self.source.nack_job(queue, job_id).await
    }
    async fn dead_letter_job(&self, queue: &str, job_id: &str) -> Result<(), AppError> {
        self.source.dead_letter_job(queue, job_id).await
    }
    async fn search(&self, query: &str, options: SearchOptions) -> Result<SearchResults, AppError> {
        self.source.search(query, options).await
    }
    async fn create_link(
        &self,
        source_id: &str,
        target_id: &str,
        relation: &str,
    ) -> Result<ThingdLink, AppError> {
        self.source
            .create_link(source_id, target_id, relation)
            .await
    }
    async fn get_links(
        &self,
        source_id: &str,
        relation: Option<&str>,
    ) -> Result<Vec<ThingdLink>, AppError> {
        self.source.get_links(source_id, relation).await
    }
    async fn delete_link(&self, link_id: &str) -> Result<(), AppError> {
        self.source.delete_link(link_id).await
    }
    async fn reset(&self) -> Result<(), AppError> {
        self.source.reset().await
    }
    async fn seed(&self) -> Result<(), AppError> {
        self.source.seed().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thingd::{MemoryThingdBackend, ThingdBackend};

    #[tokio::test]
    async fn read_through_cache_hits_and_invalidates_on_write() {
        let source: Arc<dyn ThingdBackend> = Arc::new(MemoryThingdBackend::new());
        let cache: Arc<dyn ThingdBackend> = Arc::new(MemoryThingdBackend::new());
        source
            .put_object("movies", "m-1", serde_json::json!({"title":"Before"}))
            .await
            .unwrap();
        let backend = CachingThingdBackend::new(
            source,
            cache,
            CachePolicy {
                ttl: Duration::from_secs(60),
                capacity: 10,
            },
        );

        assert_eq!(
            backend
                .get_object("movies", "m-1")
                .await
                .unwrap()
                .unwrap()
                .data["title"],
            "Before"
        );
        assert_eq!(
            backend
                .get_object("movies", "m-1")
                .await
                .unwrap()
                .unwrap()
                .data["title"],
            "Before"
        );
        assert_eq!(backend.cache_misses(), 1);
        assert_eq!(backend.cache_hits(), 1);

        backend
            .put_object("movies", "m-1", serde_json::json!({"title":"After"}))
            .await
            .unwrap();
        assert_eq!(
            backend
                .get_object("movies", "m-1")
                .await
                .unwrap()
                .unwrap()
                .data["title"],
            "After"
        );
        assert_eq!(backend.cache_misses(), 2);
    }

    #[tokio::test]
    async fn catalog_cache_bypasses_non_allowlisted_collections() {
        let source: Arc<dyn ThingdBackend> = Arc::new(MemoryThingdBackend::new());
        let cache: Arc<dyn ThingdBackend> = Arc::new(MemoryThingdBackend::new());
        source
            .put_object("catalog", "one", serde_json::json!({"title":"Catalog"}))
            .await
            .unwrap();
        source
            .put_object("users", "one", serde_json::json!({"name":"User"}))
            .await
            .unwrap();
        let backend = CachingThingdBackend::new_catalog(
            source,
            cache,
            CachePolicy::default(),
            ["catalog".to_string()],
        );

        backend.get_object("catalog", "one").await.unwrap();
        backend.get_object("catalog", "one").await.unwrap();
        backend.get_object("users", "one").await.unwrap();
        backend.get_object("users", "one").await.unwrap();

        assert_eq!(backend.cache_hits(), 1);
        assert_eq!(backend.cache_misses(), 1);
    }
}
