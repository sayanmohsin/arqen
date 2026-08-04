pub mod middleware_correlation;
pub mod middleware_log;
pub mod routes;

pub use middleware_correlation::{correlation_id_middleware, X_REQUEST_ID};
pub use middleware_log::logging_middleware;
pub use routes::{agent, agent_manifest, docs, health, ready};

use axum::{Router, extract::State, http::StatusCode, middleware, routing::get};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;

use crate::agent::ToolRegistry;
use crate::config::ServerConfig;
use crate::state::AppState;

/// Create the default Arqen router.
pub fn create_router() -> Router {
    let state = AppState::builder().build().expect("failed to build default state");
    create_router_with_state(state)
}

/// Create a router with the given app state.
pub fn create_router_with_state(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let timeout = TimeoutLayer::with_status_code(StatusCode::GATEWAY_TIMEOUT, state.config.server.request_timeout);
    let body_limit = RequestBodyLimitLayer::new(state.config.server.max_body_size);

    Router::new()
        .route("/health", get(routes::health))
        .route("/ready", get(routes::ready))
        .route("/agent", get(routes::agent))
        .route("/agent/manifest", get(routes::agent_manifest))
        .route("/docs", get(routes::docs))
        .with_state(state)
        .layer(body_limit)
        .layer(timeout)
        .layer(cors)
        .layer(middleware::from_fn(middleware_correlation::correlation_id_middleware))
        .layer(middleware::from_fn(middleware_log::logging_middleware))
}

pub async fn start_server(
    addr: SocketAddr,
    router: Router,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);
    axum::serve(listener, router)
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
