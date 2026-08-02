use axum::{Router, extract::Extension, middleware, routing::get};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::limit::RequestBodyLimitLayer;

#[derive(Clone, Debug)]
pub struct RuntimeInfo {
    pub storage_mode: String,
    pub thingd_ready: bool,
}

impl Default for RuntimeInfo {
    fn default() -> Self {
        Self {
            storage_mode: "memory".to_string(),
            thingd_ready: true,
        }
    }
}

pub mod middleware_log;
pub mod routes;

pub fn create_router() -> Router {
    create_router_with_runtime(RuntimeInfo::default())
}

pub fn create_router_with_runtime(runtime: RuntimeInfo) -> Router {
    Router::new()
        .route("/health", get(routes::health))
        .route("/ready", get(routes::ready))
        .route("/agent", get(routes::agent))
        .route("/agent/manifest", get(routes::agent_manifest))
        .route("/docs", get(routes::docs))
        .layer(Extension(runtime))
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
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
