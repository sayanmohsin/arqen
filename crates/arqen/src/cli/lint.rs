use std::process::Command;

use super::exit;
use super::output::Output;

#[derive(serde::Serialize)]
struct LintResult {
    name: String,
    status: String,
    detail: String,
}

pub fn run_lint(output: &Output) -> i32 {
    let mut results: Vec<LintResult> = Vec::new();
    let mut has_failure = false;

    if !output.is_quiet() {
        output.print("Running lint checks...");
    }

    // 1. cargo fmt --check
    match Command::new("cargo")
        .args(["fmt", "--all", "--", "--check"])
        .output()
    {
        Ok(o) if o.status.success() => {
            results.push(LintResult {
                name: "fmt".to_string(),
                status: "ok".to_string(),
                detail: "formatting OK".to_string(),
            });
        }
        Ok(o) => {
            has_failure = true;
            let stderr = String::from_utf8_lossy(&o.stderr);
            let detail = if stderr.trim().is_empty() {
                "formatting issues found".to_string()
            } else {
                stderr.trim().lines().take(3).collect::<Vec<_>>().join("; ")
            };
            results.push(LintResult {
                name: "fmt".to_string(),
                status: "fail".to_string(),
                detail,
            });
        }
        Err(e) => {
            results.push(LintResult {
                name: "fmt".to_string(),
                status: "fail".to_string(),
                detail: format!("cargo not found: {}", e),
            });
            if output.is_json() {
                output.print_json(serde_json::to_value(&results).unwrap_or_default());
            }
            return exit::DEPENDENCY;
        }
    }

    // 2. cargo clippy
    match Command::new("cargo")
        .args([
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ])
        .output()
    {
        Ok(o) if o.status.success() => {
            results.push(LintResult {
                name: "clippy".to_string(),
                status: "ok".to_string(),
                detail: "no warnings".to_string(),
            });
        }
        Ok(o) => {
            has_failure = true;
            let stderr = String::from_utf8_lossy(&o.stderr);
            let detail = if stderr.trim().is_empty() {
                "warnings found".to_string()
            } else {
                stderr.trim().lines().take(3).collect::<Vec<_>>().join("; ")
            };
            results.push(LintResult {
                name: "clippy".to_string(),
                status: "fail".to_string(),
                detail,
            });
        }
        Err(e) => {
            results.push(LintResult {
                name: "clippy".to_string(),
                status: "fail".to_string(),
                detail: format!("cargo not found: {}", e),
            });
            if output.is_json() {
                output.print_json(serde_json::to_value(&results).unwrap_or_default());
            }
            return exit::DEPENDENCY;
        }
    }

    if output.is_json() {
        output.print_json(serde_json::to_value(&results).unwrap_or_default());
    } else {
        for r in &results {
            if output.is_quiet() && r.status == "ok" {
                continue;
            }
            let marker = match r.status.as_str() {
                "ok" => "ok",
                "fail" => "FAIL",
                _ => "?",
            };
            output.print(&format!("  [{}] {}: {}", marker, r.name, r.detail));
        }
        if !output.is_quiet() {
            if has_failure {
                output.print("Lint failed");
            } else {
                output.print("All checks passed");
            }
        }
    }

    if has_failure {
        exit::RUNTIME
    } else {
        exit::SUCCESS
    }
}
