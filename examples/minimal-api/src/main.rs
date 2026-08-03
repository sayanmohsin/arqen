use arqen::config::AppConfig;
use arqen::http::{create_router, start_server};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load configuration from environment variables
    let config = AppConfig::from_env().unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}", e);
        AppConfig::default()
    });

    // Initialize logging
    arqen::logging::init_logging(&config.logging.level, &format!("{:?}", config.logging.format).to_lowercase());

    let addr: SocketAddr = config.address()?;
    let router = create_router();

    println!("Starting minimal API on {}", addr);
    println!("Endpoints:");
    println!("  GET /health - Liveness check");
    println!("  GET /ready - Readiness check");
    println!("  GET /agent - Agent description");
    println!("  GET /agent/manifest - Agent manifest");
    println!("  GET /docs - API documentation");

    start_server(addr, router).await?;
    Ok(())
}
