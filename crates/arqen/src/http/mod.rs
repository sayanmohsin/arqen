pub mod middleware_auth;
pub mod middleware_correlation;
pub mod middleware_identity;
pub mod middleware_log;
pub mod module;
pub mod routes;

pub use middleware_auth::{
    AuthGuard, Authenticated, RequireAuth, auth_middleware, optional_auth_middleware,
    require_auth_middleware,
};
pub use middleware_correlation::{X_REQUEST_ID, correlation_id_middleware};
pub use middleware_identity::{
    ARQEN_IDENTITY, POWERED_BY_HEADER, SERVER_HEADER, identity_middleware,
};
pub use middleware_log::logging_middleware;
pub use module::{HttpModule, merge_module_routes};
pub use routes::{agent, agent_manifest, docs, health, ready};

use axum::extract::FromRef;
use axum::{
    http::StatusCode,
    routing::{get, post},
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;

use crate::state::AppState;

/// Arqen's application-facing HTTP facade. These re-exports let applications
/// build routes through `arqen::http` without importing the underlying
/// transport crate directly.
pub use axum::{Router, body, extract, http, middleware, response, routing};

/// Create the default Arqen router.
pub fn create_router() -> Router {
    let state = AppState::builder()
        .build()
        .expect("failed to build default state");
    create_router_with_state(state)
}

/// Build Arqen's built-in routes without resolving state.
///
/// Returns a [`Router<S>`] containing the built-in Arqen routes (`/health`,
/// `/ready`, `/agent`, `/agent/manifest`, `/docs`) and standard middleware
/// (CORS, timeout, body limit, correlation ID, request logging), leaving the
/// router's state unresolved.
///
/// The generic state `S` lets applications with arbitrary typed state mount
/// the built-in routes. The built-in handlers extract an [`AppState`] sub-state
/// from `S`, so `S` must expose one via [`FromRef`]. When `S` is [`AppState`]
/// itself the blanket `FromRef` impl applies.
///
/// # Example
///
/// ```rust,ignore
/// use arqen::http::builtin_routes;
/// use arqen::AppState;
/// use axum::{Router, routing::get, extract::FromRef};
///
/// #[derive(Clone)]
/// struct MyState {
///     arqen: AppState,
/// }
///
/// impl FromRef<MyState> for AppState {
///     fn from_ref(state: &MyState) -> Self {
///         state.arqen.clone()
///     }
/// }
///
/// async fn my_handler() -> &'static str { "hello" }
///
/// let app_state = AppState::builder().build().unwrap();
///
/// let router: Router<MyState> = builtin_routes(&app_state)
///     .route("/api/hello", get(my_handler));
/// let router = router.with_state(MyState { arqen: app_state });
/// // Built-in: GET /health, GET /ready, GET /agent, GET /agent/manifest, GET /docs
/// // App:      GET /api/hello
/// ```
pub fn builtin_routes<S>(state: &AppState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let timeout = TimeoutLayer::with_status_code(
        StatusCode::GATEWAY_TIMEOUT,
        state.config.server.request_timeout,
    );
    let body_limit = RequestBodyLimitLayer::new(state.config.server.max_body_size);

    Router::new()
        .route("/health", get(routes::health))
        .route("/ready", get(routes::ready))
        .route("/agent", get(routes::agent))
        .route("/agent/manifest", get(routes::agent_manifest))
        .route("/agent/tools/:name", post(routes::tool_invoke))
        .route("/docs", get(routes::docs))
        .layer(body_limit)
        .layer(timeout)
        .layer(cors)
        .layer(middleware::from_fn(
            middleware_identity::identity_middleware,
        ))
        .layer(middleware::from_fn(
            middleware_correlation::correlation_id_middleware,
        ))
        .layer(middleware::from_fn(middleware_log::logging_middleware))
}

/// Create a router with the given app state.
///
/// Returns a [`Router`] with the built-in Arqen routes (`/health`, `/ready`,
/// `/agent`, `/agent/manifest`, `/docs`) and standard middleware (CORS,
/// timeout, body limit, correlation ID, request logging).
///
/// To add application-specific routes, use [`create_router_with_state_and_routes`]
/// or [`nest_routes`], or merge manually:
///
/// ```rust,ignore
/// use axum::Router;
///
/// let arqen_router = create_router_with_state(state);
/// let app_router = Router::new()
///     .nest("/api/v1", my_routes)
///     .merge(arqen_router);
/// ```
///
/// For applications that use arbitrary typed state, see [`builtin_routes`].
pub fn create_router_with_state(state: AppState) -> Router {
    builtin_routes::<AppState>(&state).with_state(state)
}

/// Create a router with the given app state, merged with application routes.
///
/// This is the primary entry point for building a complete application router.
/// It combines the built-in Arqen routes with user-provided routes in a single
/// call. Application routes are merged at the root level, so you can use
/// [`axum::Router::nest`] inside `app_routes` to namespace them.
///
/// For applications that use arbitrary typed state, use [`builtin_routes`] and
/// merge your routes into the resulting [`Router`] yourself.
///
/// # Example
///
/// ```rust,ignore
/// use axum::{Router, routing::get};
/// use arqen::http::{create_router_with_state_and_routes, start_server};
/// use arqen::AppState;
///
/// async fn my_handler() -> &'static str { "hello" }
///
/// let state = AppState::builder().build().unwrap();
/// let app_routes = Router::new()
///     .nest("/api/v1", Router::new().route("/users", get(my_handler)));
///
/// let router = create_router_with_state_and_routes(state, app_routes);
/// ```
pub fn create_router_with_state_and_routes(state: AppState, app_routes: Router) -> Router {
    create_router_with_state(state)
        .merge(app_routes)
        .layer(middleware::from_fn(
            middleware_identity::identity_middleware,
        ))
}

/// Create a router with the given app state, nesting application routes under a prefix.
///
/// Application routes are nested under the given `prefix` (e.g. `/api/v1`),
/// while Arqen's built-in routes remain at the root.
///
/// # Example
///
/// ```rust,ignore
/// use axum::{Router, routing::get};
/// use arqen::http::{nest_routes, start_server};
/// use arqen::AppState;
///
/// async fn my_handler() -> &'static str { "hello" }
///
/// let state = AppState::builder().build().unwrap();
/// let app_routes = Router::new().route("/users", get(my_handler));
///
/// let router = nest_routes(state, "/api/v1", app_routes);
/// // Built-in: GET /health, GET /ready, ...
/// // App:      GET /api/v1/users
/// ```
pub fn nest_routes(state: AppState, prefix: &str, app_routes: Router) -> Router {
    create_router_with_state(state)
        .nest(prefix, app_routes)
        .layer(middleware::from_fn(
            middleware_identity::identity_middleware,
        ))
}

pub async fn start_server(
    addr: SocketAddr,
    router: Router,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);
    axum::serve(
        listener,
        router.layer(middleware::from_fn(
            middleware_identity::identity_middleware,
        )),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "failed to install Ctrl-C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => tracing::error!(%error, "failed to install SIGTERM handler"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    tracing::info!("shutdown signal received; draining requests");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::extract::State;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "ok"
    }

    #[tokio::test]
    async fn test_create_router() {
        let router = create_router();
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let router = create_router();
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_ready_endpoint() {
        let router = create_router();
        let request = Request::builder()
            .uri("/ready")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_agent_endpoint() {
        let router = create_router();
        let request = Request::builder()
            .uri("/agent")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_docs_endpoint() {
        let router = create_router();
        let request = Request::builder().uri("/docs").body(Body::empty()).unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_cors_headers() {
        let router = create_router();
        let request = Request::builder()
            .method("OPTIONS")
            .uri("/health")
            .header("origin", "http://example.com")
            .header("access-control-request-method", "GET")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert!(
            response
                .headers()
                .contains_key("access-control-allow-origin"),
            "CORS headers should be present"
        );
    }

    #[tokio::test]
    async fn test_create_router_with_state_and_routes() {
        let state = AppState::builder().build().unwrap();
        let app_routes = Router::new().route("/api/hello", get(test_handler));
        let router = create_router_with_state_and_routes(state, app_routes);

        let built_in = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(built_in.status(), StatusCode::OK);

        let app = router
            .oneshot(
                Request::builder()
                    .uri("/api/hello")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(app.status(), StatusCode::OK);
        assert_eq!(app.headers()[SERVER_HEADER], ARQEN_IDENTITY);
        assert_eq!(app.headers()[POWERED_BY_HEADER], ARQEN_IDENTITY);
    }

    #[tokio::test]
    async fn test_nest_routes() {
        let state = AppState::builder().build().unwrap();
        let app_routes = Router::new().route("/users", get(test_handler));
        let router = nest_routes(state, "/api/v1", app_routes);

        let built_in = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(built_in.status(), StatusCode::OK);

        let app = router
            .oneshot(
                Request::builder()
                    .uri("/api/v1/users")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(app.status(), StatusCode::OK);
        assert_eq!(app.headers()[SERVER_HEADER], ARQEN_IDENTITY);
        assert_eq!(app.headers()[POWERED_BY_HEADER], ARQEN_IDENTITY);
    }

    #[tokio::test]
    async fn test_nest_routes_returns_404_for_missing() {
        let state = AppState::builder().build().unwrap();
        let app_routes = Router::new().route("/users", get(test_handler));
        let router = nest_routes(state, "/api/v1", app_routes);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/v1/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[derive(Clone)]
    struct CustomState {
        arqen: AppState,
        custom: String,
    }

    impl FromRef<CustomState> for AppState {
        fn from_ref(state: &CustomState) -> Self {
            state.arqen.clone()
        }
    }

    async fn custom_state_handler(State(state): State<CustomState>) -> String {
        state.custom
    }

    fn custom_state_router() -> Router<CustomState> {
        let app_state = AppState::builder().build().unwrap();
        builtin_routes::<CustomState>(&app_state).route("/custom", get(custom_state_handler))
    }

    #[tokio::test]
    async fn test_builtin_routes_with_custom_state() {
        let app_state = AppState::builder().build().unwrap();
        let router = custom_state_router().with_state(CustomState {
            arqen: app_state,
            custom: "hello".to_string(),
        });

        for path in ["/health", "/ready", "/agent", "/agent/manifest", "/docs"] {
            let response = router
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "path {path} should respond"
            );
        }

        let app = router
            .oneshot(
                Request::builder()
                    .uri("/custom")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(app.status(), StatusCode::OK);
        let body = axum::body::to_bytes(app.into_body(), 1024).await.unwrap();
        assert_eq!(&body[..], b"hello");
    }
}
