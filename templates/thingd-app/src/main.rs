use arqen::http::create_router;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    arqen::logging::init_logging("info", "pretty");
    
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let router = create_router();
    
    println!("Starting {{project_name}} on {}", addr);
    
    arqen::http::start_server(addr, router).await?;
    Ok(())
}