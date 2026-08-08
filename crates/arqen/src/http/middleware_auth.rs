//! Auth middleware for Arqen.
//!
//! Provides middleware that authenticates requests and inserts [`AuthContext`]
//! into request extensions, making it available to handlers via extractors.
//!
//! - [`auth_middleware`]: Enforces authentication — returns 401/403 on failure.
//! - [`optional_auth_middleware`]: Attempts authentication but continues even if it fails.
//! - [`Authenticated`]: Axum extractor that requires a valid `AuthContext`.
//! - `Option<Authenticated>`: Axum extractor for optional authentication.

use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};

use crate::auth::{AuthContext, AuthError, Authentication, Policy};
use crate::core::error::{CorrelationId, ErrorCode, ErrorResponse};

/// Default resource name passed to [`Policy::check`] by the auth guards.
const DEFAULT_AUTH_RESOURCE: &str = "request";

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
            let context = crate::context::from_extensions(req.extensions());
            req.extensions_mut().insert(context);
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
        let context = crate::context::from_extensions(req.extensions());
        req.extensions_mut().insert(context);
    }
    next.run(req).await
}

/// Extract the authenticated [`AuthContext`] from the current request.
///
/// Returns 401 when no auth middleware has inserted a context. Use this in
/// handlers behind [`auth_middleware`] or [`require_auth_middleware`].
#[async_trait]
impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .ok_or(AuthRejection::Missing)
    }
}

/// Extractor that authenticates a request and enforces a policy inline.
///
/// `A` and `P` are resolved from the router state via [`FromRef`], so the
/// application state must expose an authentication adapter and a policy (for
/// example with `#[derive(FromRef)]`). Credential failures map to 401 and
/// policy failures to 403. On success the [`AuthContext`] is inserted into the
/// request extensions, so handlers can also extract it directly.
///
/// # Example
///
/// ```rust,ignore
/// use arqen::http::middleware_auth::RequireAuth;
///
/// async fn handler(auth: RequireAuth<Arc<dyn Authentication>, Arc<dyn Policy>>) -> String {
///     format!("hello {}", auth.context.subject)
/// }
/// ```
pub struct RequireAuth<A, P> {
    /// The authenticated context.
    pub context: AuthContext,
    _auth: PhantomData<A>,
    _policy: PhantomData<P>,
}

#[async_trait]
impl<S, A, P> FromRequestParts<S> for RequireAuth<A, P>
where
    S: Send + Sync,
    A: FromRef<S> + Authentication,
    P: FromRef<S> + Policy,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = A::from_ref(state);
        let context = auth
            .authenticate(&parts.headers)
            .await
            .map_err(AuthRejection::Error)?;
        let policy = P::from_ref(state);
        policy
            .check(&context, DEFAULT_AUTH_RESOURCE)
            .map_err(AuthRejection::Error)?;
        parts.extensions.insert(context.clone());
        Ok(Self {
            context,
            _auth: PhantomData,
            _policy: PhantomData,
        })
    }
}

/// A reusable authentication + authorization guard for a route subtree.
///
/// Holds an [`Authentication`] adapter and a [`Policy`]. The default policy is
/// [`AllowAll`](crate::auth::AllowAll), which combined with mandatory
/// authentication means "any authenticated user".
#[derive(Clone)]
pub struct AuthGuard {
    /// Authentication adapter.
    pub auth: Arc<dyn Authentication>,
    /// Authorization policy applied after authentication.
    pub policy: Arc<dyn Policy>,
}

impl AuthGuard {
    /// Create a guard that requires any authenticated user.
    pub fn new(auth: Arc<dyn Authentication>) -> Self {
        Self {
            auth,
            policy: Arc::new(crate::auth::AllowAll),
        }
    }

    /// Set a custom authorization policy.
    pub fn with_policy(mut self, policy: Arc<dyn Policy>) -> Self {
        self.policy = policy;
        self
    }
}

