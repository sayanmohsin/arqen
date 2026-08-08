use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use tracing::info;
use uuid::Uuid;

use crate::context::RequestContext;

pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map_or_else(|| Uuid::new_v4().to_string(), ToOwned::to_owned);
    let context = request.extensions().get::<RequestContext>().cloned();

    let mut response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status().as_u16();

    info!(
        method = %method,
        uri = %uri,
        request_id = %request_id,
        subject = context.as_ref().and_then(|value| value.subject.as_deref()).unwrap_or("anonymous"),
        tenant_id = context.as_ref().and_then(|value| value.tenant_id.as_deref()).unwrap_or("-"),
        instance_id = context.as_ref().and_then(|value| value.instance_id.as_deref()).unwrap_or("-"),
        status = status,
        duration_ms = duration.as_millis() as u64,
        "Request completed"
    );

    let header = HeaderValue::try_from(&request_id).expect("UUID is a valid header value");
    response.headers_mut().insert("x-request-id", header);
    response
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
}
