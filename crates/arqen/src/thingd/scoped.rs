//! Tenant, instance, and subject isolation for thingd-backed applications.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::core::{AppError, ErrorKind};
use crate::thingd::{
    QueryOptions, SearchOptions, SearchResults, ThingdBackend, ThingdEvent, ThingdJob, ThingdLink,
    ThingdObject, ThingdOperation, ThingdOperationResult,
};

const MAX_SCOPED_SEARCH_SCAN: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeSubject {
    SharedTenant,
    User(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageScope {
    pub tenant_id: String,
    pub instance_id: String,
    pub subject: ScopeSubject,
}

impl StorageScope {
    pub fn shared(tenant_id: impl Into<String>, instance_id: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            instance_id: instance_id.into(),
            subject: ScopeSubject::SharedTenant,
        }
    }
    pub fn user(
        tenant_id: impl Into<String>,
        instance_id: impl Into<String>,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            instance_id: instance_id.into(),
            subject: ScopeSubject::User(subject.into()),
        }
    }
    fn token(&self) -> String {
        let subject = match &self.subject {
            ScopeSubject::SharedTenant => "shared".to_string(),
            ScopeSubject::User(value) => format!("user-{}", encode(value)),
        };
        format!(
            "__arqen_scope__/tenant-{}/instance-{}/{}",
            encode(&self.tenant_id),
            encode(&self.instance_id),
            subject
        )
    }
}

fn encode(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn scoped_error(message: impl Into<String>) -> AppError {
    AppError::new(ErrorKind::Authorization, message)
}
fn scoped_collection(scope: &str, collection: &str) -> String {
    format!("{scope}/collection/{}", encode(collection))
}
fn scoped_id(scope: &str, id: &str) -> String {
    format!("{scope}/id/{}", encode(id))
}

fn decode(value: &str) -> Result<String, AppError> {
    if !value.len().is_multiple_of(2) {
        return Err(scoped_error(
            "resource does not belong to this storage scope",
        ));
    }
    let bytes = (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).ok())
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| scoped_error("resource does not belong to this storage scope"))?;
    String::from_utf8(bytes)
        .map_err(|_| scoped_error("resource does not belong to this storage scope"))
}

fn public_id(scope: &str, id: &str) -> Result<String, AppError> {
    decode(
        id.strip_prefix(&format!("{scope}/id/"))
            .ok_or_else(|| scoped_error("resource does not belong to this storage scope"))?,
    )
}
fn logical_object(scope: &str, mut object: ThingdObject) -> Result<ThingdObject, AppError> {
    let prefix = format!("{scope}/collection/");
    object.collection = decode(
        object
            .collection
            .strip_prefix(&prefix)
            .ok_or_else(|| scoped_error("object does not belong to this storage scope"))?,
    )?;
    object.id = public_id(scope, &object.id)?;
    Ok(object)
}

pub struct ScopedThingdBackend {
    inner: Arc<dyn ThingdBackend>,
    scope: StorageScope,
}

impl ScopedThingdBackend {
    pub fn new(inner: Arc<dyn ThingdBackend>, scope: StorageScope) -> Self {
        Self { inner, scope }
    }
    pub fn tenant(
        inner: Arc<dyn ThingdBackend>,
        tenant_id: impl Into<String>,
        instance_id: impl Into<String>,
    ) -> Self {
        Self::new(inner, StorageScope::shared(tenant_id, instance_id))
    }
    pub fn user(
        inner: Arc<dyn ThingdBackend>,
        tenant_id: impl Into<String>,
        instance_id: impl Into<String>,
        subject: impl Into<String>,
    ) -> Self {
        Self::new(inner, StorageScope::user(tenant_id, instance_id, subject))
    }
    pub fn scope(&self) -> &StorageScope {
        &self.scope
    }
    fn namespace(&self) -> String {
        self.scope.token()
    }
}

