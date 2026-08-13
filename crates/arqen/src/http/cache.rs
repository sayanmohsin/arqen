//! Opt-in HTTP cache validators for safe, application-owned responses.

use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::Response;

/// Application-provided cache metadata for one response.
#[derive(Debug, Clone)]
pub struct HttpCachePolicy {
    pub etag: String,
    pub cache_control: String,
}

impl HttpCachePolicy {
    pub fn public(etag: impl Into<String>, max_age_seconds: u64) -> Self {
        Self {
            etag: etag.into(),
            cache_control: format!("public, max-age={max_age_seconds}"),
        }
    }

    pub fn private(etag: impl Into<String>) -> Self {
        Self {
            etag: etag.into(),
            cache_control: "private, no-cache".to_string(),
        }
    }
}

/// Apply an explicit cache policy and return 304 when the client's ETag matches.
pub async fn cache_headers(request: Request, next: Next) -> Response {
    let policy = request.extensions().get::<HttpCachePolicy>().cloned();
    let matching = policy.as_ref().is_some_and(|policy| {
        request
            .headers()
            .get(header::IF_NONE_MATCH)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value == policy.etag)
    });
    let mut response = next.run(request).await;
    if let Some(policy) = policy {
        if let Ok(value) = HeaderValue::from_str(&policy.etag) {
            response.headers_mut().insert(header::ETAG, value);
        }
        if let Ok(value) = HeaderValue::from_str(&policy.cache_control) {
            response.headers_mut().insert(header::CACHE_CONTROL, value);
        }
        response
            .headers_mut()
            .insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
        if matching {
            *response.status_mut() = StatusCode::NOT_MODIFIED;
            response.headers_mut().remove(header::CONTENT_LENGTH);
            *response.body_mut() = axum::body::Body::empty();
        }
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use tower::ServiceExt;

    #[tokio::test]
    async fn matching_etag_returns_not_modified() {
        let app = axum::Router::new()
            .route("/catalog", get(|| async { "catalog" }))
            .layer(axum::middleware::from_fn(cache_headers))
            .layer(axum::Extension(HttpCachePolicy::public("\"v1\"", 60)));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/catalog")
                    .header(header::IF_NONE_MATCH, "\"v1\"")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
        assert_eq!(response.headers()[header::ETAG], "\"v1\"");
    }
}
