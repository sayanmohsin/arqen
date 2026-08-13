use axum::extract::{MatchedPath, Request};
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tracing::info;

use crate::context::RequestContext;

#[derive(Debug, Clone)]
pub struct RequestLogConfig {
    pub success_sample_rate: f64,
    pub slow_request_threshold: Duration,
    pub service_name: String,
    pub environment: String,
}

impl Default for RequestLogConfig {
    fn default() -> Self {
        Self {
            success_sample_rate: 1.0,
            slow_request_threshold: Duration::from_millis(250),
            service_name: env!("CARGO_PKG_NAME").to_string(),
            environment: "development".to_string(),
        }
    }
}

static REQUEST_LOG_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();
    let request_id = request
        .extensions()
        .get::<crate::core::error::CorrelationId>()
        .map(ToString::to_string)
        .or_else(|| {
            request
                .headers()
                .get("x-request-id")
                .and_then(|value| value.to_str().ok())
                .filter(|value| {
                    !value.is_empty()
                        && value.len() <= 128
                        && value
                            .bytes()
                            .all(|byte| byte.is_ascii_graphic() && byte != b'"')
                })
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| crate::core::error::CorrelationId::new().to_string());
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(|matched| matched.as_str().to_owned())
        .unwrap_or_else(|| uri.path().to_owned());
    let context = request.extensions().get::<RequestContext>().cloned();
    let log_config = request
        .extensions()
        .get::<RequestLogConfig>()
        .cloned()
        .unwrap_or_default();

    let mut response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status().as_u16();

    let should_log = status >= 400
        || duration >= log_config.slow_request_threshold
        || sample_success(log_config.success_sample_rate);
    if should_log {
        info!(
        service = %log_config.service_name,
        service_version = env!("CARGO_PKG_VERSION"),
        environment = %log_config.environment,
        method = %method,
        route = %route,
        request_id = %request_id,
        correlation_id = %request_id,
        subject = context.as_ref().and_then(|value| value.subject.as_deref()).unwrap_or("anonymous"),
        tenant_id = context.as_ref().and_then(|value| value.tenant_id.as_deref()).unwrap_or("-"),
        instance_id = context.as_ref().and_then(|value| value.instance_id.as_deref()).unwrap_or("-"),
        status = status,
        duration_ms = duration.as_millis() as u64,
        duration_us = duration.as_micros() as u64,
        outcome = request_outcome(status),
            "request.completed"
        );
    }

    let header = HeaderValue::try_from(&request_id).expect("UUID is a valid header value");
    response.headers_mut().insert("x-request-id", header);
    response
}

fn sample_success(rate: f64) -> bool {
    if rate >= 1.0 {
        return true;
    }
    if rate <= 0.0 {
        return false;
    }
    let sequence = REQUEST_LOG_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    (sequence % 10_000) < (rate * 10_000.0) as u64
}

fn request_outcome(status: u16) -> &'static str {
    match status {
        200..=399 => "success",
        408 | 504 => "timeout",
        400..=499 => "client_error",
        502 => "dependency_error",
        500..=599 => "server_error",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::response::IntoResponse;
    use axum::routing::get;
    use tower::ServiceExt;

    async fn handler() -> impl IntoResponse {
        StatusCode::OK
    }

    #[tokio::test]
    async fn test_logging_middleware_adds_request_id() {
        let app = Router::new()
            .route("/test", get(handler))
            .layer(axum::middleware::from_fn(logging_middleware));

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key("x-request-id"));
    }

    #[tokio::test]
    async fn test_logging_middleware_preserves_existing_request_id() {
        let app = Router::new()
            .route("/test", get(handler))
            .layer(axum::middleware::from_fn(logging_middleware));

        let request = Request::builder()
            .uri("/test")
            .header("x-request-id", "custom-id-123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let request_id = response
            .headers()
            .get("x-request-id")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(request_id, "custom-id-123");
    }

    #[test]
    fn test_request_outcome_classification() {
        assert_eq!(request_outcome(200), "success");
        assert_eq!(request_outcome(504), "timeout");
        assert_eq!(request_outcome(502), "dependency_error");
        assert_eq!(request_outcome(500), "server_error");
    }

    #[test]
    fn test_sample_success_bounds() {
        assert!(sample_success(1.0));
        assert!(!sample_success(0.0));
    }
}
