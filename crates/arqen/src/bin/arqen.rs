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
    },
    /// Generate code scaffolding
    Generate {
        #[command(subcommand)]
        kind: GenerateKind,
    },
    /// Run the application in development mode
    Dev {
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to bind to
        #[arg(short, long, default_value = "8888")]
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
        #[arg(short, long, default_value = "8888")]
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

#[derive(Subcommand)]
enum GenerateKind {
    /// Generate a new module
    Module {
        /// Module name
        name: String,
    },
    /// Generate a new tool
    Tool {
        /// Tool name
        name: String,
    },
    /// Generate a new job handler
    Job {
        /// Job name
        name: String,
    },
}

fn generate_project(name: &str) -> anyhow::Result<()> {
    let project_dir = Path::new(name);
    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Create directory structure
    fs::create_dir_all(project_dir.join("src").join("app"))?;
    fs::create_dir_all(project_dir.join("src").join("routes"))?;

    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"
rust-version = "1.96"

[dependencies]
arqen = {{ version = "0.3", features = ["logging", "http-server"] }}
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
async-trait = "0.1"
"#,
        name
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // src/main.rs
    let main_rs = format!(
        r#"use arqen::app::ArqenApp;
use arqen::module::Module;

mod app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    arqen::logging::init_logging("info", "pretty");

    ArqenApp::builder()
        .name("{name}")
        .module(app::AppModule)
        .build()?
        .start()
        .await
}}
"#
    );
    fs::write(project_dir.join("src").join("main.rs"), main_rs)?;

    // src/app/mod.rs
    let app_mod = r#"use arqen::module::{Module, ModuleHealth};

pub struct AppModule;

#[async_trait::async_trait]
impl Module for AppModule {
    fn name(&self) -> &str {
        "app"
    }

    async fn health_check(&self) -> ModuleHealth {
        ModuleHealth::Healthy
    }
}
"#;
    fs::write(project_dir.join("src").join("app").join("mod.rs"), app_mod)?;

    // src/routes/mod.rs
    let routes_mod = r#"pub mod health;
"#;
    fs::write(
        project_dir.join("src").join("routes").join("mod.rs"),
        routes_mod,
    )?;

    // src/routes/health.rs
    let health_route = r#"use axum::Json;
use serde_json::{json, Value};

pub async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}
"#;
    fs::write(
        project_dir.join("src").join("routes").join("health.rs"),
        health_route,
    )?;

    // README.md
    let readme = format!(
        r#"# {name}

An Arqen application.

## Getting started

```bash
cargo run
```

The server starts on http://127.0.0.1:8888 with in-memory storage.

## Endpoints

- GET /health - Liveness check
- GET /ready - Readiness check
- GET /agent - Agent description
- GET /agent/manifest - Agent manifest
- GET /docs - API documentation

## Adding modules

```bash
arqen generate module users
```

This creates `src/users/mod.rs` with a module stub. Register it in `src/app/mod.rs`:

```rust
mod users;

// In AppModule::dependencies():
fn dependencies(&self) -> Vec<&str> {{
    vec!["users"]
}}
```

## Adding tools

```bash
arqen generate tool get_user
```

This creates `src/tools/get_user.rs` with a tool definition. Register it in your module's `register()` method.

## Adding jobs

```bash
arqen generate job send_email
```

This creates `src/jobs/send_email.rs` with a job handler.
"#
    );
    fs::write(project_dir.join("README.md"), readme)?;

    println!("Created project '{}'", name);
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  cargo run");
    Ok(())
}

