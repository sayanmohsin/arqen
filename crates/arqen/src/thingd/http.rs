use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::thingd::traits::*;

pub struct HttpThingdBackend {
    base_url: String,
    client: Client,
    auth_token: Option<String>,
}

impl HttpThingdBackend {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
            auth_token: None,
        }
    }

    pub fn with_auth(mut self, token: &str) -> Self {
        self.auth_token = Some(token.to_string());
        self
    }

    async fn get<R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<R, crate::core::AppError> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.get(&url);
        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }
        let response = request.send().await.map_err(|e| {
            crate::core::AppError::new(
                crate::core::ErrorKind::External,
                format!("HTTP request failed: {}", e),
            )
        })?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(crate::core::AppError::new(
                crate::core::ErrorKind::External,
                format!("HTTP error {}: {}", status, text),
            ));
        }
        response.json().await.map_err(|e| {
            crate::core::AppError::new(
                crate::core::ErrorKind::External,
                format!("Failed to parse response: {}", e),
            )
        })
    }

    async fn post<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: T,
    ) -> Result<R, crate::core::AppError> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.post(&url).json(&body);
        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }
        let response = request.send().await.map_err(|e| {
            crate::core::AppError::new(
                crate::core::ErrorKind::External,
                format!("HTTP request failed: {}", e),
            )
        })?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(crate::core::AppError::new(
                crate::core::ErrorKind::External,
                format!("HTTP error {}: {}", status, text),
            ));
        }
        response.json().await.map_err(|e| {
            crate::core::AppError::new(
                crate::core::ErrorKind::External,
                format!("Failed to parse response: {}", e),
            )
        })
    }

    async fn put<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: T,
    ) -> Result<R, crate::core::AppError> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.put(&url).json(&body);
        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }
        let response = request.send().await.map_err(|e| {
            crate::core::AppError::new(
                crate::core::ErrorKind::External,
                format!("HTTP request failed: {}", e),
            )
        })?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(crate::core::AppError::new(
                crate::core::ErrorKind::External,
                format!("HTTP error {}: {}", status, text),
            ));
        }
        response.json().await.map_err(|e| {
            crate::core::AppError::new(
                crate::core::ErrorKind::External,
                format!("Failed to parse response: {}", e),
            )
        })
    }

    async fn delete<R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<R, crate::core::AppError> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.delete(&url);
        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }
        let response = request.send().await.map_err(|e| {
            crate::core::AppError::new(
                crate::core::ErrorKind::External,
                format!("HTTP request failed: {}", e),
            )
        })?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(crate::core::AppError::new(
                crate::core::ErrorKind::External,
                format!("HTTP error {}: {}", status, text),
            ));
        }
        response.json().await.map_err(|e| {
            crate::core::AppError::new(
                crate::core::ErrorKind::External,
                format!("Failed to parse response: {}", e),
            )
        })
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
                if e.to_string().contains("404") {
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
        filter: Option<ThingdFilter>,
    ) -> Result<Vec<ThingdObject>, crate::core::AppError> {
        let path = format!("/collections/{}/objects", collection);
        let body = serde_json::json!({ "filter": filter });
        self.post(&path, body).await
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
                if e.to_string().contains("404") || e.to_string().contains("no jobs") {
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
