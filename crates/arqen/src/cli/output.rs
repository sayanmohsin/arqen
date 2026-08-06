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