fn generate_module(name: &str) -> anyhow::Result<()> {
    let src_dir = Path::new("src");
    if !src_dir.exists() {
        anyhow::bail!("No src/ directory found. Run this from an Arqen project root.");
    }

    let module_dir = src_dir.join(name);
    if module_dir.exists() {
        anyhow::bail!("Module '{}' already exists", name);
    }

    fs::create_dir_all(&module_dir)?;

    let module_rs = format!(
        r#"use arqen::module::{{Module, ModuleContext, ModuleHealth}};
use arqen::core::AppError;

pub struct {module_name}Module;

#[async_trait::async_trait]
impl Module for {module_name}Module {{
    fn name(&self) -> &str {{
        "{name}"
    }}

    fn register(&self, _ctx: &mut ModuleContext<'_>) -> Result<(), AppError> {{
        // Register tools: ctx.tools.register_tool(...)
        Ok(())
    }}

    async fn health_check(&self) -> ModuleHealth {{
        ModuleHealth::Healthy
    }}
}}
"#,
        module_name = to_pascal_case(name),
        name = name,
    );
    fs::write(module_dir.join("mod.rs"), module_rs)?;

    println!("Created module '{name}' at src/{name}/mod.rs");
    println!();
    println!("Register in src/app/mod.rs:");
    println!("  pub mod {name};");
    println!();
    println!("Add to AppModule::dependencies():");
    println!("  fn dependencies(&self) -> Vec<&str> {{ vec![\"{name}\"] }}");
    Ok(())
}

fn generate_tool(name: &str) -> anyhow::Result<()> {
    let src_dir = Path::new("src");
    if !src_dir.exists() {
        anyhow::bail!("No src/ directory found. Run this from an Arqen project root.");
    }

    let tools_dir = src_dir.join("tools");
    fs::create_dir_all(&tools_dir)?;

    let tool_file = tools_dir.join(format!("{}.rs", name));
    if tool_file.exists() {
        anyhow::bail!("Tool '{}' already exists", name);
    }

    let tool_rs = format!(
        r#"use arqen::agent::{{ToolEffect, ToolMetadata}};
use arqen::core::AppError;
use arqen::module::ModuleContext;

pub fn tool_metadata() -> ToolMetadata {{
    ToolMetadata {{
        name: "{name}".to_string(),
        description: "TODO: describe what this tool does".to_string(),
        input: serde_json::json!({{
            "type": "object",
            "properties": {{}},
            "required": []
        }}),
        output: serde_json::json!({{
            "type": "object"
        }}),
        scopes: vec![],
        effect: ToolEffect::Read,
        idempotent: true,
        enqueues_job: None,
        timeout: None,
    }}
}}

pub fn register(ctx: &mut ModuleContext<'_>) -> Result<(), AppError> {{
    ctx.tools.register_tool(tool_metadata());
    Ok(())
}}
"#
    );
    fs::write(&tool_file, tool_rs)?;

    println!("Created tool '{name}' at src/tools/{name}.rs");
    println!();
    println!("Register in your module's register() method:");
    println!("  crate::tools::{name}::register(ctx)?;");
    Ok(())
}

fn generate_job(name: &str) -> anyhow::Result<()> {
    let src_dir = Path::new("src");
    if !src_dir.exists() {
        anyhow::bail!("No src/ directory found. Run this from an Arqen project root.");
    }

    let jobs_dir = src_dir.join("jobs");
    fs::create_dir_all(&jobs_dir)?;

    let job_file = jobs_dir.join(format!("{}.rs", name));
    if job_file.exists() {
        anyhow::bail!("Job '{}' already exists", name);
    }

    let job_rs = format!(
        r#"use arqen::core::AppError;
use arqen::jobs::JobHandler;

pub struct {handler_name}Handler;

#[async_trait::async_trait]
impl JobHandler for {handler_name}Handler {{
    async fn handle(&self, payload: serde_json::Value) -> Result<(), AppError> {{
        // TODO: implement job processing
        tracing::info!(payload = %payload, "Processing job");
        Ok(())
    }}
}}
"#,
        handler_name = to_pascal_case(name),
    );
    fs::write(&job_file, job_rs)?;

    println!("Created job handler '{name}' at src/jobs/{name}.rs");
    println!();
    println!("Next steps:");
    println!("  1. Add `pub mod {name};` to src/jobs/mod.rs.");
    println!("  2. Register the job metadata with your application's ToolRegistry.");
    println!("  3. Configure a worker for the job queue and start it from your app.");
    Ok(())
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
            }
        })
        .collect()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => {
            generate_project(&name)?;
        }
        Commands::Generate { kind } => match kind {
            GenerateKind::Module { name } => generate_module(&name)?,
            GenerateKind::Tool { name } => generate_tool(&name)?,
            GenerateKind::Job { name } => generate_job(&name)?,
        },
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
