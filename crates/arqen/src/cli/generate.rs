use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone)]
pub struct ProjectOptions {
    pub output_dir: Option<PathBuf>,
    pub yes: bool,
    pub http: bool,
    pub thingd: bool,
    pub logging: bool,
    pub examples: bool,
    pub nice_code: bool,
}

impl Default for ProjectOptions {
    fn default() -> Self {
        Self {
            output_dir: None,
            yes: false,
            http: true,
            thingd: false,
            logging: true,
            examples: false,
            nice_code: false,
        }
    }
}

fn ask_yes_no(question: &str, default: bool) -> anyhow::Result<bool> {
    let suffix = if default { "Y/n" } else { "y/N" };
    print!("{question} [{suffix}] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    match answer.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        "" => Ok(default),
        _ => anyhow::bail!("Please answer yes or no"),
    }
}

fn resolve_options(mut options: ProjectOptions, output: &Output) -> anyhow::Result<ProjectOptions> {
    if options.yes || output.is_json() || output.is_quiet() || !io::stdin().is_terminal() {
        return Ok(options);
    }

    output.print("Configure your new Arqen application:");
    options.http = ask_yes_no("Include the HTTP server?", options.http)?;
    options.thingd = ask_yes_no("Include embedded native Thingd storage?", options.thingd)?;
    options.logging = ask_yes_no("Include structured logging?", options.logging)?;
    options.examples = ask_yes_no(
        "Include starter module, tool, and job examples?",
        options.examples,
    )?;
    options.nice_code = ask_yes_no(
        "Add optional Nice Code documentation and CI?",
        options.nice_code,
    )?;
    Ok(options)
}

fn package_name(name: &str, project_dir: &Path, explicit_output: bool) -> String {
    let candidate = if explicit_output {
        name.to_string()
    } else {
        project_dir
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| name.to_string())
    };
    candidate.replace('-', "_")
}

