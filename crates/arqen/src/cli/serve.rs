use std::net::SocketAddr;

use super::exit;
use super::output::Output;

pub fn serve_dev(
    file: Option<&str>,
    host: Option<&str>,
    port: Option<u16>,
    log: Option<&str>,
    storage: Option<&str>,
    log_format: Option<&crate::config::LogFormat>,
    output: &Output,
) -> i32 {
    let cli = crate::config::CliOverrides {
        host: host.map(str::to_owned),
        port,
        log_level: log.map(str::to_owned),
        log_format: log_format.copied(),
        storage_mode: storage.map(str::to_owned),
    };

    let path = file.map(std::path::PathBuf::from);
    let config = match crate::config::AppConfig::load_with_file(cli, path.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            output.print_error(&e.to_string());
            return exit::CONFIG;
        }
    };

    let storage = config.storage.mode.as_str();
    let addr: SocketAddr = match config.address() {
        Ok(a) => a,
        Err(e) => {
            output.print_error(&e.to_string());
            return exit::CONFIG;
        }
    };

    crate::logging::init_logging_with_config(&config.logging);

    let state = match crate::AppState::builder()
        .with_config(config.clone())
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

    output.print_banner(&addr, storage, "development");

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

#[allow(clippy::too_many_arguments)]
pub fn serve_start(
    file: Option<&str>,
    host: Option<&str>,
    port: Option<u16>,
    log: Option<&str>,
    storage: Option<&str>,
    log_format: Option<&crate::config::LogFormat>,
    skip_schema_validation: bool,
    output: &Output,
) -> i32 {
    let cli = crate::config::CliOverrides {
        host: host.map(str::to_owned),
        port,
        log_level: log.map(str::to_owned),
        log_format: Some(
            log_format
                .copied()
                .unwrap_or(crate::config::LogFormat::Json),
        ),
        storage_mode: storage.map(str::to_owned),
    };

    let path = file.map(std::path::PathBuf::from);
    let config = match crate::config::AppConfig::load_with_file(cli, path.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            output.print_error(&e.to_string());
            return exit::CONFIG;
        }
    };

    if let Err(error) = config.validate_production_with_schema_validation(!skip_schema_validation) {
        output.print_error(&format!("production configuration is unsafe: {error}"));
        return exit::CONFIG;
    }

    let storage = config.storage.mode.as_str();
    let addr: SocketAddr = match config.address() {
        Ok(a) => a,
        Err(e) => {
            output.print_error(&e.to_string());
            return exit::CONFIG;
        }
    };

    crate::logging::init_logging_with_config(&config.logging);

    let state = match crate::AppState::builder()
        .with_config(config.clone())
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

    output.print_banner(&addr, storage, "production");

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
