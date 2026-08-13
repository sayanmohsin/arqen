use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};

use crate::core::error::{CorrelationId, REQUEST_CORRELATION_ID};

/// Header name for correlation ID.
pub const X_REQUEST_ID: &str = "X-Request-Id";
const MAX_CORRELATION_ID_LENGTH: usize = 128;

/// Middleware that generates or extracts correlation ID from request header.
///
/// If the request contains an `X-Request-Id` header, it's used as the correlation ID.
/// Otherwise, a new UUID is generated.
///
/// The correlation ID is added to:
/// - Request extensions (accessible via `Extension<CorrelationId>`)
/// - Response header (`X-Request-Id`)
///
/// The request is wrapped in the [`REQUEST_CORRELATION_ID`] task-local scope so
/// error responses generated while handling the request propagate the same ID.
pub async fn correlation_id_middleware(mut request: Request, next: Next) -> Response {
    let correlation_id = extract_or_generate_correlation_id(&request);

    // Insert into request extensions
    request.extensions_mut().insert(correlation_id.clone());

    let mut response = REQUEST_CORRELATION_ID
        .scope(correlation_id.clone(), next.run(request))
        .await;

    // Add to response header
    if let Ok(value) = HeaderValue::from_str(&correlation_id.0) {
        response.headers_mut().insert(X_REQUEST_ID, value);
    }

    response
}

/// Extract correlation ID from request header or generate a new one.
fn extract_or_generate_correlation_id(request: &Request) -> CorrelationId {
    request
        .headers()
        .get(X_REQUEST_ID)
        .and_then(|value| value.to_str().ok())
        .filter(|s| !s.is_empty() && s.len() <= MAX_CORRELATION_ID_LENGTH)
        .filter(|s| {
            s.bytes()
                .all(|byte| byte.is_ascii_graphic() && byte != b'"')
        })
        .map(|s| CorrelationId(s.to_string()))
        .unwrap_or_else(CorrelationId::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error::{AppError, ErrorKind};
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::middleware;
    use axum::routing::get;
    use tower::ServiceExt;

    #[test]
    fn test_correlation_id_generated_when_missing() {
        let request = Request::builder().body(Body::empty()).unwrap();

        let correlation_id = extract_or_generate_correlation_id(&request);
        assert!(!correlation_id.0.is_empty());
    }

    #[test]
    fn test_correlation_id_extracted_from_header() {
        let request = Request::builder()
            .header(X_REQUEST_ID, "my-custom-id")
            .body(Body::empty())
            .unwrap();

        let correlation_id = extract_or_generate_correlation_id(&request);
        assert_eq!(correlation_id.0, "my-custom-id");
    }

    #[test]
    fn test_correlation_id_generated_for_empty_header() {
        let request = Request::builder()
            .header(X_REQUEST_ID, "")
            .body(Body::empty())
            .unwrap();

        let correlation_id = extract_or_generate_correlation_id(&request);
        assert!(!correlation_id.0.is_empty());
        assert_ne!(correlation_id.0, "");
    }

    #[test]
    fn test_extract_or_generate_generates_unique_ids() {
        let request = Request::builder().body(Body::empty()).unwrap();

        let id1 = extract_or_generate_correlation_id(&request);
        let id2 = extract_or_generate_correlation_id(&request);
        assert_ne!(id1.0, id2.0);
    }

    #[test]
    fn test_extract_or_generate_preserves_existing_id() {
        let request = Request::builder()
            .header(X_REQUEST_ID, "my-id")
            .body(Body::empty())
            .unwrap();

        let id1 = extract_or_generate_correlation_id(&request);
        let id2 = extract_or_generate_correlation_id(&request);
        assert_eq!(id1.0, "my-id");
        assert_eq!(id2.0, "my-id");
    }

    #[test]
    fn test_extract_or_generate_rejects_unsafe_or_oversized_ids() {
        let unsafe_request = Request::builder()
            .header(X_REQUEST_ID, "bad\"id")
            .body(Body::empty())
            .unwrap();
        let oversized_request = Request::builder()
            .header(X_REQUEST_ID, "x".repeat(129))
            .body(Body::empty())
            .unwrap();

        assert_ne!(
            extract_or_generate_correlation_id(&unsafe_request).0,
            "bad\"id"
        );
        assert_eq!(
            extract_or_generate_correlation_id(&oversized_request)
                .0
                .len(),
            36
        );
    }

    async fn failing_handler() -> Result<&'static str, AppError> {
        Err(AppError::new(ErrorKind::NotFound, "not found"))
    }

    async fn internal_error_handler() -> Result<&'static str, AppError> {
        Err(AppError::new(
            ErrorKind::Internal,
            "database connection failed",
        ))
    }

    fn error_router() -> Router {
        Router::new()
            .route("/fail", get(failing_handler))
            .route("/internal", get(internal_error_handler))
            .layer(middleware::from_fn(correlation_id_middleware))
    }

    #[tokio::test]
    async fn test_error_response_propagates_request_correlation_id() {
        let router = error_router();
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/fail")
                    .header(X_REQUEST_ID, "corr-abc-123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response
                .headers()
                .get(X_REQUEST_ID)
                .unwrap()
                .to_str()
                .unwrap(),
            "corr-abc-123"
        );

        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["correlation_id"], "corr-abc-123");
    }

    #[tokio::test]
    async fn test_internal_error_response_preserves_redaction_and_correlation_id() {
        let router = error_router();
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/internal")
                    .header(X_REQUEST_ID, "corr-red-456")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["correlation_id"], "corr-red-456");
        assert_eq!(json["error"]["message"], "An internal error occurred");
    }

    #[tokio::test]
    async fn test_error_response_generates_correlation_id_without_middleware() {
        async fn bare_failing_handler() -> Result<&'static str, AppError> {
            Err(AppError::new(ErrorKind::NotFound, "not found"))
        }

        let router = Router::new().route("/fail", get(bare_failing_handler));
        let response = router
            .oneshot(Request::builder().uri("/fail").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let correlation_id = json["error"]["correlation_id"].as_str().unwrap();
        assert!(!correlation_id.is_empty());
    }

    #[tokio::test]
    async fn test_task_local_scope_roundtrip() {
        let scoped = REQUEST_CORRELATION_ID
            .scope(CorrelationId("scoped-id".to_string()), async {
                CorrelationId::current()
            })
            .await;
        assert_eq!(scoped.0, "scoped-id");

        let outside = CorrelationId::current();
        assert_ne!(outside.0, "scoped-id");
        assert!(!outside.0.is_empty());
    }
}