#[async_trait]
impl ThingdBackend for ScopedThingdBackend {
    async fn get_object(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<Option<ThingdObject>, AppError> {
        let namespace = self.namespace();
        self.inner
            .get_object(
                &scoped_collection(&namespace, collection),
                &scoped_id(&namespace, id),
            )
            .await?
            .map(|object| logical_object(&namespace, object))
            .transpose()
    }
    async fn get_object_versioned(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<Option<crate::thingd::VersionedThingdObject>, AppError> {
        let namespace = self.namespace();
        self.inner
            .get_object_versioned(
                &scoped_collection(&namespace, collection),
                &scoped_id(&namespace, id),
            )
            .await?
            .map(|versioned| {
                Ok(crate::thingd::VersionedThingdObject {
                    object: logical_object(&namespace, versioned.object)?,
                    version: versioned.version,
                })
            })
            .transpose()
    }
    async fn put_object(
        &self,
        collection: &str,
        id: &str,
        data: Value,
    ) -> Result<ThingdObject, AppError> {
        let namespace = self.namespace();
        logical_object(
            &namespace,
            self.inner
                .put_object(
                    &scoped_collection(&namespace, collection),
                    &scoped_id(&namespace, id),
                    data,
                )
                .await?,
        )
    }
    async fn put_object_if_version(
        &self,
        collection: &str,
        id: &str,
        data: Value,
        expected_version: u64,
    ) -> Result<ThingdObject, AppError> {
        let namespace = self.namespace();
        logical_object(
            &namespace,
            self.inner
                .put_object_if_version(
                    &scoped_collection(&namespace, collection),
                    &scoped_id(&namespace, id),
                    data,
                    expected_version,
                )
                .await?,
        )
    }
    async fn delete_object(&self, collection: &str, id: &str) -> Result<(), AppError> {
        let namespace = self.namespace();
        self.inner
            .delete_object(
                &scoped_collection(&namespace, collection),
                &scoped_id(&namespace, id),
            )
            .await
    }
    async fn query_objects(
        &self,
        collection: &str,
        options: QueryOptions,
    ) -> Result<Vec<ThingdObject>, AppError> {
        let namespace = self.namespace();
        self.inner
            .query_objects(&scoped_collection(&namespace, collection), options)
            .await?
            .into_iter()
            .map(|object| logical_object(&namespace, object))
            .collect()
    }
    async fn count_objects(&self, collection: &str) -> Result<usize, AppError> {
        let namespace = self.namespace();
        self.inner
            .count_objects(&scoped_collection(&namespace, collection))
            .await
    }
    async fn batch_write(
        &self,
        operations: Vec<ThingdOperation>,
    ) -> Result<Vec<ThingdOperationResult>, AppError> {
        let namespace = self.namespace();
        self.inner
            .batch_write(
                operations
                    .into_iter()
                    .map(|operation| match operation {
                        ThingdOperation::Put {
                            collection,
                            id,
                            data,
                        } => ThingdOperation::Put {
                            collection: scoped_collection(&namespace, &collection),
                            id: scoped_id(&namespace, &id),
                            data,
                        },
                        ThingdOperation::Delete { collection, id } => ThingdOperation::Delete {
                            collection: scoped_collection(&namespace, &collection),
                            id: scoped_id(&namespace, &id),
                        },
                    })
                    .collect(),
            )
            .await
    }
    async fn append_event(
        &self,
        stream: &str,
        event_type: &str,
        data: Value,
    ) -> Result<ThingdEvent, AppError> {
        let namespace = self.namespace();
        let mut event = self
            .inner
            .append_event(&format!("{namespace}/event/{stream}"), event_type, data)
            .await?;
        event.stream = stream.to_string();
        Ok(event)
    }
    async fn read_events(
        &self,
        stream: &str,
        from: Option<String>,
        limit: usize,
    ) -> Result<Vec<ThingdEvent>, AppError> {
        let namespace = self.namespace();
        let mut events = self
            .inner
            .read_events(&format!("{namespace}/event/{stream}"), from, limit)
            .await?;
        for event in &mut events {
            event.stream = stream.to_string();
        }
        Ok(events)
    }
    async fn push_job(
        &self,
        queue: &str,
        payload: Value,
        max_retries: u32,
    ) -> Result<ThingdJob, AppError> {
        let namespace = self.namespace();
        let mut job = self
            .inner
            .push_job(&format!("{namespace}/queue/{queue}"), payload, max_retries)
            .await?;
        job.queue = queue.to_string();
        job.id = scoped_id(&namespace, &job.id);
        Ok(job)
    }
    async fn push_job_with_options(
        &self,
        queue: &str,
        payload: Value,
        max_retries: u32,
        options: crate::thingd::PushJobOptions,
    ) -> Result<ThingdJob, AppError> {
        let namespace = self.namespace();
        let mut job = self
            .inner
            .push_job_with_options(
                &format!("{namespace}/queue/{queue}"),
                payload,
                max_retries,
                options,
            )
            .await?;
        job.queue = queue.to_string();
        job.id = scoped_id(&namespace, &job.id);
        Ok(job)
    }
    async fn claim_job(
        &self,
        queue: &str,
        worker_id: &str,
        lease_seconds: u32,
    ) -> Result<Option<ThingdJob>, AppError> {
        let namespace = self.namespace();
        let mut job = self
            .inner
            .claim_job(
                &format!("{namespace}/queue/{queue}"),
                worker_id,
                lease_seconds,
            )
            .await?;
        if let Some(job) = &mut job {
            job.queue = queue.to_string();
            job.id = scoped_id(&namespace, &job.id);
        }
        Ok(job)
    }
    async fn complete_job(&self, queue: &str, job_id: &str) -> Result<(), AppError> {
        let namespace = self.namespace();
        self.inner
            .complete_job(
                &format!("{namespace}/queue/{queue}"),
                &public_id(&namespace, job_id)?,
            )
            .await
    }
    async fn nack_job(&self, queue: &str, job_id: &str) -> Result<(), AppError> {
        let namespace = self.namespace();
        self.inner
            .nack_job(
                &format!("{namespace}/queue/{queue}"),
                &public_id(&namespace, job_id)?,
            )
            .await
    }
    async fn dead_letter_job(&self, queue: &str, job_id: &str) -> Result<(), AppError> {
        let namespace = self.namespace();
        self.inner
            .dead_letter_job(
                &format!("{namespace}/queue/{queue}"),
                &public_id(&namespace, job_id)?,
            )
            .await
    }
    async fn search(&self, query: &str, options: SearchOptions) -> Result<SearchResults, AppError> {
        let namespace = self.namespace();
        let offset = options.offset;
        let limit = options.limit;
        let mut bounded = options;
        bounded.offset = 0;
        bounded.limit = MAX_SCOPED_SEARCH_SCAN.saturating_add(1);
        let mut results = self.inner.search(query, bounded).await?;
        if results.total > MAX_SCOPED_SEARCH_SCAN {
            return Err(AppError::new(
                ErrorKind::Unavailable,
                format!(
                    "scoped search exceeds the {}-object scan limit; narrow the query",
                    MAX_SCOPED_SEARCH_SCAN
                ),
            ));
        }
        let prefix = format!("{namespace}/collection/");
        results
            .items
            .retain(|object| object.collection.starts_with(&prefix));
        results.items = results
            .items
            .into_iter()
            .map(|object| logical_object(&namespace, object))
            .collect::<Result<Vec<_>, _>>()?;
        results.total = results.items.len();
        results.items = results.items.into_iter().skip(offset).take(limit).collect();
        Ok(results)
    }
    async fn create_link(
        &self,
        source_id: &str,
        target_id: &str,
        relation: &str,
    ) -> Result<ThingdLink, AppError> {
        let namespace = self.namespace();
        let mut link = self
            .inner
            .create_link(
                &scoped_id(&namespace, source_id),
                &scoped_id(&namespace, target_id),
                relation,
            )
            .await?;
        link.id = scoped_id(&namespace, &link.id);
        link.source_id = public_id(&namespace, &link.source_id)?;
        link.target_id = public_id(&namespace, &link.target_id)?;
        Ok(link)
    }
    async fn get_links(
        &self,
        source_id: &str,
        relation: Option<&str>,
    ) -> Result<Vec<ThingdLink>, AppError> {
        let namespace = self.namespace();
        self.inner
            .get_links(&scoped_id(&namespace, source_id), relation)
            .await?
            .into_iter()
            .map(|mut link| {
                link.id = scoped_id(&namespace, &link.id);
                link.source_id = public_id(&namespace, &link.source_id)?;
                link.target_id = public_id(&namespace, &link.target_id)?;
                Ok(link)
            })
            .collect()
    }
    async fn delete_link(&self, link_id: &str) -> Result<(), AppError> {
        let namespace = self.namespace();
        self.inner
            .delete_link(&public_id(&namespace, link_id)?)
            .await
    }
    async fn reset(&self) -> Result<(), AppError> {
        Err(AppError::new(
            ErrorKind::NotImpl,
            "scoped reset is unavailable; reset the underlying test backend explicitly",
        ))
    }
    async fn seed(&self) -> Result<(), AppError> {
        self.inner.seed().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thingd::MemoryThingdBackend;
    #[tokio::test]
    async fn tenant_scopes_cannot_read_each_other() {
        let inner: Arc<dyn ThingdBackend> = Arc::new(MemoryThingdBackend::new());
        let first = ScopedThingdBackend::tenant(inner.clone(), "tenant-a", "instance-1");
        let second = ScopedThingdBackend::tenant(inner, "tenant-b", "instance-1");
        first
            .put_object("movies", "m-1", serde_json::json!({"title":"A"}))
            .await
            .unwrap();
        assert!(second.get_object("movies", "m-1").await.unwrap().is_none());
    }
    #[tokio::test]
    async fn user_scope_preserves_logical_objects() {
        let inner: Arc<dyn ThingdBackend> = Arc::new(MemoryThingdBackend::new());
        let user = ScopedThingdBackend::user(inner, "tenant-a", "instance-1", "user-1");
        let object = user
            .put_object("movies", "m-1", serde_json::json!({"title":"A"}))
            .await
            .unwrap();
        assert_eq!(object.collection, "movies");
        assert_eq!(object.id, "m-1");
    }

    #[tokio::test]
    async fn scopes_cannot_search_other_tenants_or_instances() {
        let inner: Arc<dyn ThingdBackend> = Arc::new(MemoryThingdBackend::new());
        let first = ScopedThingdBackend::tenant(inner.clone(), "tenant-a", "instance-1");
        let other_tenant = ScopedThingdBackend::tenant(inner.clone(), "tenant-b", "instance-1");
        let other_instance = ScopedThingdBackend::tenant(inner, "tenant-a", "instance-2");
        first
            .put_object("notes", "one", serde_json::json!({"text":"needle"}))
            .await
            .unwrap();
        other_tenant
            .put_object("notes", "two", serde_json::json!({"text":"needle"}))
            .await
            .unwrap();
        other_instance
            .put_object("notes", "three", serde_json::json!({"text":"needle"}))
            .await
            .unwrap();

        let results = first
            .search(
                "needle",
                SearchOptions {
                    limit: 10,
                    offset: 0,
                    filters: vec![],
                },
            )
            .await
            .unwrap();
        assert_eq!(results.total, 1);
        assert_eq!(results.items[0].id, "one");
    }

    #[tokio::test]
    async fn scoped_search_fails_closed_above_scan_budget() {
        let inner = Arc::new(MemoryThingdBackend::new());
        for index in 0..=MAX_SCOPED_SEARCH_SCAN {
            inner
                .put_object(
                    "shared",
                    &format!("item-{index}"),
                    serde_json::json!({"text":"needle"}),
                )
                .await
                .unwrap();
        }
        let scope = ScopedThingdBackend::tenant(inner, "tenant-a", "instance-1");
        let error = scope
            .search(
                "needle",
                SearchOptions {
                    limit: 5,
                    offset: 0,
                    filters: vec![],
                },
            )
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Unavailable);
    }

    #[tokio::test]
    async fn scoped_conditional_writes_enforce_create_match_and_stale_conflict() {
        let inner: Arc<dyn ThingdBackend> = Arc::new(MemoryThingdBackend::new());
        let first = ScopedThingdBackend::tenant(inner.clone(), "tenant-a", "instance-1");
        let second = ScopedThingdBackend::tenant(inner, "tenant-b", "instance-1");
        first
            .put_object_if_version("items", "one", serde_json::json!({"v":1}), 0)
            .await
            .unwrap();
        let version = first
            .get_object_versioned("items", "one")
            .await
            .unwrap()
            .unwrap()
            .version;
        assert_eq!(version, 1);
        first
            .put_object_if_version("items", "one", serde_json::json!({"v":2}), version)
            .await
            .unwrap();
        let stale = first
            .put_object_if_version("items", "one", serde_json::json!({"v":3}), 1)
            .await
            .unwrap_err();
        assert_eq!(stale.kind, ErrorKind::Conflict);
        second
            .put_object_if_version("items", "one", serde_json::json!({"v":4}), 0)
            .await
            .unwrap();
        assert_eq!(
            first
                .get_object("items", "one")
                .await
                .unwrap()
                .unwrap()
                .data["v"],
            2
        );
    }

    #[tokio::test]
    async fn scope_isolation_covers_events_jobs_links_and_batches() {
        let inner: Arc<dyn ThingdBackend> = Arc::new(MemoryThingdBackend::new());
        let first = ScopedThingdBackend::user(inner.clone(), "tenant-a", "instance-1", "user-a");
        let second = ScopedThingdBackend::user(inner, "tenant-a", "instance-1", "user-b");

        first
            .append_event("activity", "created", serde_json::json!({"v":1}))
            .await
            .unwrap();
        assert!(
            second
                .read_events("activity", None, 10)
                .await
                .unwrap()
                .is_empty()
        );

        first
            .push_job("work", serde_json::json!({"v":1}), 3)
            .await
            .unwrap();
        assert!(
            second
                .claim_job("work", "worker", 30)
                .await
                .unwrap()
                .is_none()
        );

        first
            .create_link("source", "target", "related")
            .await
            .unwrap();
        assert!(second.get_links("source", None).await.unwrap().is_empty());

        first
            .batch_write(vec![ThingdOperation::Put {
                collection: "records".to_string(),
                id: "one".to_string(),
                data: serde_json::json!({"v":1}),
            }])
            .await
            .unwrap();
        assert!(second.get_object("records", "one").await.unwrap().is_none());
    }
}
