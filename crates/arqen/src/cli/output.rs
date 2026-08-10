#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

impl ColorChoice {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "always" | "yes" | "true" => Self::Always,
            "never" | "no" | "false" => Self::Never,
            _ => Self::Auto,
        }
    }
}

use std::io::IsTerminal;

pub struct Output {
    json: bool,
    quiet: bool,
    verbose: bool,
    #[allow(dead_code)]
    color: ColorChoice,
}

impl Output {
    pub fn from_args(json: bool, quiet: bool, verbose: bool, color: &str) -> Self {
        Self {
            json,
            quiet,
            verbose,
            color: ColorChoice::parse(color),
        }
    }

    pub fn is_json(&self) -> bool {
        self.json
    }

    pub fn is_quiet(&self) -> bool {
        self.quiet
    }

    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub fn print(&self, msg: &str) {
        if !self.quiet {
            println!("{}", msg);
        }
    }

    pub fn print_banner(&self, addr: &std::net::SocketAddr, storage: &str, environment: &str) {
        if self.json || self.quiet {
            return;
        }
        let use_color = match self.color {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => std::io::stdout().is_terminal(),
        };
        let cyan = if use_color { "\x1b[36m" } else { "" };
        let dim = if use_color { "\x1b[2m" } else { "" };
        let reset = if use_color { "\x1b[0m" } else { "" };
        println!(
            "{cyan}     _\n    / \\\n   / __ \\  _ __ __ _  ___ _ __\n  / /  \\ \\| '__/ _` |/ _ \\ '__|\n / /____\\ \\| | | (_| |  __/ |\n/_/      \\_\\_|  \\__, |\\___|_|\n                  |___/{reset}"
        );
        println!(
            "{dim}  Arqen v{} · {} · {}{reset}",
            env!("CARGO_PKG_VERSION"),
            environment,
            storage
        );
        println!(
            "  API:    http://{addr}\n  Health: http://{addr}/health\n  Docs:   http://{addr}/docs\n  Agent:  http://{addr}/agent\n"
        );
    }

    pub fn print_json(&self, value: serde_json::Value) {
        if let Ok(s) = serde_json::to_string_pretty(&value) {
            println!("{}", s);
        }
    }

    pub fn print_verbose(&self, msg: &str) {
        if self.verbose && !self.quiet {
            eprintln!("[verbose] {}", msg);
        }
    }

    pub fn print_error(&self, msg: &str) {
        eprintln!("error: {}", msg);
    }
}
