//! Auth middleware for Arqen.
//!
//! Provides middleware that authenticates requests and inserts [`AuthContext`]
//! into request extensions, making it available to handlers via extractors.
//!
//! - [`auth_middleware`]: Enforces authentication — returns 401/403 on failure.
//! - [`optional_auth_middleware`]: Attempts authentication but continues even if it fails.
//! - [`Authenticated`]: Axum extractor that requires a valid `AuthContext`.
//! - `Option<Authenticated>`: Axum extractor for optional authentication.

use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};

use crate::auth::{AuthContext, AuthError, Authentication};
use crate::core::error::{CorrelationId, ErrorCode, ErrorResponse};

/// Axum extractor that requires authentication.
///
/// The request must go through [`auth_middleware`] (or [`optional_auth_middleware`])
/// for this to succeed. Returns 401 if no `AuthContext` is present.
///
/// # Example
///
/// ```rust,ignore
/// use arqen::http::middleware_auth::Authenticated;
///
/// async fn protected_handler(
///     auth: Authenticated,
/// ) -> String {
///     format!("Hello, {}", auth.0.subject)
/// }
/// ```
pub struct Authenticated(pub AuthContext);

impl Authenticated {
    pub fn into_inner(self) -> AuthContext {
        self.0
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Authenticated
where
    S: Send + Sync,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let _ = state;
        parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .map(Authenticated)
            .ok_or(AuthRejection::Missing)
    }
}

/// Rejection type for auth extraction errors.
#[derive(Debug)]
pub enum AuthRejection {
    /// No auth context found in request extensions.
    Missing,
    /// Authentication error from the adapter.
    Error(AuthError),
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AuthRejection::Missing => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::Authentication,
                "authentication required".to_string(),
            ),
            AuthRejection::Error(AuthError::Missing) => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::Authentication,
                "authentication required".to_string(),
            ),
            AuthRejection::Error(AuthError::Invalid) => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::Authentication,
                "invalid credentials".to_string(),
            ),
            AuthRejection::Error(AuthError::Expired) => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::Authentication,
                "credentials expired".to_string(),
            ),
            AuthRejection::Error(AuthError::Unauthorized(msg)) => {
                (StatusCode::FORBIDDEN, ErrorCode::Authorization, msg)
            }
        };

        let correlation_id = CorrelationId::current();
        let body = ErrorResponse::new(code, message, correlation_id.0);
        (status, axum::Json(body)).into_response()
    }
}

/// Auth middleware that enforces authentication.
///
/// Returns 401 if authentication fails. The authenticated [`AuthContext`] is
/// inserted into request extensions and can be extracted by handlers using
/// [`Authenticated`] or `Option<Authenticated>`.
///
/// # Example
///
/// ```rust,ignore
/// use arqen::auth::ApiKeyAuth;
/// use arqen::http::middleware_auth::auth_middleware;
/// use std::sync::Arc;
///
/// let auth = Arc::new(ApiKeyAuth::new().with_key("secret", "user-1"));
/// let router = Router::new()
///     .route("/protected", get(my_handler))
///     .layer(middleware::from_fn_with_state(auth, auth_middleware));
/// ```
pub async fn auth_middleware(
    State(auth): State<Arc<dyn Authentication>>,
    mut req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Response {
    match auth.authenticate(req.headers()).await {
        Ok(ctx) => {
            req.extensions_mut().insert(ctx);
            next.run(req).await
        }
        Err(e) => {
            let (status, code, message) = match e {
                AuthError::Missing => (
                    StatusCode::UNAUTHORIZED,
                    ErrorCode::Authentication,
                    "authentication required".to_string(),
                ),
                AuthError::Invalid => (
                    StatusCode::UNAUTHORIZED,
                    ErrorCode::Authentication,
                    "invalid credentials".to_string(),
                ),
                AuthError::Expired => (
                    StatusCode::UNAUTHORIZED,
                    ErrorCode::Authentication,
                    "credentials expired".to_string(),
                ),
                AuthError::Unauthorized(msg) => {
                    (StatusCode::FORBIDDEN, ErrorCode::Authorization, msg)
                }
            };

            let correlation_id = CorrelationId::current();
            let body = ErrorResponse::new(code, message, correlation_id.0);
            (status, axum::Json(body)).into_response()
        }
    }
}

/// Auth middleware that optionally authenticates requests.
///
/// Always continues to the handler. If authentication succeeds, the
/// [`AuthContext`] is inserted into extensions. Handlers can use
/// `Option<Authenticated>` to check for identity without blocking.
///
/// # Example
///
/// ```rust,ignore
/// use arqen::http::middleware_auth::{optional_auth_middleware, Authenticated};
///
/// async fn public_handler(
///     auth: Option<Authenticated>,
/// ) -> String {
///     match auth {
///         Some(auth) => format!("Hello, {}", auth.0.subject),
///         None => "Hello, anonymous".to_string(),
///     }
/// }
/// ```
pub async fn optional_auth_middleware(
    State(auth): State<Arc<dyn Authentication>>,
    mut req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Response {
    if let Ok(ctx) = auth.authenticate(req.headers()).await {
        req.extensions_mut().insert(ctx);
    }
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use tower::ServiceExt;

    use crate::auth::ApiKeyAuth;

    fn test_auth() -> Arc<dyn Authentication> {
        Arc::new(ApiKeyAuth::new().with_key("test-key", "user-123"))
    }

    async fn protected_handler(auth: Authenticated) -> String {
        format!("hello {}", auth.0.subject)
    }

    async fn optional_handler(auth: Option<Authenticated>) -> String {
        match auth {
            Some(auth) => format!("hello {}", auth.0.subject),
            None => "hello anonymous".to_string(),
        }
    }

    #[tokio::test]
    async fn test_auth_middleware_success() {
        let auth = test_auth();
        let router = Router::new()
            .route("/protected", get(protected_handler))
            .layer(axum::middleware::from_fn_with_state(
                auth.clone(),
                auth_middleware,
            ));

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("x-api-key", "test-key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_middleware_missing_credentials() {
        let auth = test_auth();
        let router = Router::new()
            .route("/protected", get(protected_handler))
            .layer(axum::middleware::from_fn_with_state(
                auth.clone(),
                auth_middleware,
            ));

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_invalid_credentials() {
        let auth = test_auth();
        let router = Router::new()
            .route("/protected", get(protected_handler))
            .layer(axum::middleware::from_fn_with_state(
                auth.clone(),
                auth_middleware,
            ));

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("x-api-key", "wrong-key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_optional_auth_middleware_with_credentials() {
        let auth = test_auth();
        let router = Router::new().route("/public", get(optional_handler)).layer(
            axum::middleware::from_fn_with_state(auth.clone(), optional_auth_middleware),
        );

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/public")
                    .header("x-api-key", "test-key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body, "hello user-123");
    }

    #[tokio::test]
    async fn test_optional_auth_middleware_without_credentials() {
        let auth = test_auth();
        let router = Router::new().route("/public", get(optional_handler)).layer(
            axum::middleware::from_fn_with_state(auth.clone(), optional_auth_middleware),
        );

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/public")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body, "hello anonymous");
    }

    #[tokio::test]
    async fn test_authenticated_extractor_without_context() {
        let router = Router::new().route("/protected", get(protected_handler));

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_rejection_display() {
        let rejection = AuthRejection::Missing;
        let response = rejection.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
