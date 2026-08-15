use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;
use uuid::Uuid;

use crate::core::{AppError, ErrorKind};

use crate::thingd::traits::*;

/// HTTP client implementation of [`ThingdBackend`].
///
/// Proxies all operations to a remote thingd server over HTTP.
pub struct HttpThingdBackend {
    base_url: String,
    client: Client,
    auth_token: Option<String>,
    policy: HttpClientPolicy,
    concurrency: Arc<Semaphore>,
}

const DEFAULT_MAX_CONCURRENCY: usize = 16;

#[derive(Debug, Clone)]
pub struct HttpClientPolicy {
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub max_retries: u32,
    pub initial_backoff: Duration,
    /// Maximum wall-clock time spent retrying one HTTP operation.
    pub max_retry_duration: Duration,
    pub max_query_scan_objects: usize,
}

impl Default for HttpClientPolicy {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            max_retries: 2,
            initial_backoff: Duration::from_millis(50),
            max_retry_duration: Duration::from_secs(30),
            max_query_scan_objects: 100_000,
        }
    }
}

impl HttpThingdBackend {
    pub fn new(base_url: &str) -> Self {
        Self::with_policy(base_url, HttpClientPolicy::default())
    }

    pub fn with_policy(base_url: &str, policy: HttpClientPolicy) -> Self {
        let base = base_url.trim_end_matches('/').to_string();
        // Ensure the base URL ends with /v1 for the thingd sidecar REST API
        let base = if base.ends_with("/v1") {
            base
        } else {
            format!("{}/v1", base)
        };
        Self {
            base_url: base,
            client: Client::builder()
                .connect_timeout(policy.connect_timeout)
                .timeout(policy.request_timeout)
                .pool_max_idle_per_host(32)
                .build()
                .expect("valid reqwest client configuration"),
            auth_token: None,
            policy,
            concurrency: Arc::new(Semaphore::new(DEFAULT_MAX_CONCURRENCY)),
        }
    }

    pub fn with_auth(mut self, token: &str) -> Self {
        self.auth_token = Some(token.to_string());
        self
    }

    /// Bound the number of active requests sent to the remote Thingd server.
    pub fn with_max_concurrency(mut self, max_concurrency: usize) -> Self {
        self.concurrency = Arc::new(Semaphore::new(max_concurrency.max(1)));
        self
    }

    fn authenticated(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(token) = &self.auth_token {
            request.bearer_auth(token)
        } else {
            request
        }
    }

    fn status_kind(status: reqwest::StatusCode) -> ErrorKind {
        match status.as_u16() {
            401 => ErrorKind::Authentication,
            403 => ErrorKind::Authorization,
            404 => ErrorKind::NotFound,
            409 => ErrorKind::Conflict,
            429 => ErrorKind::RateLimited,
            408 | 504 => ErrorKind::Timeout,
            500..=599 => ErrorKind::Unavailable,
            _ => ErrorKind::External,
        }
    }

    async fn response_error(response: reqwest::Response) -> AppError {
        let status = response.status();
        let detail = response
            .json::<Value>()
            .await
            .ok()
            .and_then(|body| body["error"]["message"].as_str().map(str::to_owned))
            .unwrap_or_else(|| "thingd request failed".to_string());
        AppError::new(
            Self::status_kind(status),
            format!("HTTP error {}: {}", status.as_u16(), detail),
        )
    }