/// Middleware that authenticates and authorizes requests.
///
/// Pair with [`AuthGuard`] via `axum::middleware::from_fn_with_state` to
/// protect a whole route subtree:
///
/// ```rust,ignore
/// use arqen::http::middleware_auth::{AuthGuard, require_auth_middleware};
///
/// let guard = AuthGuard::new(auth);
/// let router = Router::new()
///     .route("/protected", get(handler))
///     .layer(middleware::from_fn_with_state(guard, require_auth_middleware));
/// ```
///
/// Credential failures return 401; policy failures return 403. On success the
/// [`AuthContext`] is inserted into request extensions.
pub async fn require_auth_middleware(
    State(guard): State<AuthGuard>,
    mut req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Response {
    match guard.auth.authenticate(req.headers()).await {
        Ok(ctx) => {
            if let Err(e) = guard.policy.check(&ctx, DEFAULT_AUTH_RESOURCE) {
                return AuthRejection::Error(e).into_response();
            }
            req.extensions_mut().insert(ctx);
            next.run(req).await
        }
        Err(e) => AuthRejection::Error(e).into_response(),
    }
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

    #[derive(Clone)]
    struct GuardState {
        auth: Arc<dyn Authentication>,
        policy: Arc<dyn Policy>,
    }

    impl FromRef<GuardState> for Arc<dyn Authentication> {
        fn from_ref(state: &GuardState) -> Self {
            state.auth.clone()
        }
    }

    impl FromRef<GuardState> for Arc<dyn Policy> {
        fn from_ref(state: &GuardState) -> Self {
            state.policy.clone()
        }
    }

    async fn guard_handler(auth: AuthContext) -> String {
        format!("hello {}", auth.subject)
    }

    async fn require_auth_handler(
        auth: RequireAuth<Arc<dyn Authentication>, Arc<dyn Policy>>,
    ) -> String {
        format!("hello {}", auth.context.subject)
    }

    #[tokio::test]
    async fn test_require_auth_middleware_success() {
        let guard = AuthGuard::new(test_auth());
        let router = Router::new().route("/protected", get(guard_handler)).layer(
            axum::middleware::from_fn_with_state(guard, require_auth_middleware),
        );

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
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body, "hello user-123");
    }

    #[tokio::test]
    async fn test_require_auth_middleware_missing_credentials() {
        let guard = AuthGuard::new(test_auth());
        let router = Router::new().route("/protected", get(guard_handler)).layer(
            axum::middleware::from_fn_with_state(guard, require_auth_middleware),
        );

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
    async fn test_require_auth_middleware_policy_failure_returns_403() {
        let guard = AuthGuard::new(test_auth()).with_policy(Arc::new(crate::auth::DenyAll));
        let router = Router::new().route("/protected", get(guard_handler)).layer(
            axum::middleware::from_fn_with_state(guard, require_auth_middleware),
        );

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

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_require_auth_extractor_success() {
        let state = GuardState {
            auth: test_auth(),
            policy: Arc::new(crate::auth::AllowAll),
        };
        let router = Router::new()
            .route("/protected", get(require_auth_handler))
            .with_state(state);

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
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body, "hello user-123");
    }

    #[tokio::test]
    async fn test_require_auth_extractor_missing_credentials_returns_401() {
        let state = GuardState {
            auth: test_auth(),
            policy: Arc::new(crate::auth::AllowAll),
        };
        let router = Router::new()
            .route("/protected", get(require_auth_handler))
            .with_state(state);

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
    async fn test_require_auth_extractor_policy_failure_returns_403() {
        let state = GuardState {
            auth: test_auth(),
            policy: Arc::new(crate::auth::DenyAll),
        };
        let router = Router::new()
            .route("/protected", get(require_auth_handler))
            .with_state(state);

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

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_auth_context_extractor_without_middleware_returns_401() {
        let router = Router::new().route("/protected", get(guard_handler));

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
}
