use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

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
}

#[derive(Debug, Clone)]
pub struct HttpClientPolicy {
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub max_retries: u32,
    pub initial_backoff: Duration,
}

impl Default for HttpClientPolicy {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            max_retries: 2,
            initial_backoff: Duration::from_millis(50),
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
        }
    }

    pub fn with_auth(mut self, token: &str) -> Self {
        self.auth_token = Some(token.to_string());
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
        let body = response.text().await.unwrap_or_default();
        let detail: String = body.chars().take(512).collect();
        AppError::new(
            Self::status_kind(status),
            format!("HTTP error {}: {}", status.as_u16(), detail),
        )
    }

    async fn finish<R: for<'de> Deserialize<'de>>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<R, AppError> {
        let response = self.authenticated(request).send().await.map_err(|error| {
            let kind = if error.is_timeout() {
                ErrorKind::Timeout
            } else {
                ErrorKind::Unavailable
            };
            AppError::new(kind, format!("HTTP request failed: {error}"))
        })?;
        if !response.status().is_success() {
            return Err(Self::response_error(response).await);
        }
        response.json().await.map_err(|error| {
            AppError::new(
                ErrorKind::Dependency,
                format!("failed to parse thingd response: {error}"),
            )
        })
    }

    async fn get<R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<R, crate::core::AppError> {
        let url = format!("{}{}", self.base_url, path);
        for attempt in 0..=self.policy.max_retries {
            let request = self.authenticated(self.client.get(&url));
            match request.send().await {
                Ok(response) if response.status().is_success() => {
                    return response.json().await.map_err(|error| {
                        AppError::new(
                            ErrorKind::Dependency,
                            format!("failed to parse thingd response: {error}"),
                        )
                    });
                }
                Ok(response)
                    if (response.status().is_server_error()
                        || response.status().as_u16() == 429)
                        && attempt < self.policy.max_retries =>
                {
                    let _ = response.bytes().await;
                    sleep(
                        self.policy
                            .initial_backoff
                            .saturating_mul(2u32.saturating_pow(attempt)),
                    )
                    .await;
                }
                Ok(response) => return Err(Self::response_error(response).await),
                Err(error)
                    if (error.is_timeout() || error.is_connect())
                        && attempt < self.policy.max_retries =>
                {
                    sleep(
                        self.policy
                            .initial_backoff
                            .saturating_mul(2u32.saturating_pow(attempt)),
                    )
                    .await;
                }
                Err(error) => {
                    let kind = if error.is_timeout() {
                        ErrorKind::Timeout
                    } else {
                        ErrorKind::Unavailable
                    };
                    return Err(AppError::new(kind, format!("HTTP request failed: {error}")));
                }
            }
        }
        Err(AppError::new(
            ErrorKind::Unavailable,
            "HTTP retry policy exhausted",
        ))
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
        self.finish(self.client.post(&url).json(&body)).await
    }

    async fn put<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: T,
    ) -> Result<R, crate::core::AppError> {
        let url = format!("{}{}", self.base_url, path);
        self.finish(self.client.put(&url).json(&body)).await
    }

    async fn delete<R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<R, crate::core::AppError> {
        let url = format!("{}{}", self.base_url, path);
        self.finish(self.client.delete(&url)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

#[derive(Deserialize)]
struct EmptyResponse {}

#[derive(Deserialize)]
struct CountResponse {
    count: usize,
}

#[async_trait]
impl ThingdBackend for HttpThingdBackend {
    async fn get_object(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<Option<ThingdObject>, crate::core::AppError> {
        let path = format!("/collections/{}/objects/{}", collection, id);
        match self.get::<ThingdObject>(&path).await {
            Ok(obj) => Ok(Some(obj)),
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
        let path = format!("/collections/{}/objects/{}", collection, id);
        let body = serde_json::json!({ "data": data });
        self.put(&path, body).await
    }

    async fn delete_object(&self, collection: &str, id: &str) -> Result<(), crate::core::AppError> {
        let path = format!("/collections/{}/objects/{}", collection, id);
        self.delete::<EmptyResponse>(&path).await?;
        Ok(())
    }

    async fn query_objects(
        &self,
        collection: &str,
        options: QueryOptions,
    ) -> Result<Vec<ThingdObject>, crate::core::AppError> {
        let path = format!("/collections/{}/objects", collection);
        self.post(&path, options).await
    }

    async fn count_objects(&self, collection: &str) -> Result<usize, crate::core::AppError> {
        let path = format!("/collections/{}/count", collection);
        let response: CountResponse = self.get(&path).await?;
        Ok(response.count)
    }

    async fn batch_write(
        &self,
        operations: Vec<ThingdOperation>,
    ) -> Result<Vec<ThingdOperationResult>, crate::core::AppError> {
        let path = "/batch";
        self.post(path, operations).await
    }

    async fn append_event(
        &self,
        stream: &str,
        event_type: &str,
        data: serde_json::Value,
    ) -> Result<ThingdEvent, crate::core::AppError> {
        let path = format!("/streams/{}/events", stream);
        let body = serde_json::json!({ "event_type": event_type, "data": data });
        self.post(&path, body).await
    }

    async fn read_events(
        &self,
        stream: &str,
        from: Option<String>,
        limit: usize,
    ) -> Result<Vec<ThingdEvent>, crate::core::AppError> {
        let path = format!("/streams/{}/events?limit={}", stream, limit);
        let path = if let Some(from_id) = from {
            format!("{}&from={}", path, from_id)
        } else {
            path
        };
        self.get(&path).await
    }

    async fn push_job(
        &self,
        queue: &str,
        payload: serde_json::Value,
        max_retries: u32,
    ) -> Result<ThingdJob, crate::core::AppError> {
        let path = format!("/queues/{}/jobs", queue);
        let body = serde_json::json!({ "payload": payload, "max_retries": max_retries });
        self.post(&path, body).await
    }

    async fn claim_job(
        &self,
        queue: &str,
        worker_id: &str,
        lease_seconds: u32,
    ) -> Result<Option<ThingdJob>, crate::core::AppError> {
        let path = format!("/queues/{}/claim", queue);
        let body = serde_json::json!({ "worker_id": worker_id, "lease_seconds": lease_seconds });
        match self.post::<_, Option<ThingdJob>>(&path, body).await {
            Ok(job) => Ok(job),
            Err(e) => {
                if Self::is_not_found(&e) || e.message.contains("no jobs") {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn complete_job(&self, queue: &str, job_id: &str) -> Result<(), crate::core::AppError> {
        let path = format!("/queues/{}/jobs/{}/complete", queue, job_id);
        self.post::<_, EmptyResponse>(&path, serde_json::json!({}))
            .await?;
        Ok(())
    }

    async fn nack_job(&self, queue: &str, job_id: &str) -> Result<(), crate::core::AppError> {
        let path = format!("/queues/{}/jobs/{}/nack", queue, job_id);
        self.post::<_, EmptyResponse>(&path, serde_json::json!({}))
            .await?;
        Ok(())
    }

    async fn dead_letter_job(
        &self,
        queue: &str,
        job_id: &str,
    ) -> Result<(), crate::core::AppError> {
        let path = format!("/queues/{}/jobs/{}/dead", queue, job_id);
        self.post::<_, EmptyResponse>(&path, serde_json::json!({}))
            .await?;
        Ok(())
    }

    async fn search(
        &self,
        query: &str,
        options: SearchOptions,
    ) -> Result<SearchResults, crate::core::AppError> {
        let path = "/search";
        let body = serde_json::json!({ "query": query, "options": options });
        self.post(path, body).await
    }

    async fn create_link(
        &self,
        source_id: &str,
        target_id: &str,
        relation: &str,
    ) -> Result<ThingdLink, crate::core::AppError> {
        let path = "/links";
        let body = serde_json::json!({ "source_id": source_id, "target_id": target_id, "relation": relation });
        self.post(path, body).await
    }

    async fn get_links(
        &self,
        source_id: &str,
        relation: Option<&str>,
    ) -> Result<Vec<ThingdLink>, crate::core::AppError> {
        let path = match relation {
            Some(r) => format!("/links/{}?relation={}", source_id, r),
            None => format!("/links/{}", source_id),
        };
        self.get(&path).await
    }

    async fn delete_link(&self, link_id: &str) -> Result<(), crate::core::AppError> {
        let path = format!("/links/{}", link_id);
        self.delete::<EmptyResponse>(&path).await?;
        Ok(())
    }

    async fn reset(&self) -> Result<(), crate::core::AppError> {
        let path = "/reset";
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
