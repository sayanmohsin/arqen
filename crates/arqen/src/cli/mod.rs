pub mod check;
pub mod doctor;
pub mod exit;
pub mod generate;
pub mod output;
pub mod serve;

use clap::{Parser, Subcommand};

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
    color: String,
}

#[derive(Subcommand)]
pub enum Commands {
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
        /// Config file path
        #[arg(long = "file")]
        config_file: Option<String>,
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
        /// Config file path
        #[arg(long = "file")]
        config_file: Option<String>,
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
    },
    /// Run checks on the current environment
    Check,
    /// Diagnose Rust, thingd, Docker, and environment setup
    Doctor,
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
        Commands::New { name } => {
            generate::generate_project(name, output)?;
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
        } => {
            let code = serve::serve_dev(config_file.as_deref(), host, *port, log, storage, output);
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
        } => {
            let code =
                serve::serve_start(config_file.as_deref(), host, *port, log, storage, output);
            if code != exit::SUCCESS {
                std::process::exit(code);
            }
        }
        Commands::Up {
            services,
            file,
            dry_run,
        } => {
            let rt = tokio::runtime::Runtime::new().map_err(|e| anyhow::anyhow!(e))?;
            rt.block_on(crate::dev::run_up(file, services, *dry_run))?;
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
    }
    Ok(())
}

pub fn run() -> i32 {
    let cli = Cli::parse();
    let output = Output::from_args(cli.json, cli.quiet, cli.verbose, &cli.color);

    match dispatch(&cli, &output) {
        Ok(()) => exit::SUCCESS,
        Err(e) => {
            output.print_error(&e.to_string());
            exit::RUNTIME
        }
    }
}
