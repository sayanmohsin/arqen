//! Public Arqen response identity.

use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};

/// Standard server identity header exposed by Arqen-managed responses.
pub const SERVER_HEADER: &str = "server";
/// Framework identity header exposed by Arqen-managed responses.
pub const POWERED_BY_HEADER: &str = "x-powered-by";
/// Public framework identity value.
pub const ARQEN_IDENTITY: &str = "Arqen";

#[derive(Clone, Copy)]
pub(crate) struct IdentityApplied;

/// Mark a response as being served by Arqen.
pub async fn identity_middleware(mut request: Request, next: Next) -> Response {
    let already_applied = request.extensions().get::<IdentityApplied>().is_some();
    request.extensions_mut().insert(IdentityApplied);
    let mut response = next.run(request).await;
    if already_applied {
        return response;
    }
    response
        .headers_mut()
        .insert(SERVER_HEADER, HeaderValue::from_static(ARQEN_IDENTITY));
    response
        .headers_mut()
        .insert(POWERED_BY_HEADER, HeaderValue::from_static(ARQEN_IDENTITY));
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router, body::Body, http::Request, http::StatusCode, response::IntoResponse, routing::get,
    };
    use tower::ServiceExt;

    async fn handler() -> impl IntoResponse {
        StatusCode::OK
    }

    #[tokio::test]
    async fn exposes_arqen_identity_and_replaces_other_frameworks() {
        let app = Router::new()
            .route("/", get(handler))
            .layer(axum::middleware::from_fn(identity_middleware));
        let request = Request::builder().uri("/").body(Body::empty()).unwrap();
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.headers()[SERVER_HEADER], ARQEN_IDENTITY);
        assert_eq!(response.headers()[POWERED_BY_HEADER], ARQEN_IDENTITY);
        assert!(
            !response
                .headers()
                .values()
                .any(|value| { value.to_str().is_ok_and(|value| value.contains("Axum")) })
        );
    }
}
