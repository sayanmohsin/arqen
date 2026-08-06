use std::process::Command;

use super::exit;
use super::output::Output;

pub fn run_build(release: bool, output: &Output) -> i32 {
    if !output.is_quiet() {
        output.print("Building...");
    }

    let mut args = vec!["build"];
    if release {
        args.push("--release");
    }

    match Command::new("cargo").args(&args).status() {
        Ok(status) if status.success() => {
            if output.is_json() {
                output.print_json(serde_json::json!({
                    "command": "build",
                    "status": "ok"
                }));
            } else if !output.is_quiet() {
                output.print("Build succeeded");
            }
            exit::SUCCESS
        }
        Ok(status) => {
            let code = status.code().unwrap_or(1);
            if output.is_json() {
                output.print_json(serde_json::json!({
                    "command": "build",
                    "status": "fail",
                    "exit_code": code
                }));
            } else if !output.is_quiet() {
                output.print(&format!("Build failed (exit code {})", code));
            }
            exit::RUNTIME
        }
        Err(e) => {
            if output.is_json() {
                output.print_json(serde_json::json!({
                    "command": "build",
                    "status": "fail",
                    "detail": format!("cargo not found: {}", e)
                }));
            } else {
                output.print(&format!("cargo not found: {}", e));
            }
            exit::DEPENDENCY
        }
    }
}
