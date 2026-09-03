use std::net::SocketAddr;

use super::exit;
use super::output::Output;

#[allow(clippy::too_many_arguments)]
pub fn serve_dev(
    file: Option<&str>,
    host: Option<&str>,
    port: Option<u16>,
    log: Option<&str>,
    storage: Option<&str>,
    log_format: Option<&crate::config::LogFormat>,
    watch: bool,
    output: &Output,
) -> i32 {
    if is_arqen_application() {
        return run_application(
            "development",
            file,
            host,
            port,
            log,
            storage,
            log_format,
            watch,
            output,
        );
    }
    if watch {
        return run_watch(output);
    }
    let cli = crate::config::CliOverrides {
        host: host.map(str::to_owned),
        port,
        log_level: log.map(str::to_owned),
        log_format: log_format.copied().or_else(|| {
            std::env::var_os("ARQEN_LOG_FORMAT")
                .is_none()
                .then_some(crate::config::LogFormat::Compact)
        }),
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

fn is_arqen_application() -> bool {
    let Ok(manifest) = std::fs::read_to_string("Cargo.toml") else {
        return false;
    };
    manifest.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with("arqen =") || line.starts_with("arqen = {")
    })
}

#[allow(clippy::too_many_arguments)]
fn run_application(
    mode: &str,
    file: Option<&str>,
    host: Option<&str>,
    port: Option<u16>,
    log: Option<&str>,
    storage: Option<&str>,
    log_format: Option<&crate::config::LogFormat>,
    watch: bool,
    output: &Output,
) -> i32 {
    let mut command = std::process::Command::new("cargo");
    if watch {
        command.args(["watch", "--why", "-x", "run"]);
    } else {
        command.args(["run", "--quiet"]);
    }
    if let Some(file) = file {
        command.env("ARQEN_CONFIG_FILE", file);
    }
    if let Some(host) = host {
        command.env("ARQEN_HOST", host);
    }
    if let Some(port) = port {
        command.env("ARQEN_PORT", port.to_string());
    }
    if let Some(log) = log {
        command.env("ARQEN_LOG_LEVEL", log);
    }
    if let Some(storage) = storage {
        command.env("ARQEN_STORAGE_MODE", storage);
    }
    if let Some(format) = log_format {
        let format = match format {
            crate::config::LogFormat::Pretty => "pretty",
            crate::config::LogFormat::Compact => "compact",
            crate::config::LogFormat::Json => "json",
        };
        command.env("ARQEN_LOG_FORMAT", format);
    } else if std::env::var_os("ARQEN_LOG_FORMAT").is_none() {
        command.env(
            "ARQEN_LOG_FORMAT",
            if mode == "production" {
                "json"
            } else {
                "compact"
            },
        );
    }
    if mode == "production" {
        command.env("ARQEN_LAUNCH_MODE", "production");
    }
    output.print(&format!(
        "Starting application in {mode} mode{}",
        if watch { " with automatic reload" } else { "" }
    ));

    match command.status() {
        Ok(status) if status.success() => exit::SUCCESS,
        Ok(status) => {
            output.print_error(&format!(
                "application process exited with {}",
                status
                    .code()
                    .map_or_else(|| "a signal".to_string(), |code| format!("code {code}"))
            ));
            status.code().unwrap_or(exit::RUNTIME)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            output.print_error(
                "application launch requires Cargo to be installed and available on PATH",
            );
            exit::CONFIG
        }
        Err(error) => {
            output.print_error(&format!("failed to start application: {error}"));
            exit::RUNTIME
        }
    }
}

fn run_watch(output: &Output) -> i32 {
    let mut command = std::process::Command::new("cargo");
    command.args(["watch", "--why", "-x", "run"]);
    if output.is_verbose() && !output.is_quiet() {
        output.print_verbose("starting cargo watch -x run");
    }
    match command.status() {
        Ok(status) if status.success() => exit::SUCCESS,
        Ok(status) => status.code().unwrap_or(exit::RUNTIME),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            output.print_error(
                "automatic reload requires 'cargo-watch'; install it with `cargo install cargo-watch`",
            );
            exit::CONFIG
        }
        Err(error) => {
            output.print_error(&format!("failed to start automatic reload: {error}"));
            exit::RUNTIME
        }
    }
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

    if is_arqen_application() {
        return run_application(
            "production",
            file,
            host,
            port,
            log,
            storage,
            log_format,
            false,
            output,
        );
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