    fn retry_after(response: &reqwest::Response) -> Option<Duration> {
        response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.trim().parse::<u64>().ok())
            .map(Duration::from_secs)
    }

    fn retryable_status(status: reqwest::StatusCode) -> bool {
        status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS
    }

    async fn finish_with<R, F>(&self, mut build: F) -> Result<R, AppError>
    where
        R: for<'de> Deserialize<'de>,
        F: FnMut() -> reqwest::RequestBuilder,
    {
        let _permit =
            self.concurrency.acquire().await.map_err(|_| {
                AppError::new(ErrorKind::Unavailable, "thingd client is shutting down")
            })?;
        let deadline = Instant::now() + self.policy.max_retry_duration;
        for attempt in 0..=self.policy.max_retries {
            let response = match self.authenticated(build()).send().await {
                Ok(response) => response,
                Err(error)
                    if (error.is_timeout() || error.is_connect())
                        && attempt < self.policy.max_retries
                        && Instant::now() < deadline =>
                {
                    let delay = self
                        .policy
                        .initial_backoff
                        .saturating_mul(2u32.saturating_pow(attempt));
                    sleep(delay.min(deadline.saturating_duration_since(Instant::now()))).await;
                    continue;
                }
                Err(error) => {
                    let kind = if error.is_timeout() {
                        ErrorKind::Timeout
                    } else {
                        ErrorKind::Unavailable
                    };
                    return Err(AppError::new(kind, format!("HTTP request failed: {error}")));
                }
            };
            if response.status().is_success() {
                return response
                    .json::<Envelope<R>>()
                    .await
                    .map(|envelope| envelope.data)
                    .map_err(|error| {
                        AppError::new(
                            ErrorKind::Dependency,
                            format!("failed to parse thingd response: {error}"),
                        )
                    });
            }
            if Self::retryable_status(response.status())
                && attempt < self.policy.max_retries
                && Instant::now() < deadline
            {
                let delay = Self::retry_after(&response).unwrap_or_else(|| {
                    self.policy
                        .initial_backoff
                        .saturating_mul(2u32.saturating_pow(attempt))
                });
                let _ = response.bytes().await;
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    break;
                }
                sleep(delay.min(remaining)).await;
                continue;
            }
            return Err(Self::response_error(response).await);
        }
        Err(AppError::new(
            ErrorKind::Unavailable,
            "HTTP retry policy exhausted",
        ))
    }

    async fn get<R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<R, crate::core::AppError> {
        let url = format!("{}{}", self.base_url, path);
        self.finish_with(|| self.client.get(&url)).await
    }

    fn is_not_found(err: &crate::core::AppError) -> bool {
        err.kind == ErrorKind::NotFound
    }

    async fn post<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: T,
    ) -> Result<R, crate::core::AppError> {
        let url = format!("{}{}", self.base_url, path);
        let body = serde_json::to_value(body)
            .map_err(|error| AppError::new(ErrorKind::Validation, error.to_string()))?;
        let key = Uuid::new_v4().to_string();
        self.finish_with(|| {
            self.client
                .post(&url)
                .header("Idempotency-Key", &key)
                .json(&body)
        })
        .await
    }

    async fn put<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: T,
    ) -> Result<R, crate::core::AppError> {
        let url = format!("{}{}", self.base_url, path);
        let body = serde_json::to_value(body)
            .map_err(|error| AppError::new(ErrorKind::Validation, error.to_string()))?;
        let key = Uuid::new_v4().to_string();
        self.finish_with(|| {
            self.client
                .put(&url)
                .header("Idempotency-Key", &key)
                .json(&body)
        })
        .await
    }

    async fn delete<R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<R, crate::core::AppError> {
        let url = format!("{}{}", self.base_url, path);
        self.finish_with(|| self.client.delete(&url)).await
    }

    async fn delete_with_body<R: for<'de> Deserialize<'de>, T: Serialize>(
        &self,
        path: &str,
        body: T,
    ) -> Result<R, crate::core::AppError> {
        let url = format!("{}{}", self.base_url, path);
        let body = serde_json::to_value(body)
            .map_err(|error| AppError::new(ErrorKind::Validation, error.to_string()))?;
        let key = Uuid::new_v4().to_string();
        self.finish_with(|| {
            self.client
                .delete(&url)
                .header("Idempotency-Key", &key)
                .json(&body)
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Json, Router,
        extract::Query,
        http::{HeaderMap, HeaderValue, StatusCode},
        routing::{any, get},
    };
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tokio::net::TcpListener;

    async fn mock_objects_server(
        total: usize,
    ) -> (String, Arc<Mutex<Vec<usize>>>, tokio::task::JoinHandle<()>) {
        let requested_limits = Arc::new(Mutex::new(Vec::new()));
        let request_log = Arc::clone(&requested_limits);
        let app = Router::new().route(
            "/v1/objects",
            get(move |Query(query): Query<HashMap<String, String>>| {
                let request_log = Arc::clone(&request_log);
                async move {
                    let limit = query
                        .get("limit")
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or_default();
                    let offset = query
                        .get("offset")
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or_default();
                    request_log.lock().unwrap().push(limit);
                    let page = (offset..offset.saturating_add(limit).min(total))
                        .map(|index| {
                            json!({
                                "id": format!("title-{index}"),
                                "collection": "titles",
                                "body": { "index": index },
                                "createdAt": "2026-01-01T00:00:00Z",
                                "updatedAt": "2026-01-01T00:00:00Z"
                            })
                        })
                        .collect::<Vec<_>>();
                    Json(json!({ "data": page }))
                }
            }),
        );
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (format!("http://{address}"), requested_limits, server)
    }

    #[test]
    fn maps_public_http_statuses_to_stable_error_kinds() {
        let cases = [
            (401, ErrorKind::Authentication),
            (403, ErrorKind::Authorization),
            (404, ErrorKind::NotFound),
            (409, ErrorKind::Conflict),
            (429, ErrorKind::RateLimited),
            (408, ErrorKind::Timeout),
            (504, ErrorKind::Timeout),
            (503, ErrorKind::Unavailable),
            (400, ErrorKind::External),
        ];
        for (status, expected) in cases {
            assert_eq!(
                HttpThingdBackend::status_kind(reqwest::StatusCode::from_u16(status).unwrap()),
                expected
            );
        }
    }

    #[test]
    fn default_policy_is_bounded_and_read_retry_only() {
        let policy = HttpClientPolicy::default();
        assert_eq!(policy.max_retries, 2);
        assert!(policy.connect_timeout <= policy.request_timeout);
        assert!(policy.initial_backoff > Duration::ZERO);
    }

    #[tokio::test]
    async fn bounded_query_requests_only_the_requested_window() {
        let (base_url, requested_limits, server) = mock_objects_server(1_200).await;
        let backend = HttpThingdBackend::new(&base_url);

        let results = backend
            .query_objects(
                "titles",
                QueryOptions {
                    limit: Some(100),
                    offset: 40,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(results.len(), 100);
        assert_eq!(&*requested_limits.lock().unwrap(), &[140]);
        server.abort();
    }

    #[tokio::test]
    async fn unbounded_query_continues_paging_until_collection_end() {
        let (base_url, requested_limits, server) = mock_objects_server(1_200).await;
        let backend = HttpThingdBackend::new(&base_url);

        let results = backend
            .query_objects("titles", QueryOptions::default())
            .await
            .unwrap();

        assert_eq!(results.len(), 1_200);
        assert_eq!(&*requested_limits.lock().unwrap(), &[500, 500, 500]);
        server.abort();
    }

    #[tokio::test]
    async fn retries_503_mutation_with_retry_after_and_stable_idempotency_key() {
        let attempts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let keys = Arc::new(Mutex::new(Vec::new()));
        let attempts_for_route = Arc::clone(&attempts);
        let keys_for_route = Arc::clone(&keys);
        let app = Router::new().route(
            "/v1/objects/titles/title-1",
            any(move |headers: HeaderMap| {
                let attempts = Arc::clone(&attempts_for_route);
                let keys = Arc::clone(&keys_for_route);
                async move {
                    keys.lock().unwrap().push(
                        headers
                            .get("Idempotency-Key")
                            .and_then(|value| value.to_str().ok())
                            .unwrap_or_default()
                            .to_string(),
                    );
                    if attempts.fetch_add(1, std::sync::atomic::Ordering::Relaxed) == 0 {
                        let mut response_headers = HeaderMap::new();
                        response_headers.insert("Retry-After", HeaderValue::from_static("0"));
                        (
                            StatusCode::SERVICE_UNAVAILABLE,
                            response_headers,
                            Json(json!({ "error": { "message": "indexing" } })),
                        )
                    } else {
                        (
                            StatusCode::OK,
                            HeaderMap::new(),
                            Json(json!({
                                "data": {
                                    "id": "title-1",
                                    "collection": "titles",
                                    "createdAt": "2026-01-01T00:00:00Z",
                                    "updatedAt": "2026-01-01T00:00:00Z"
                                }
                            })),
                        )
                    }
                }
            }),
        );
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let policy = HttpClientPolicy {
            initial_backoff: Duration::ZERO,
            max_retry_duration: Duration::from_secs(1),
            ..Default::default()
        };
        let backend = HttpThingdBackend::with_policy(&format!("http://{address}"), policy);

        backend
            .put_object("titles", "title-1", json!({ "name": "Example" }))
            .await
            .unwrap();

        assert_eq!(attempts.load(std::sync::atomic::Ordering::Relaxed), 2);
        let keys = keys.lock().unwrap();
        assert_eq!(keys.len(), 2);
        assert!(!keys[0].is_empty());
        assert_eq!(keys[0], keys[1]);
        server.abort();
    }
}

#[derive(Deserialize)]
struct EmptyResponse {}

#[derive(Deserialize)]
struct Envelope<T> {
    data: T,
}

fn encode(value: &str) -> String {
    urlencoding::encode(value).into_owned()
}

fn parse_body(value: &Value) -> Value {
    match value {
        Value::String(text) => serde_json::from_str(text).unwrap_or_else(|_| value.clone()),
        _ => value.clone(),
    }
}

fn object_from_json(value: &Value) -> ThingdObject {
    ThingdObject {
        id: value["id"].as_str().unwrap_or_default().to_string(),
        collection: value["collection"].as_str().unwrap_or_default().to_string(),
        data: parse_body(value.get("body").unwrap_or(&Value::Null)),
        created_at: value["createdAt"].as_str().unwrap_or_default().to_string(),
        updated_at: value["updatedAt"].as_str().unwrap_or_default().to_string(),
    }
}

fn event_from_json(value: &Value) -> ThingdEvent {
    ThingdEvent {
        id: value["id"].as_str().unwrap_or_default().to_string(),
        stream: value["stream"].as_str().unwrap_or_default().to_string(),
        event_type: value["type"].as_str().unwrap_or_default().to_string(),
        data: Value::Null,
        timestamp: value["createdAt"].as_str().unwrap_or_default().to_string(),
    }
}

fn job_from_json(value: &Value) -> ThingdJob {
    let created_at = value["createdAt"].as_str().unwrap_or_default().to_string();
    let state = match value["status"].as_str() {
        Some("leased") => JobState::Leased,
        Some("completed") => JobState::Completed,
        Some("dead") => JobState::Dead,
        _ => JobState::Queued,
    };
    let payload = value["body"]
        .as_str()
        .and_then(|body| serde_json::from_str(body).ok())
        .unwrap_or(Value::Null);
    ThingdJob {
        id: value["id"].as_str().unwrap_or_default().to_string(),
        queue: value["queue"].as_str().unwrap_or_default().to_string(),
        payload,
        state,
        attempts: value["attempts"].as_u64().unwrap_or_default() as u32,
        max_retries: value["maxAttempts"].as_u64().unwrap_or(3) as u32,
        lease_expires_at: value["leaseExpiresAtMs"].as_i64().map(|ms| ms.to_string()),
        created_at: created_at.clone(),
        updated_at: created_at,
        available_at_ms: value["availableAtMs"].as_i64(),
    }
}

fn link_from_json(value: &Value) -> ThingdLink {
    ThingdLink {
        id: value["id"].as_str().unwrap_or_default().to_string(),
        source_id: value["fromRef"].as_str().unwrap_or_default().to_string(),
        target_id: value["toRef"].as_str().unwrap_or_default().to_string(),
        relation: value["linkType"].as_str().unwrap_or_default().to_string(),
        created_at: value["createdAt"].as_str().unwrap_or_default().to_string(),
    }
}

#[async_trait]
impl ThingdBackend for HttpThingdBackend {
    async fn get_object(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<Option<ThingdObject>, crate::core::AppError> {
        let path = format!("/objects/{}/{}", encode(collection), encode(id));
        match self.get::<Value>(&path).await {
            Ok(value) => Ok(Some(object_from_json(&value))),
            Err(e) => {
                if Self::is_not_found(&e) {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn put_object(
        &self,
        collection: &str,
        id: &str,
        data: serde_json::Value,
    ) -> Result<ThingdObject, crate::core::AppError> {
        let path = format!("/objects/{}/{}", encode(collection), encode(id));
        let value: Value = self.put(&path, data.clone()).await?;
        let mut object = object_from_json(&value);
        // thingd-server's PUT response is metadata-only; the GET response
        // includes the stored body. Preserve the backend contract by returning
        // the data supplied by the caller when the response omits it.
        if object.id.is_empty() {
            object.id = id.to_string();
        }
        if object.collection.is_empty() {
            object.collection = collection.to_string();
        }
        if object.data.is_null() {
            object.data = data;
        }
        Ok(object)
    }

    async fn delete_object(&self, collection: &str, id: &str) -> Result<(), crate::core::AppError> {
        let path = format!("/objects/{}/{}", encode(collection), encode(id));
        self.delete::<EmptyResponse>(&path).await?;
        Ok(())
    }

    async fn query_objects(
        &self,
        collection: &str,
        options: QueryOptions,
    ) -> Result<Vec<ThingdObject>, crate::core::AppError> {
        const PAGE_SIZE: usize = 500;
        let fetch_budget = options
            .limit
            .map(|limit| limit.saturating_add(options.offset));

        // A zero-sized result never needs a remote scan. For bounded queries,
        // fetch only the window needed to apply the client-side pagination.
        if options.limit == Some(0) {
            return Ok(Vec::new());
        }

        let mut objects = Vec::new();
        let mut offset = 0usize;
        loop {
            let request_limit = fetch_budget
                .map(|budget| budget.saturating_sub(objects.len()).min(PAGE_SIZE))
                .unwrap_or(PAGE_SIZE);
            if request_limit == 0 {
                break;
            }
            let mut path = format!(
                "/objects?collection={}&limit={}&offset={}",
                encode(collection),
                request_limit,
                offset
            );
            for filter in &options.filters {
                if let (FilterOperator::Eq, Some(value)) = (&filter.operator, filter.value.as_str())
                {
                    path.push_str(&format!(
                        "&filter.{}={}",
                        encode(&filter.field),
                        encode(value)
                    ));
                }
            }
            let values: Value = self.get(&path).await?;
            let page = values
                .as_array()
                .map(|items| items.iter().map(object_from_json).collect::<Vec<_>>())
                .unwrap_or_default();
            let page_len = page.len();
            objects.extend(page);
            if objects.len() > self.policy.max_query_scan_objects {
                return Err(crate::core::AppError::new(
                    ErrorKind::Validation,
                    format!(
                        "query scan exceeded the configured limit of {} objects",
                        self.policy.max_query_scan_objects
                    ),
                ));
            }
            if fetch_budget.is_some_and(|budget| objects.len() >= budget) {
                break;
            }
            if page_len < request_limit {
                break;
            }
            offset += request_limit;
        }
        let filtered = crate::thingd::traits::filter_objects(objects, &options.filters)?;
        Ok(filtered
            .into_iter()
            .skip(options.offset)
            .take(options.limit.unwrap_or(usize::MAX))
            .collect())
    }

    async fn count_objects(&self, collection: &str) -> Result<usize, crate::core::AppError> {
        let path = format!("/counts/objects/{}", encode(collection));
        let response: Value = self.get(&path).await?;
        Ok(response["count"].as_u64().unwrap_or_default() as usize)
    }

    async fn batch_write(
        &self,
        operations: Vec<ThingdOperation>,
    ) -> Result<Vec<ThingdOperationResult>, crate::core::AppError> {
        let mut results = vec![None; operations.len()];
        let mut puts: std::collections::HashMap<String, Vec<(usize, String, Value)>> =
            std::collections::HashMap::new();
        let mut deletes: std::collections::HashMap<String, Vec<(usize, String)>> =
            std::collections::HashMap::new();
        for (index, operation) in operations.into_iter().enumerate() {
            match operation {
                ThingdOperation::Put {
                    collection,
                    id,
                    data,
                } => puts.entry(collection).or_default().push((index, id, data)),
                ThingdOperation::Delete { collection, id } => {
                    deletes.entry(collection).or_default().push((index, id))
                }
            }
        }
        for (collection, objects) in puts {
            let body: Vec<Value> = objects
                .iter()
                .map(|(_, id, data)| {
                    let mut value = data.clone();
                    if let Some(object) = value.as_object_mut() {
                        object.insert("id".to_string(), Value::String(id.clone()));
                    }
                    value
                })
                .collect();
            let path = format!("/objects/batch?collection={}", encode(&collection));
            let result: Result<Value, AppError> = self.put(&path, body).await;
            for (index, _, _) in objects {
                results[index] = Some(match &result {
                    Ok(_) => ThingdOperationResult {
                        success: true,
                        error: None,
                    },
                    Err(error) => ThingdOperationResult {
                        success: false,
                        error: Some(error.message.clone()),
                    },
                });
            }
        }
        for (collection, ids) in deletes {
            let path = format!("/objects/batch?collection={}", encode(&collection));
            let ids_only: Vec<&str> = ids.iter().map(|(_, id)| id.as_str()).collect();
            let result: Result<Value, AppError> =
                self.delete_with_body(&path, json!(ids_only)).await;
            for (index, _) in ids {
                results[index] = Some(match &result {
                    Ok(_) => ThingdOperationResult {
                        success: true,
                        error: None,
                    },
                    Err(error) => ThingdOperationResult {
                        success: false,
                        error: Some(error.message.clone()),
                    },
                });
            }
        }
        Ok(results
            .into_iter()
            .map(|result| {
                result.unwrap_or(ThingdOperationResult {
                    success: false,
                    error: Some("batch operation did not return a result".to_string()),
                })
            })
            .collect())
    }

    async fn append_event(
        &self,
        stream: &str,
        event_type: &str,
        data: serde_json::Value,
    ) -> Result<ThingdEvent, crate::core::AppError> {
        let path = format!("/events/{}", encode(stream));
        let value: Value = self
            .post(&path, json!({ "type": event_type, "data": data.clone() }))
            .await?;
        let mut event = event_from_json(&value);
        event.stream = stream.to_string();
        event.event_type = event_type.to_string();
        event.data = data;
        Ok(event)
    }

    async fn read_events(
        &self,
        stream: &str,
        from: Option<String>,
        limit: usize,
    ) -> Result<Vec<ThingdEvent>, crate::core::AppError> {
        let path = format!("/events?stream={}&limit={}", encode(stream), limit);
        let path = if let Some(from_id) = from {
            format!("{}&fromSequence={}", path, encode(&from_id))
        } else {
            path
        };
        let values: Value = self.get(&path).await?;
        Ok(values
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .map(|value| {
                        let mut event = event_from_json(value);
                        event.stream = stream.to_string();
                        event
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    async fn push_job(
        &self,
        queue: &str,
        payload: serde_json::Value,
        max_retries: u32,
    ) -> Result<ThingdJob, crate::core::AppError> {
        self.push_job_with_options(queue, payload, max_retries, Default::default())
            .await
    }

    async fn push_job_with_options(
        &self,
        queue: &str,
        payload: serde_json::Value,
        max_retries: u32,
        options: crate::thingd::PushJobOptions,
    ) -> Result<ThingdJob, crate::core::AppError> {
        if options.idempotency_key.is_some() || options.delay_ms.is_some() {
            return Err(crate::core::AppError::new(
                crate::core::ErrorKind::NotImpl,
                "HTTP Thingd queue contract does not support deterministic or delayed enqueue",
            ));
        }
        let path = format!("/queues/{}/push", encode(queue));
        let value: Value = self
            .post(
                &path,
                json!({ "payload": payload, "maxAttempts": max_retries }),
            )
            .await?;
        Ok(job_from_json(&value))
    }

    async fn claim_job(
        &self,
        queue: &str,
        worker_id: &str,
        lease_seconds: u32,
    ) -> Result<Option<ThingdJob>, crate::core::AppError> {
        let path = format!("/queues/{}/claim", encode(queue));
        let value: Value = self
            .post(
                &path,
                json!({ "workerId": worker_id, "leaseMs": lease_seconds * 1000 }),
            )
            .await?;
        Ok((!value.is_null()).then(|| job_from_json(&value)))
    }

    async fn complete_job(&self, queue: &str, job_id: &str) -> Result<(), crate::core::AppError> {
        let path = format!("/queues/{}/ack", encode(queue));
        self.post::<_, EmptyResponse>(&path, json!({ "jobId": job_id }))
            .await?;
        Ok(())
    }

    async fn nack_job(&self, queue: &str, job_id: &str) -> Result<(), crate::core::AppError> {
        let path = format!("/queues/{}/nack", encode(queue));
        self.post::<_, EmptyResponse>(&path, json!({ "jobId": job_id }))
            .await?;
        Ok(())
    }

    async fn dead_letter_job(
        &self,
        queue: &str,
        job_id: &str,
    ) -> Result<(), crate::core::AppError> {
        let _ = (queue, job_id);
        Ok(())
    }

    async fn search(
        &self,
        query: &str,
        options: SearchOptions,
    ) -> Result<SearchResults, crate::core::AppError> {
        let values: Value = self
            .post(
                "/search",
                json!({ "query": query, "limit": usize::MAX, "offset": 0 }),
            )
            .await?;
        let objects = values
            .as_array()
            .map(|items| items.iter().map(object_from_json).collect::<Vec<_>>())
            .unwrap_or_default();
        let filtered = crate::thingd::traits::filter_objects(objects, &options.filters)?;
        let total = filtered.len();
        let items = filtered
            .into_iter()
            .skip(options.offset)
            .take(options.limit)
            .collect();
        Ok(SearchResults { total, items })
    }

    async fn create_link(
        &self,
        source_id: &str,
        target_id: &str,
        relation: &str,
    ) -> Result<ThingdLink, crate::core::AppError> {
        let path = "/links";
        let value: Value = self
            .post(
                path,
                json!({ "fromRef": source_id, "linkType": relation, "toRef": target_id }),
            )
            .await?;
        Ok(link_from_json(&value))
    }

    async fn get_links(
        &self,
        source_id: &str,
        relation: Option<&str>,
    ) -> Result<Vec<ThingdLink>, crate::core::AppError> {
        let mut path = format!("/links?reference={}&direction=Outgoing", encode(source_id));
        if let Some(relation) = relation {
            path.push_str(&format!("&linkType={}", encode(relation)));
        }
        let values: Value = self.get(&path).await?;
        Ok(values
            .as_array()
            .map(|items| items.iter().map(link_from_json).collect())
            .unwrap_or_default())
    }

    async fn delete_link(&self, link_id: &str) -> Result<(), crate::core::AppError> {
        let path = format!("/links/{}", link_id);
        self.delete::<EmptyResponse>(&path).await?;
        Ok(())
    }

    async fn reset(&self) -> Result<(), crate::core::AppError> {
        let path = "/admin/clear-default-db";
        self.post::<_, EmptyResponse>(path, serde_json::json!({}))
            .await?;
        Ok(())
    }

    async fn seed(&self) -> Result<(), crate::core::AppError> {
        let path = "/seed";
        self.post::<_, EmptyResponse>(path, serde_json::json!({}))
            .await?;
        Ok(())
    }
}
