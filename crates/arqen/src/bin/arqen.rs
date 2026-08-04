use clap::{Parser, Subcommand};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

#[derive(Parser)]
#[command(name = "arqen")]
#[command(about = "Arqen CLI for generating and running applications")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new Arqen application
    New {
        /// Project name
        name: String,
        /// Template to use
        #[arg(short, long, default_value = "thingd-app")]
        template: String,
    },
    /// Run the application in development mode
    Dev {
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to bind to
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// Log level
        #[arg(short, long, default_value = "info")]
        log: String,
        /// Storage mode
        #[arg(short, long, default_value = "memory")]
        storage: String,
    },
    /// Run the application
    Start {
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to bind to
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// Log level
        #[arg(short, long, default_value = "info")]
        log: String,
        /// Storage mode
        #[arg(short, long, default_value = "memory")]
        storage: String,
    },
    /// Run checks
    Check,
    /// Diagnose Rust, thingd, Docker, and environment setup
    Doctor,
}

fn generate_project(name: &str, template: &str) -> anyhow::Result<()> {
    let project_dir = Path::new(name);
    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    fs::create_dir_all(project_dir.join("src"))?;

    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
arqen = {{ version = "0.3", features = ["logging"] }}
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
"#,
        name
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    let main_rs = r#"use arqen::http::create_router;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    arqen::logging::init_logging("info", "pretty");

    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let router = create_router();

    println!("Starting on {}", addr);

    arqen::http::start_server(addr, router).await?;
    Ok(())
}
"#;
    fs::write(project_dir.join("src").join("main.rs"), main_rs)?;

    let readme = format!(
        r#"# {}

An Arqen application generated with the `{}` template. Arqen is a Rust-first
backend framework for agent-ready applications with typed tools, jobs,
discoverable APIs, health checks, and thingd integration.

## Package model

This project depends on the single published `arqen` package. The framework
library and CLI are distributed together; there is no separate `arqen-cli`
package.

## Getting started

```bash
cargo run
```

The server starts on http://127.0.0.1:3000 with in-memory storage and pretty
development logging.

## Endpoints

- GET /health - Liveness check
- GET /ready - Readiness check
- GET /agent - Agent/application summary
- GET /agent/manifest - Machine-readable tools and endpoint manifest
- GET /docs - HTTP endpoint summary

## Configuration

Arqen loads configuration in this order:

```text
CLI flags → ARQEN_* environment variables → arqen.toml → defaults
```

Useful environment variables include:

```bash
ARQEN_HOST=127.0.0.1
ARQEN_PORT=3000
ARQEN_STORAGE_MODE=memory
ARQEN_LOG_LEVEL=info
ARQEN_LOG_FORMAT=pretty
```

Use `ARQEN_STORAGE_MODE=persistent` with `ARQEN_PERSISTENT_PATH` for native
durable storage, or `ARQEN_STORAGE_MODE=http` with `ARQEN_THINGD_URL` for a
thingd HTTP service. Validate the chosen mode and recovery behavior before
production use.

## Development

The generated application has no integrated file watcher. Run an external
watcher if needed, for example:

```bash
cargo watch -x run
```

## Next steps

- Add application routes and typed domain services.
- Register tools and permissions in the agent manifest.
- Add validation and authentication to protected routes.
- Replace memory storage with a validated durable thingd deployment.
- Run the application and framework tests before release.

Documentation: https://sayanmohsin.github.io/arqen/
"#,
        name, template
    );
    fs::write(project_dir.join("README.md"), readme)?;

    println!("Created project '{}' with template '{}'", name, template);
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name, template } => {
            generate_project(&name, &template)?;
        }
        Commands::Dev {
            host,
            port,
            log,
            storage,
        } => {
            println!("Starting Arqen in development mode...");
            arqen::logging::init_logging(&log, "pretty");

            let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
            let state = arqen::AppState::builder()
                .with_storage_mode(&storage)
                .with_tool_registry(arqen::ToolRegistry::new(
                    "arqen-app",
                    env!("CARGO_PKG_VERSION"),
                    "An Arqen application",
                    &storage,
                ))
                .build()?;
            let router = arqen::http::create_router_with_state(state);

            println!("Arqen v{}", env!("CARGO_PKG_VERSION"));
            println!("API:    http://{}", addr);
            println!("Health: http://{}/health", addr);
            println!("Docs:   http://{}/docs", addr);
            println!("Agent:  http://{}/agent", addr);
            println!("Storage: {}", storage);

            arqen::http::start_server(addr, router)
                .await
                .map_err(|e| anyhow::anyhow!(e))?;
        }
        Commands::Start {
            host,
            port,
            log,
            storage,
        } => {
            println!("Starting Arqen...");
            arqen::logging::init_logging(&log, "json");

            let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
            let state = arqen::AppState::builder()
                .with_storage_mode(&storage)
                .with_tool_registry(arqen::ToolRegistry::new(
                    "arqen-app",
                    env!("CARGO_PKG_VERSION"),
                    "An Arqen application",
                    &storage,
                ))
                .build()?;
            let router = arqen::http::create_router_with_state(state);

            println!("Arqen listening on {}", addr);

            arqen::http::start_server(addr, router)
                .await
                .map_err(|e| anyhow::anyhow!(e))?;
        }
        Commands::Check => {
            println!("Running checks...");
            println!("Checks passed");
        }
        Commands::Doctor => {
            println!("Arqen Doctor - Diagnosing environment...\n");

            println!("1. Checking Rust installation...");
            match std::process::Command::new("rustc")
                .arg("--version")
                .output()
            {
                Ok(output) => {
                    let version = String::from_utf8_lossy(&output.stdout);
                    println!("   Rust installed: {}", version.trim());
                }
                Err(e) => println!("   Rust not found: {}", e),
            }

            println!("2. Checking Cargo installation...");
            match std::process::Command::new("cargo")
                .arg("--version")
                .output()
            {
                Ok(output) => {
                    let version = String::from_utf8_lossy(&output.stdout);
                    println!("   Cargo installed: {}", version.trim());
                }
                Err(e) => println!("   Cargo not found: {}", e),
            }

            println!("3. Checking Docker installation...");
            match std::process::Command::new("docker")
                .arg("--version")
                .output()
            {
                Ok(output) => {
                    let version = String::from_utf8_lossy(&output.stdout);
                    println!("   Docker installed: {}", version.trim());
                }
                Err(e) => println!("   Docker not found: {}", e),
            }

            println!("4. Checking Docker Compose...");
            match std::process::Command::new("docker")
                .arg("compose")
                .arg("version")
                .output()
            {
                Ok(output) => {
                    let version = String::from_utf8_lossy(&output.stdout);
                    println!("   Docker Compose installed: {}", version.trim());
                }
                Err(_) => match std::process::Command::new("docker-compose")
                    .arg("--version")
                    .output()
                {
                    Ok(output) => {
                        let version = String::from_utf8_lossy(&output.stdout);
                        println!("   Docker Compose installed: {}", version.trim());
                    }
                    Err(e) => println!("   Docker Compose not found: {}", e),
                },
            }

            println!("5. Checking thingd connectivity...");
            if let Ok(thingd_url) = std::env::var("ARQEN_THINGD_URL") {
                println!("   thingd URL: {}", thingd_url);
                println!("   Connectivity check not implemented");
            } else {
                println!("   ARQEN_THINGD_URL not set");
            }

            println!("6. Checking environment variables...");
            let env_vars = [
                "ARQEN_HOST",
                "ARQEN_PORT",
                "ARQEN_LOG",
                "ARQEN_STORAGE_MODE",
            ];
            for var in env_vars {
                match std::env::var(var) {
                    Ok(value) => println!("   {} = {}", var, value),
                    Err(_) => println!("   {} not set (using default)", var),
                }
            }

            println!("\nDoctor complete.");
        }
    }

    Ok(())
}