pub fn generate_project(
    name: &str,
    options: ProjectOptions,
    output: &Output,
) -> anyhow::Result<()> {
    let options = resolve_options(options, output)?;
    let project_dir = options
        .output_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(name));
    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", project_dir.display());
    }

    if name.trim().is_empty() {
        anyhow::bail!("Project name must be a non-empty package name");
    }
    let pkg_name = package_name(name, &project_dir, options.output_dir.is_some());
    if options.output_dir.is_some() && (name.contains('/') || name.contains('\\')) {
        anyhow::bail!("Project name must not contain a path separator when --output is used");
    }
    if pkg_name.is_empty() || pkg_name.starts_with('.') || pkg_name.contains(' ') {
        anyhow::bail!("Project name must be a valid Rust package name");
    }

    fs::create_dir_all(project_dir.join("src"))?;
    if options.http {
        fs::create_dir_all(project_dir.join("src").join("app"))?;
    }

    let mut features = Vec::new();
    if options.http {
        features.push("http-server");
        features.push("advanced-transport");
    }
    if options.thingd {
        features.push("thingd-native");
    }
    if options.logging {
        features.push("logging");
    }
    let feature_text = features
        .iter()
        .map(|feature| format!("\"{feature}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let cargo_toml = format!(
        r#"[package]
name = "{pkg_name}"
version = "0.1.0"
edition = "2024"
rust-version = "1.96"

[dependencies]
arqen = {{ version = "{arqen_version}", default-features = false, features = [{feature_text}] }}
"#,
        arqen_version = env!("CARGO_PKG_VERSION"),
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    let main_rs = if options.http {
        let logging = if options.logging {
            "    arqen::logging::init_logging(\"info\", \"compact\");\n"
        } else {
            ""
        };
        format!(
            r#"use arqen::config::{{AppConfig, CliOverrides}};
use arqen::http::{{create_router_with_state_and_routes, start_server}};
use arqen::http::raw::Router;
use arqen::state::AppState;
use std::net::SocketAddr;

mod app;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {{
    arqen::run(async {{
{logging}    let config = AppConfig::load(CliOverrides::default())?;
    let state = AppState::builder()
        .with_config(config)
        .with_modules(vec![app::AppModule])?
        .build()?;
    let addr: SocketAddr = format!("{{}}:{{}}", state.config.server.host, state.config.server.port).parse()?;
    let routes: Router = app::routes();
    start_server(addr, create_router_with_state_and_routes(state, routes)).await
    }})
}}
"#
        )
    } else {
        r#"use arqen::config::{AppConfig, CliOverrides};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    arqen::run(async {
    let config = AppConfig::load(CliOverrides::default())?;
    if std::env::var_os("ARQEN_LAUNCH_MODE").as_deref() == Some(std::ffi::OsStr::new("production")) {{
        config.validate_production()?;
    }}
    println!("{} is configured for {} storage", env!("CARGO_PKG_NAME"), config.storage.mode.as_str());
    Ok(())
    })
}
"#.to_string()
    };
    fs::write(project_dir.join("src").join("main.rs"), main_rs)?;

    if options.http {
        let app_mod = r#"use arqen::module::Module;
use arqen::http::raw::{Router, routing::get};

pub struct AppModule;

impl Module for AppModule {
    fn name(&self) -> &str {
        "app"
    }

}

pub fn routes() -> Router {
    async fn hello() -> &'static str {
        "hello from Arqen"
    }

    Router::new().route("/api/hello", get(hello))
}
"#;
        fs::write(project_dir.join("src").join("app").join("mod.rs"), app_mod)?;
    }

    let storage = if options.thingd { "native" } else { "memory" };
    let readme = format!(
        r#"# {pkg_name}

An Arqen application generated by Arqen {arqen_version}.

## Getting started

```bash
arqen dev
```

{runtime_note}

For automatic restart during development, install the optional file watcher
and run `arqen dev --watch`. Use `arqen start` for a production-style launch.

## Endpoints

- GET /health - Liveness check
- GET /ready - Readiness check
- GET /agent - Agent description
- GET /agent/manifest - Agent manifest
- GET /docs - API documentation
{endpoint_note}

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
"#,
        arqen_version = env!("CARGO_PKG_VERSION"),
        endpoint_note = if options.http {
            "- GET /api/hello - Example application route"
        } else {
            ""
        },
        runtime_note = if options.http {
            format!("The server starts on http://127.0.0.1:8888 with {storage} storage.")
        } else {
            "This starter is a library-ready command-line application. Add the `http-server` feature when you are ready to expose HTTP routes.".to_string()
        }
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

    let config = if options.thingd {
        "[storage]\nmode = \"native\"\npersistent_path = \".data/thingd\"\n"
    } else {
        "[storage]\nmode = \"memory\"\n"
    };
    fs::write(project_dir.join("arqen.toml"), config)?;
    fs::write(
        project_dir.join(".env.example"),
        "# Arqen configuration\nARQEN_LOG_LEVEL=info\nARQEN_LOG_FORMAT=compact\nARQEN_PORT=8888\n",
    )?;
    fs::write(project_dir.join("AGENTS.md"), agent_guide())?;

    let mut files = vec![
        "Cargo.toml",
        "src/main.rs",
        "README.md",
        "rustfmt.toml",
        "clippy.toml",
        "arqen.toml",
        ".env.example",
        "AGENTS.md",
    ];
    if options.http {
        files.push("src/app/mod.rs");
    }
    if options.examples {
        fs::create_dir_all(project_dir.join("examples"))?;
        fs::write(
            project_dir.join("examples").join("README.md"),
            examples_guide(),
        )?;
        files.push("examples/README.md");
    }
    if options.nice_code {
        fs::create_dir_all(project_dir.join(".github").join("workflows"))?;
        fs::write(project_dir.join("NICE_CODE.md"), nice_code_guide())?;
        fs::write(
            project_dir
                .join(".github")
                .join("workflows")
                .join("nice-code.yml"),
            nice_code_workflow(),
        )?;
        files.push("NICE_CODE.md");
        files.push(".github/workflows/nice-code.yml");
    }

    if output.is_json() {
        let summary = serde_json::json!({
            "command": "new",
            "project": name,
            "output": project_dir,
            "options": {"http": options.http, "thingd": options.thingd, "logging": options.logging, "examples": options.examples, "nice_code": options.nice_code},
            "files": files,
        });
        output.print_json(summary);
    } else {
        output.print(&format!("Created project '{}'", project_dir.display()));
        output.print("");
        output.print("Next steps:");
        output.print(&format!("  cd {}", project_dir.display()));
        output.print("  arqen dev");
    }
    Ok(())
}

fn agent_guide() -> &'static str {
    r#"# Project guidance

- Keep application-specific domain logic in `src/`; keep framework changes in Arqen itself.
- Do not commit secrets. Use `.env.example` as a template only.
- Preserve in-memory and native storage behavior when adding persistence-dependent code.
- Run `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` before submitting changes.
- Keep optional tools such as Nice Code outside the runtime dependency graph.
"#
}

fn examples_guide() -> &'static str {
    r#"# Starter examples

Use the built-in generators from the project root:

```bash
arqen generate module users
arqen generate tool get_user
arqen generate job send_email
```

Register generated modules in `src/app/mod.rs`, then add their routes and
capabilities to the application state during startup.
"#
}

fn nice_code_guide() -> &'static str {
    r#"# Optional Nice Code checks

Nice Code is an optional development and CI tool. It is not an Arqen runtime
dependency and is not required to build or run this application.

Run it without installing it permanently:

```bash
npx --yes @sayanmohsin/nice-code@0.1.11 --changed --project .
```
"#
}

fn nice_code_workflow() -> &'static str {
    r#"name: Nice Code

on:
  pull_request:
  push:
    branches: [main]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - run: npx --yes @sayanmohsin/nice-code@0.1.11 --ci --project . --format sarif
"#
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
        r#"use arqen::module::{{Module, ModuleContext}};
use arqen::core::AppError;

pub struct {module_name}Module;

impl Module for {module_name}Module {{
    fn name(&self) -> &str {{
        "{name}"
    }}

    fn register(&self, _ctx: &mut ModuleContext<'_>) -> Result<(), AppError> {{
        // Register tools: ctx.tools.register_tool(...)
        Ok(())
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
