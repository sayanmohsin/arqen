use std::path::Path;
use std::process::Command;

use super::exit;
use super::output::Output;

#[derive(serde::Serialize)]
pub struct CheckResult {
    pub name: String,
    pub status: String,
    pub detail: String,
}

pub fn run_check(output: &Output) -> i32 {
    let mut results: Vec<CheckResult> = Vec::new();
    let mut has_critical_failure = false;
    let mut config_invalid = false;

    // 1. rustc
    match Command::new("rustc").arg("--version").output() {
        Ok(o) if o.status.success() => {
            let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
            results.push(CheckResult {
                name: "rustc".to_string(),
                status: "ok".to_string(),
                detail: v,
            });
        }
        _ => {
            has_critical_failure = true;
            results.push(CheckResult {
                name: "rustc".to_string(),
                status: "fail".to_string(),
                detail: "rustc not found".to_string(),
            });
        }
    }

    // 2. cargo
    match Command::new("cargo").arg("--version").output() {
        Ok(o) if o.status.success() => {
            let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
            results.push(CheckResult {
                name: "cargo".to_string(),
                status: "ok".to_string(),
                detail: v,
            });
        }
        _ => {
            has_critical_failure = true;
            results.push(CheckResult {
                name: "cargo".to_string(),
                status: "fail".to_string(),
                detail: "cargo not found".to_string(),
            });
        }
    }

    // 3. project structure
    if Path::new("Cargo.toml").exists() {
        results.push(CheckResult {
            name: "project".to_string(),
            status: "ok".to_string(),
            detail: "Cargo.toml found".to_string(),
        });
    } else {
        results.push(CheckResult {
            name: "project".to_string(),
            status: "warn".to_string(),
            detail: "Cargo.toml not found in current directory".to_string(),
        });
    }

    // 4. config discovery
    let config_path =
        std::env::var("ARQEN_CONFIG_FILE").unwrap_or_else(|_| "arqen.toml".to_string());
    if Path::new(&config_path).exists() {
        match crate::config::AppConfig::from_file(&config_path) {
            Ok(_) => results.push(CheckResult {
                name: "config".to_string(),
                status: "ok".to_string(),
                detail: format!("{} is valid", config_path),
            }),
            Err(e) => {
                config_invalid = true;
                results.push(CheckResult {
                    name: "config".to_string(),
                    status: "fail".to_string(),
                    detail: format!("{}: {}", config_path, e),
                });
            }
        }
    } else {
        results.push(CheckResult {
            name: "config".to_string(),
            status: "ok".to_string(),
            detail: format!("{} not found (using defaults)", config_path),
        });
    }

    if output.is_json() {
        output.print_json(serde_json::to_value(&results).unwrap_or_default());
    } else {
        if !output.is_quiet() {
            output.print("Running checks...");
        }
        for r in &results {
            if output.is_quiet() && r.status == "ok" {
                continue;
            }
            let marker = match r.status.as_str() {
                "ok" => "ok",
                "warn" => "warn",
                "fail" => "FAIL",
                _ => "?",
            };
            output.print(&format!("  [{}] {}: {}", marker, r.name, r.detail));
        }
        if !output.is_quiet() {
            output.print("Checks passed");
        }
    }

    if has_critical_failure {
        exit::DEPENDENCY
    } else if config_invalid {
        exit::CONFIG
    } else {
        exit::SUCCESS
    }
}
