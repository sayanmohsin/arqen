use std::net::SocketAddr;

use super::exit;
use super::output::Output;

pub fn serve_dev(
    file: Option<&str>,
    host: &str,
    port: u16,
    log: &str,
    storage: &str,
    output: &Output,
) -> i32 {
    let cli = crate::config::CliOverrides {
        host: Some(host.to_string()),
        port: Some(port),
        log_level: Some(log.to_string()),
        storage_mode: Some(storage.to_string()),
    };

    let path = file.map(std::path::PathBuf::from);
    let config = match crate::config::AppConfig::load_with_file(cli, path.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            output.print_error(&e.to_string());
            return exit::CONFIG;
        }
    };

    let addr: SocketAddr = match config.address() {
        Ok(a) => a,
        Err(e) => {
            output.print_error(&e.to_string());
            return exit::CONFIG;
        }
    };

    crate::logging::init_logging(&config.logging.level, "pretty");

    let state = match crate::AppState::builder()
        .with_storage_mode(storage)
        .with_tool_registry(crate::ToolRegistry::new(
            "arqen-app",
            env!("CARGO_PKG_VERSION"),
            "An Arqen application",
            storage,
        ))
        .build()
    {
        Ok(s) => s,
        Err(e) => {
            output.print_error(&e.to_string());
            return exit::RUNTIME;
        }
    };
    let router = crate::http::create_router_with_state(state);

    output.print(&format!("Arqen v{}", env!("CARGO_PKG_VERSION")));
    output.print(&format!("API:    http://{}", addr));
    output.print(&format!("Health: http://{}/health", addr));
    output.print(&format!("Docs:   http://{}/docs", addr));
    output.print(&format!("Agent:  http://{}/agent", addr));
    output.print(&format!("Storage: {}", storage));

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            output.print_error(&e.to_string());
            return exit::RUNTIME;
        }
    };
    if let Err(e) = rt.block_on(crate::http::start_server(addr, router)) {
        output.print_error(&e.to_string());
        return exit::RUNTIME;
    }
    exit::SUCCESS
}

pub fn serve_start(
    file: Option<&str>,
    host: &str,
    port: u16,
    log: &str,
    storage: &str,
    output: &Output,
) -> i32 {
    let cli = crate::config::CliOverrides {
        host: Some(host.to_string()),
        port: Some(port),
        log_level: Some(log.to_string()),
        storage_mode: Some(storage.to_string()),
    };

    let path = file.map(std::path::PathBuf::from);
    let config = match crate::config::AppConfig::load_with_file(cli, path.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            output.print_error(&e.to_string());
            return exit::CONFIG;
        }
    };

    let addr: SocketAddr = match config.address() {
        Ok(a) => a,
        Err(e) => {
            output.print_error(&e.to_string());
            return exit::CONFIG;
        }
    };

    crate::logging::init_logging(&config.logging.level, "json");

    let state = match crate::AppState::builder()
        .with_storage_mode(storage)
        .with_tool_registry(crate::ToolRegistry::new(
            "arqen-app",
            env!("CARGO_PKG_VERSION"),
            "An Arqen application",
            storage,
        ))
        .build()
    {
        Ok(s) => s,
        Err(e) => {
            output.print_error(&e.to_string());
            return exit::RUNTIME;
        }
    };
    let router = crate::http::create_router_with_state(state);

    output.print(&format!("Arqen listening on {}", addr));

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            output.print_error(&e.to_string());
            return exit::RUNTIME;
        }
    };
    if let Err(e) = rt.block_on(crate::http::start_server(addr, router)) {
        output.print_error(&e.to_string());
        return exit::RUNTIME;
    }
    exit::SUCCESS
}
