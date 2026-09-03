pub mod build;
pub mod check;
pub mod doc;
pub mod doctor;
pub mod exit;
pub mod format;
pub mod generate;
pub mod lint;
#[cfg(feature = "thingd-migration")]
pub mod migration;
pub mod output;
pub mod serve;
pub mod store;
pub mod test;
pub mod thingd_schema;

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use output::Output;

#[derive(Parser)]
#[command(
    name = "arqen",
    version,
    about = "Arqen CLI for generating and running applications"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true)]
    json: bool,
    #[arg(long, global = true)]
    verbose: bool,
    #[arg(long, global = true)]
    quiet: bool,
    #[arg(long, global = true, default_value = "auto")]
    color: ColorChoice,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ColorChoice {
    Auto,
    Always,
    Never,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a new Arqen application
    New {
        /// Project name
        name: String,
        /// Output directory (defaults to the project name)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Accept the default project options without prompting
        #[arg(long)]
        yes: bool,
        /// Do not include the Arqen HTTP server feature
        #[arg(long)]
        no_http: bool,
        /// Include embedded native Thingd storage
        #[arg(long)]
        thingd: bool,
        /// Do not include embedded native Thingd storage
        #[arg(long)]
        no_thingd: bool,
        /// Do not include Arqen logging
        #[arg(long)]
        no_logging: bool,
        /// Include starter module, tool, and job guidance
        #[arg(long)]
        examples: bool,
        /// Include optional Nice Code documentation and CI configuration
        #[arg(long)]
        nice_code: bool,
    },
    /// Generate code scaffolding
    Generate {
        #[command(subcommand)]
        kind: GenerateKind,
    },
    /// Run the application in development mode
    Dev {
        /// Host to bind to
        #[arg(long)]
        host: Option<String>,
        /// Port to bind to
        #[arg(short, long)]
        port: Option<u16>,
        /// Log level
        #[arg(short, long)]
        log: Option<String>,
        /// Storage mode
        #[arg(short, long)]
        storage: Option<String>,
        /// Log output format
        #[arg(long, value_enum)]
        log_format: Option<crate::config::LogFormat>,
        /// Config file path
        #[arg(long = "file")]
        config_file: Option<String>,
        /// Restart the application after Rust source/configuration changes.
        #[arg(long)]
        watch: bool,
    },
    /// Run the application (aliases: run, serve)
    #[command(visible_alias = "run", alias = "serve")]
    Start {
        /// Host to bind to
        #[arg(long)]
        host: Option<String>,
        /// Port to bind to
        #[arg(short, long)]
        port: Option<u16>,
        /// Log level
        #[arg(short, long)]
        log: Option<String>,
        /// Storage mode
        #[arg(short, long)]
        storage: Option<String>,
        /// Log output format (production defaults to JSON)
        #[arg(long, value_enum)]
        log_format: Option<crate::config::LogFormat>,
        /// Config file path
        #[arg(long = "file")]
        config_file: Option<String>,
        /// Defer native schema validation to the application startup workflow
        #[arg(long)]
        skip_schema_validation: bool,
    },
    /// Run and supervise local dev services defined in arqen.toml
    Up {
        /// Names of services to start (defaults to all)
        #[arg(value_name = "SERVICE")]
        services: Vec<String>,
        /// Config file to read dev services from
        #[arg(long, default_value = "arqen.toml")]
        file: std::path::PathBuf,
        /// Print the services that would be started without running them
        #[arg(long)]
        dry_run: bool,
        /// Preserve child output without Arqen service prefixes.
        #[arg(long)]
        raw: bool,
        /// Wait for configured service readiness URLs after startup.
        #[arg(long)]
        wait_ready: bool,
    },
    /// Run checks on the current environment
    Check,
    /// Diagnose Rust, thingd, Docker, and environment setup
    Doctor,
    /// Run lint checks (fmt + clippy)
    Lint,
    /// Auto-format code
    Format,
    /// Run tests
    Test {
        /// Build and run in release mode
        #[arg(long)]
        release: bool,
    },
    /// Build the project
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Generate documentation
    Doc,
    /// Validate Thingd schemas and inspect a remote Thingd instance.
    Thingd {
        #[command(subcommand)]
        command: ThingdCommand,
    },
    /// Export or import an embedded native Thingd store.
    Store {
        #[command(subcommand)]
        command: StoreCommand,
    },
}

