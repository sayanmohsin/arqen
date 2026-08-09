use std::fs;
use std::path::Path;

use super::output::Output;

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

pub fn generate_project(name: &str, output: &Output) -> anyhow::Result<()> {
    let project_dir = Path::new(name);
    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    let pkg_name = project_dir
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| name.to_string());

    fs::create_dir_all(project_dir.join("src").join("app"))?;

    let cargo_toml = format!(
        r#"[package]
name = "{pkg_name}"
version = "0.1.0"
edition = "2024"
rust-version = "1.96"

[dependencies]
arqen = {{ version = "0.6", features = ["logging"] }}
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
async-trait = "0.1"
tracing = "0.1"
"#,
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    let main_rs = format!(
        r#"use arqen::app::ArqenApp;

mod app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {{
    arqen::logging::init_logging("info", "pretty");

    ArqenApp::builder()
        .name("{pkg_name}")
        .module(app::AppModule)
        .build()?
        .start()
        .await
}}
"#
    );
    fs::write(project_dir.join("src").join("main.rs"), main_rs)?;

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

    let readme = format!(
        r#"# {pkg_name}

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

## Code quality

```bash
arqen lint       # check formatting and clippy warnings
arqen format     # auto-fix formatting
arqen test       # run tests
arqen build      # build the project
```

See the [tooling guide](https://sayanmohsin.github.io/arqen/tooling)
for details.
"#
    );
    fs::write(project_dir.join("README.md"), readme)?;

    let rustfmt_toml = r#"edition = "2024"
max_width = 100
use_field_init_shorthand = true
use_try_shorthand = true
"#;
    fs::write(project_dir.join("rustfmt.toml"), rustfmt_toml)?;

    let clippy_toml = r#"avoid-breaking-exported-api = true
"#;
    fs::write(project_dir.join("clippy.toml"), clippy_toml)?;

    if output.is_json() {
        let summary = serde_json::json!({
            "command": "new",
            "project": name,
            "files": [
                "Cargo.toml",
                "src/main.rs",
                "src/app/mod.rs",
                "README.md",
                "rustfmt.toml",
                "clippy.toml",
            ],
        });
        output.print_json(summary);
    } else {
        output.print(&format!("Created project '{}'", name));
        output.print("");
        output.print("Next steps:");
        output.print(&format!("  cd {}", name));
        output.print("  cargo run");
    }
    Ok(())
}

pub fn generate_module(name: &str, output: &Output) -> anyhow::Result<()> {
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

    if output.is_json() {
        let summary = serde_json::json!({
            "command": "generate",
            "kind": "module",
            "name": name,
            "path": format!("src/{}/mod.rs", name),
        });
        output.print_json(summary);
    } else {
        output.print(&format!("Created module '{}' at src/{}/mod.rs", name, name));
        output.print("");
        output.print("Register in src/app/mod.rs:");
        output.print(&format!("  pub mod {};", name));
        output.print("");
        output.print("Add to AppModule::dependencies():");
        output.print(&format!(
            "  fn dependencies(&self) -> Vec<&str> {{ vec![\"{}\"] }}",
            name
        ));
    }
    Ok(())
}

pub fn generate_tool(name: &str, output: &Output) -> anyhow::Result<()> {
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

    if output.is_json() {
        let summary = serde_json::json!({
            "command": "generate",
            "kind": "tool",
            "name": name,
            "path": format!("src/tools/{}.rs", name),
        });
        output.print_json(summary);
    } else {
        output.print(&format!("Created tool '{}' at src/tools/{}.rs", name, name));
        output.print("");
        output.print("Register in your module's register() method:");
        output.print(&format!("  crate::tools::{}::register(ctx)?;", name));
    }
    Ok(())
}

pub fn generate_job(name: &str, output: &Output) -> anyhow::Result<()> {
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

    if output.is_json() {
        let summary = serde_json::json!({
            "command": "generate",
            "kind": "job",
            "name": name,
            "path": format!("src/jobs/{}.rs", name),
        });
        output.print_json(summary);
    } else {
        output.print(&format!(
            "Created job handler '{}' at src/jobs/{}.rs",
            name, name
        ));
        output.print("");
        output.print("Next steps:");
        output.print(&format!("  1. Add `pub mod {};` to src/jobs/mod.rs.", name));
        output.print("  2. Register the job metadata with your application's ToolRegistry.");
        output.print("  3. Configure a worker for the job queue and start it from your app.");
    }
    Ok(())
}
