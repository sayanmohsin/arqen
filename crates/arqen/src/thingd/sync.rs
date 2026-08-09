//! Provider-neutral Thingd 0.77 replication client.
//!
//! Arqen exposes the public Thingd replication contract as a typed lifecycle
//! boundary. It does not implement replication semantics, conflict resolution,
//! tombstones, or provenance itself.

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::sleep;

use crate::core::{AppError, ErrorKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicationChange {
    pub source_id: String,
    pub cursor: u64,
    pub idempotency_key: String,
    pub change: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicationStatus {
    pub source_id: String,
    pub provider: String,
    pub project_id: String,
    pub instance_slug: String,
    pub role: String,
    pub latest_cursor: u64,
    pub change_count: u64,
    pub last_applied_cursor: u64,
    pub quarantined_conflicts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicationSnapshot {
    pub source_id: String,
    pub cursor: u64,
    pub objects: Vec<Value>,
    pub events: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct SyncClientPolicy {
    pub request_timeout: Duration,
    pub max_retries: u32,
    pub initial_backoff: Duration,
}

impl Default for SyncClientPolicy {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            max_retries: 2,
            initial_backoff: Duration::from_millis(100),
        }
    }
}

#[derive(Clone)]
pub struct ThingdSyncClient {
    base_url: String,
    client: Client,
    auth_token: Option<String>,
    policy: SyncClientPolicy,
}

#[async_trait]
pub trait SyncEndpoint: Send + Sync {
    async fn events(&self, after: u64, limit: u32) -> Result<SyncPage, AppError>;
    async fn apply(&self, changes: &[ReplicationChange]) -> Result<ApplyResult, AppError>;
    async fn status(&self, source_id: Option<&str>) -> Result<ReplicationStatus, AppError>;
    async fn conflicts(&self) -> Result<Vec<Value>, AppError>;
    async fn snapshot(&self) -> Result<ReplicationSnapshot, AppError>;
    async fn apply_snapshot(
        &self,
        snapshot: &ReplicationSnapshot,
        replace: bool,
    ) -> Result<ApplyResult, AppError>;
}

#[async_trait]
pub trait SyncCheckpointStore: Send + Sync {
    async fn load(&self) -> Result<u64, AppError>;
    async fn save(&self, cursor: u64) -> Result<(), AppError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPage {
    pub source_id: String,
    pub after: u64,
    pub next: u64,
    pub changes: Vec<ReplicationChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    pub applied: u64,
    pub skipped: u64,
    pub conflicts: Vec<Value>,
    pub last_applied_cursor: u64,
}

impl ThingdSyncClient {
    pub fn new(base_url: &str) -> Self {
        Self::with_policy(base_url, SyncClientPolicy::default())
    }

    pub fn with_policy(base_url: &str, policy: SyncClientPolicy) -> Self {
        let base_url = base_url.trim_end_matches('/');
        let base_url = if base_url.ends_with("/v1") {
            base_url.to_string()
        } else {
            format!("{base_url}/v1")
        };
        Self {
            base_url,
            client: Client::builder()
                .timeout(policy.request_timeout)
                .pool_max_idle_per_host(16)
                .build()
                .expect("valid Thingd sync HTTP client"),
            auth_token: None,
            policy,
        }
    }

    pub fn with_auth(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    async fn request<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&T>,
    ) -> Result<R, AppError> {
        let url = format!("{}{}", self.base_url, path);
        for attempt in 0..=self.policy.max_retries {
            let mut request = self.client.request(method.clone(), &url);
            if let Some(token) = &self.auth_token {
                request = request.bearer_auth(token);
            }
            if let Some(body) = body {
                request = request.json(body);
            }
            match request.send().await {
                Ok(response) if response.status().is_success() => {
                    return response
                        .json::<Envelope<R>>()
                        .await
                        .map(|envelope| envelope.data)
                        .map_err(|error| AppError::new(ErrorKind::Dependency, error.to_string()));
                }
                Ok(response)
                    if (response.status() == StatusCode::TOO_MANY_REQUESTS
                        || response.status().is_server_error())
                        && attempt < self.policy.max_retries =>
                {
                    let _ = response.bytes().await;
                    sleep(
                        self.policy
                            .initial_backoff
                            .saturating_mul(2_u32.saturating_pow(attempt)),
                    )
                    .await;
                }
                Ok(response) => return Err(Self::status_error(response).await),
                Err(error)
                    if attempt < self.policy.max_retries
                        && (error.is_timeout() || error.is_connect()) =>
                {
                    sleep(
                        self.policy
                            .initial_backoff
                            .saturating_mul(2_u32.saturating_pow(attempt)),
                    )
                    .await;
                }
                Err(error) => {
                    return Err(AppError::new(
                        if error.is_timeout() {
                            ErrorKind::Timeout
                        } else {
                            ErrorKind::Unavailable
                        },
                        "Thingd sync request failed",
                    ));
                }
            }
        }
        Err(AppError::new(
            ErrorKind::Unavailable,
            "Thingd sync retry policy exhausted",
        ))
    }

    async fn status_error(response: reqwest::Response) -> AppError {
        let status = response.status();
        let kind = match status.as_u16() {
            401 => ErrorKind::Authentication,
            403 => ErrorKind::Authorization,
            409 => ErrorKind::Conflict,
            408 | 504 => ErrorKind::Timeout,
            429 => ErrorKind::RateLimited,
            500..=599 => ErrorKind::Unavailable,
            _ => ErrorKind::External,
        };
        AppError::new(kind, format!("Thingd sync HTTP status {}", status.as_u16()))
    }

    pub async fn current_schema(&self) -> Result<Option<Value>, AppError> {
        self.request(
            reqwest::Method::GET,
            "/schema/current",
            None::<&Value>.as_ref(),
        )
        .await
    }

    pub async fn validate_schema(&self, source: &str) -> Result<Value, AppError> {
        self.request(
            reqwest::Method::POST,
            "/schema/validate",
            Some(&serde_json::json!({ "source": source })),
        )
        .await
    }

    pub async fn migrations(&self) -> Result<Vec<Value>, AppError> {
        self.request(reqwest::Method::GET, "/migrations", None::<&Value>.as_ref())
            .await
    }
}

#[derive(Deserialize)]
struct Envelope<T> {
    data: T,
}

#[async_trait]
impl SyncEndpoint for ThingdSyncClient {
    async fn events(&self, after: u64, limit: u32) -> Result<SyncPage, AppError> {
        self.request(
            reqwest::Method::GET,
            &format!("/replication/events?after={after}&limit={limit}"),
            None::<&Value>.as_ref(),
        )
        .await
    }

    async fn apply(&self, changes: &[ReplicationChange]) -> Result<ApplyResult, AppError> {
        self.request(
            reqwest::Method::POST,
            "/replication/apply",
            Some(&serde_json::json!({ "changes": changes })),
        )
        .await
    }

    async fn status(&self, source_id: Option<&str>) -> Result<ReplicationStatus, AppError> {
        let path = source_id
            .map(|source| format!("/replication/status?sourceId={source}"))
            .unwrap_or_else(|| "/replication/status".to_string());
        self.request(reqwest::Method::GET, &path, None::<&Value>.as_ref())
            .await
    }

    async fn conflicts(&self) -> Result<Vec<Value>, AppError> {
        self.request(
            reqwest::Method::GET,
            "/replication/conflicts",
            None::<&Value>.as_ref(),
        )
        .await
    }

    async fn snapshot(&self) -> Result<ReplicationSnapshot, AppError> {
        self.request(
            reqwest::Method::GET,
            "/replication/snapshot",
            None::<&Value>.as_ref(),
        )
        .await
    }

    async fn apply_snapshot(
        &self,
        snapshot: &ReplicationSnapshot,
        replace: bool,
    ) -> Result<ApplyResult, AppError> {
        self.request(
            reqwest::Method::POST,
            "/replication/snapshot",
            Some(&serde_json::json!({ "snapshot": snapshot, "replace": replace })),
        )
        .await
    }
}

pub struct ThingdSyncWorker<S: SyncEndpoint + 'static, T: SyncEndpoint + 'static> {
    source: Arc<S>,
    target: Arc<T>,
    cursor: Arc<AtomicU64>,
    batch_size: u32,
    checkpoint: Option<Arc<dyn SyncCheckpointStore>>,
    collections: Option<Arc<Vec<String>>>,
    snapshot_fallback: bool,
}

impl<S: SyncEndpoint + 'static, T: SyncEndpoint + 'static> ThingdSyncWorker<S, T> {
    pub fn new(source: Arc<S>, target: Arc<T>, cursor: u64, batch_size: u32) -> Self {
        Self {
            source,
            target,
            cursor: Arc::new(AtomicU64::new(cursor)),
            batch_size: batch_size.clamp(1, 1000),
            checkpoint: None,
            collections: None,
            snapshot_fallback: true,
        }
    }

    pub fn with_checkpoint(mut self, checkpoint: Arc<dyn SyncCheckpointStore>) -> Self {
        self.checkpoint = Some(checkpoint);
        self
    }

    pub fn with_collection_allowlist(mut self, collections: Vec<String>) -> Self {
        self.collections = Some(Arc::new(collections));
        self
    }

    pub fn with_snapshot_fallback(mut self, enabled: bool) -> Self {
        self.snapshot_fallback = enabled;
        self
    }

    pub fn cursor(&self) -> u64 {
        self.cursor.load(Ordering::Acquire)
    }

    pub async fn run_once(&self) -> Result<ApplyResult, AppError> {
        if self.cursor() == 0
            && let Some(checkpoint) = &self.checkpoint
        {
            self.cursor
                .store(checkpoint.load().await?, Ordering::Release);
        }
        let page = match self.source.events(self.cursor(), self.batch_size).await {
            Ok(page) => page,
            Err(error) if error.kind == ErrorKind::Conflict && self.snapshot_fallback => {
                let snapshot = self.source.snapshot().await?;
                let result = self.target.apply_snapshot(&snapshot, true).await?;
                self.cursor.store(snapshot.cursor, Ordering::Release);
                if let Some(checkpoint) = &self.checkpoint {
                    checkpoint.save(snapshot.cursor).await?;
                }
                return Ok(result);
            }
            Err(error) => return Err(error),
        };
        if page.changes.is_empty() {
            return Ok(ApplyResult {
                applied: 0,
                skipped: 0,
                conflicts: Vec::new(),
                last_applied_cursor: self.cursor(),
            });
        }
        let changes = if let Some(allowlist) = &self.collections {
            page.changes
                .iter()
                .filter(|item| {
                    item.change
                        .get("collection")
                        .and_then(Value::as_str)
                        .is_some_and(|collection| {
                            allowlist.iter().any(|allowed| allowed == collection)
                        })
                })
                .cloned()
                .collect::<Vec<_>>()
        } else {
            page.changes.clone()
        };
        if changes.is_empty() {
            self.cursor.store(page.next, Ordering::Release);
            if let Some(checkpoint) = &self.checkpoint {
                checkpoint.save(page.next).await?;
            }
            return Ok(ApplyResult {
                applied: 0,
                skipped: page.changes.len() as u64,
                conflicts: Vec::new(),
                last_applied_cursor: page.next,
            });
        }
        let result = self.target.apply(&changes).await?;
        self.cursor
            .store(result.last_applied_cursor.max(page.next), Ordering::Release);
        if let Some(checkpoint) = &self.checkpoint {
            checkpoint.save(self.cursor()).await?;
        }
        Ok(result)
    }

    pub async fn run_until_shutdown(
        &self,
        mut shutdown: watch::Receiver<bool>,
        interval: Duration,
    ) -> Result<(), AppError> {
        loop {
            if *shutdown.borrow() {
                return Ok(());
            }
            self.run_once().await?;
            tokio::select! {
                _ = tokio::time::sleep(interval) => {},
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() { return Ok(()); }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thingd_replication_wire_names_are_camel_case() {
        let change = ReplicationChange {
            source_id: "local".into(),
            cursor: 7,
            idempotency_key: "local:7".into(),
            change: serde_json::json!({"operation": "object.upsert"}),
        };
        let value = serde_json::to_value(change).unwrap();
        assert_eq!(value["sourceId"], "local");
        assert_eq!(value["idempotencyKey"], "local:7");
        assert!(value.get("source_id").is_none());
    }

    #[test]
    fn apply_result_reads_thingd_response_names() {
        let result: ApplyResult = serde_json::from_value(serde_json::json!({
            "applied": 1,
            "skipped": 0,
            "conflicts": [],
            "lastAppliedCursor": 9
        }))
        .unwrap();
        assert_eq!(result.last_applied_cursor, 9);
    }
}