#[derive(Subcommand)]
pub enum StoreCommand {
    /// Export native store files, including lock metadata, to a gzip archive.
    Export {
        #[arg(long)]
        output: std::path::PathBuf,
        #[arg(long)]
        data_dir: Option<std::path::PathBuf>,
    },
    /// Import a native store archive into a directory.
    Import {
        #[arg(long)]
        input: std::path::PathBuf,
        #[arg(long)]
        data_dir: Option<std::path::PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum ThingdCommand {
    /// Migrate an embedded native Thingd store to a standalone HTTP server.
    #[cfg(feature = "thingd-migration")]
    Migrate {
        #[arg(long)]
        source: std::path::PathBuf,
        #[arg(long)]
        destination: String,
        #[arg(long)]
        auth_token: Option<String>,
        #[arg(long, default_value_t = 100)]
        batch_size: usize,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        check: bool,
        #[arg(long)]
        include_replication: bool,
        #[arg(long)]
        encryption_key: Option<String>,
    },
    /// Validate a local `.thingd` schema file.
    SchemaValidate {
        path: std::path::PathBuf,
        /// Optional Thingd URL for authoritative syntax validation.
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        token: Option<String>,
    },
    /// Read the current remote schema and migration history.
    SchemaRemote {
        url: String,
        #[arg(long)]
        token: Option<String>,
    },
    /// Seed a remote Thingd instance with bounded startup retries.
    Seed {
        url: String,
        #[arg(long)]
        token: Option<String>,
        #[arg(long, default_value_t = 12)]
        attempts: u32,
    },
}

#[derive(Subcommand)]
pub enum GenerateKind {
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

fn dispatch(cli: &Cli, output: &Output) -> anyhow::Result<()> {
    match &cli.command {
        Commands::New {
            name,
            output: output_dir,
            yes,
            no_http,
            thingd,
            no_thingd,
            no_logging,
            examples,
            nice_code,
        } => {
            let options = generate::ProjectOptions {
                output_dir: output_dir.clone(),
                yes: *yes,
                http: !*no_http,
                thingd: *thingd && !*no_thingd,
                logging: !*no_logging,
                examples: *examples,
                nice_code: *nice_code,
            };
            generate::generate_project(name, options, output)?;
        }
        Commands::Generate { kind } => match kind {
            GenerateKind::Module { name } => generate::generate_module(name, output)?,
            GenerateKind::Tool { name } => generate::generate_tool(name, output)?,
            GenerateKind::Job { name } => generate::generate_job(name, output)?,
        },
        Commands::Dev {
            host,
            port,
            log,
            storage,
            config_file,
            log_format,
            watch,
        } => {
            let code = serve::serve_dev(
                config_file.as_deref(),
                host.as_deref(),
                *port,
                log.as_deref(),
                storage.as_deref(),
                log_format.as_ref(),
                *watch,
                output,
            );
            if code != exit::SUCCESS {
                std::process::exit(code);
            }
        }
        Commands::Start {
            host,
            port,
            log,
            storage,
            config_file,
            log_format,
            skip_schema_validation,
        } => {
            let code = serve::serve_start(
                config_file.as_deref(),
                host.as_deref(),
                *port,
                log.as_deref(),
                storage.as_deref(),
                log_format.as_ref(),
                *skip_schema_validation,
                output,
            );
            if code != exit::SUCCESS {
                std::process::exit(code);
            }
        }
        Commands::Up {
            services,
            file,
            dry_run,
            raw,
            wait_ready,
        } => {
            let rt = tokio::runtime::Runtime::new().map_err(|e| anyhow::anyhow!(e))?;
            rt.block_on(crate::dev::run_up(
                file,
                services,
                *dry_run,
                *raw,
                *wait_ready,
            ))?;
        }
        Commands::Check => {
            let code = check::run_check(output);
            if code != exit::SUCCESS {
                std::process::exit(code);
            }
        }
        Commands::Doctor => {
            let code = doctor::run_doctor(output);
            if code != exit::SUCCESS {
                std::process::exit(code);
            }
        }
        Commands::Lint => {
            let code = lint::run_lint(output);
            if code != exit::SUCCESS {
                std::process::exit(code);
            }
        }
        Commands::Format => {
            let code = format::run_format(output);
            if code != exit::SUCCESS {
                std::process::exit(code);
            }
        }
        Commands::Test { release } => {
            let code = test::run_test(*release, output);
            if code != exit::SUCCESS {
                std::process::exit(code);
            }
        }
        Commands::Build { release } => {
            let code = build::run_build(*release, output);
            if code != exit::SUCCESS {
                std::process::exit(code);
            }
        }
        Commands::Doc => {
            let code = doc::run_doc(output);
            if code != exit::SUCCESS {
                std::process::exit(code);
            }
        }
        Commands::Thingd { command } => {
            #[cfg(feature = "thingd-migration")]
            if let ThingdCommand::Migrate {
                source,
                destination,
                auth_token,
                batch_size,
                dry_run,
                check,
                include_replication,
                encryption_key,
            } = command
            {
                let options = crate::migration::ThingdMigrationOptions {
                    source_path: source.clone(),
                    destination_url: destination.clone(),
                    destination_auth_token: auth_token.clone(),
                    dry_run: *dry_run,
                    include_replication: *include_replication,
                    batch_size: *batch_size,
                    source_encryption_key: encryption_key.clone(),
                    ..Default::default()
                };
                migration::run(options, *check, output)?;
                return Ok(());
            }
            let code = thingd_schema::run(command, output);
            if code != exit::SUCCESS {
                std::process::exit(code);
            }
        }
        Commands::Store { command } => {
            let code = match command {
                StoreCommand::Export {
                    output: path,
                    data_dir,
                } => store::export(data_dir.as_deref(), path, output),
                StoreCommand::Import {
                    input: path,
                    data_dir,
                } => store::import(data_dir.as_deref(), path, output),
            };
            if code != exit::SUCCESS {
                std::process::exit(code);
            }
        }
    }
    Ok(())
}

pub fn run() -> i32 {
    let cli = Cli::parse();
    let color = match cli.color {
        ColorChoice::Auto => "auto",
        ColorChoice::Always => "always",
        ColorChoice::Never => "never",
    };
    let output = Output::from_args(cli.json, cli.quiet, cli.verbose, color);

    match dispatch(&cli, &output) {
        Ok(()) => exit::SUCCESS,
        Err(e) => {
            output.print_error(&e.to_string());
            exit::RUNTIME
        }
    }
}
