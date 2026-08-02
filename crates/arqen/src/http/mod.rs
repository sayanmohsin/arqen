pub mod middleware_log;
pub mod routes;

pub use middleware_log::logging_middleware;
pub use routes::{agent, agent_manifest, docs, health, ready};

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
    axum::serve(listener, router).await?;
    Ok(())
}
