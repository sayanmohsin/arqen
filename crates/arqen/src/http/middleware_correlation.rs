use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};

use crate::core::error::CorrelationId;

/// Header name for correlation ID.
pub const X_REQUEST_ID: &str = "X-Request-Id";

/// Middleware that generates or extracts correlation ID from request header.
///
/// If the request contains an `X-Request-Id` header, it's used as the correlation ID.
/// Otherwise, a new UUID is generated.
///
/// The correlation ID is added to:
/// - Request extensions (accessible via `Extension<CorrelationId>`)
/// - Response header (`X-Request-Id`)
pub async fn correlation_id_middleware(mut request: Request, next: Next) -> Response {
    let correlation_id = extract_or_generate_correlation_id(&request);

    // Insert into request extensions
    request.extensions_mut().insert(correlation_id.clone());

    let mut response = next.run(request).await;

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
        .filter(|s| !s.is_empty())
        .map(|s| CorrelationId(s.to_string()))
        .unwrap_or_else(CorrelationId::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;

    #[test]
    fn test_correlation_id_generated_when_missing() {
        let request = Request::builder()
            .body(Body::empty())
            .unwrap();

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
        let request = Request::builder()
            .body(Body::empty())
            .unwrap();

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
}
