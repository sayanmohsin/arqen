use std::process::Command;

use super::exit;
use super::output::Output;

#[derive(serde::Serialize)]
pub struct CheckResult {
    pub name: String,
    pub status: String,
    pub detail: String,
}

fn run_command(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
}

pub fn run_doctor(output: &Output) -> i32 {
    let mut results: Vec<CheckResult> = Vec::new();
    let mut has_critical_failure = false;

    // 1. rustc
    match run_command("rustc", &["--version"]) {
        Some(v) => results.push(CheckResult {
            name: "rustc".to_string(),
            status: "ok".to_string(),
            detail: v,
        }),
        None => {
            has_critical_failure = true;
            results.push(CheckResult {
                name: "rustc".to_string(),
                status: "fail".to_string(),
                detail: "rustc not found".to_string(),
            });
        }
    }

    // 2. cargo
    match run_command("cargo", &["--version"]) {
        Some(v) => results.push(CheckResult {
            name: "cargo".to_string(),
            status: "ok".to_string(),
            detail: v,
        }),
        None => {
            has_critical_failure = true;
            results.push(CheckResult {
                name: "cargo".to_string(),
                status: "fail".to_string(),
                detail: "cargo not found".to_string(),
            });
        }
    }

    // 3. docker
    match run_command("docker", &["--version"]) {
        Some(v) => results.push(CheckResult {
            name: "docker".to_string(),
            status: "ok".to_string(),
            detail: v,
        }),
        None => results.push(CheckResult {
            name: "docker".to_string(),
            status: "warn".to_string(),
            detail: "docker not found (optional)".to_string(),
        }),
    }

    // 4. docker-compose
    let compose_found = run_command("docker", &["compose", "version"])
        .or_else(|| run_command("docker-compose", &["--version"]));
    match compose_found {
        Some(v) => results.push(CheckResult {
            name: "docker-compose".to_string(),
            status: "ok".to_string(),
            detail: v,
        }),
        None => results.push(CheckResult {
            name: "docker-compose".to_string(),
            status: "warn".to_string(),
            detail: "docker-compose not found (optional)".to_string(),
        }),
    }

    // 5. thingd connectivity
    match std::env::var("ARQEN_THINGD_URL") {
        Ok(url) => results.push(CheckResult {
            name: "thingd".to_string(),
            status: "ok".to_string(),
            detail: format!("ARQEN_THINGD_URL={}", url),
        }),
        Err(_) => results.push(CheckResult {
            name: "thingd".to_string(),
            status: "warn".to_string(),
            detail: "ARQEN_THINGD_URL not set".to_string(),
        }),
    }

    // 6. env vars
    let env_vars = [
        "ARQEN_HOST",
        "ARQEN_PORT",
        "ARQEN_LOG_LEVEL",
        "ARQEN_STORAGE_MODE",
    ];
    let env_detail: Vec<String> = env_vars
        .iter()
        .map(|var| match std::env::var(var) {
            Ok(value) => format!("{}={}", var, value),
            Err(_) => format!("{} (default)", var),
        })
        .collect();
    results.push(CheckResult {
        name: "env".to_string(),
        status: "ok".to_string(),
        detail: env_detail.join(", "),
    });

    if output.is_json() {
        output.print_json(serde_json::to_value(&results).unwrap_or_default());
    } else {
        if !output.is_quiet() {
            output.print("Arqen Doctor - Diagnosing environment...\n");
        }
        for (i, r) in results.iter().enumerate() {
            if output.is_quiet() && r.status == "ok" {
                continue;
            }
            let marker = match r.status.as_str() {
                "ok" => "ok",
                "warn" => "warn",
                "fail" => "FAIL",
                _ => "?",
            };
            output.print(&format!(
                "  {}. [{}] {}: {}",
                i + 1,
                marker,
                r.name,
                r.detail
            ));
        }
        if !output.is_quiet() {
            output.print("\nDoctor complete.");
        }
    }

    if has_critical_failure {
        exit::DEPENDENCY
    } else {
        exit::SUCCESS
    }
}
