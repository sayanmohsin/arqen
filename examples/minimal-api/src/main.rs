use arqen::http::{create_router, start_server};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    arqen::logging::init_logging("info", "pretty");
    
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
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
