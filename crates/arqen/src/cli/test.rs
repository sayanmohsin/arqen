use std::process::Command;

use super::exit;
use super::output::Output;

pub fn run_test(release: bool, output: &Output) -> i32 {
    if !output.is_quiet() {
        output.print("Running tests...");
    }

    let mut args = vec!["test", "--all-features"];
    if release {
        args.push("--release");
    }

    match Command::new("cargo").args(&args).status() {
        Ok(status) if status.success() => {
            if output.is_json() {
                output.print_json(serde_json::json!({
                    "command": "test",
                    "status": "ok"
                }));
            } else if !output.is_quiet() {
                output.print("All tests passed");
            }
            exit::SUCCESS
        }
        Ok(status) => {
            let code = status.code().unwrap_or(1);
            if output.is_json() {
                output.print_json(serde_json::json!({
                    "command": "test",
                    "status": "fail",
                    "exit_code": code
                }));
            } else if !output.is_quiet() {
                output.print(&format!("Tests failed (exit code {})", code));
            }
            exit::RUNTIME
        }
        Err(e) => {
            if output.is_json() {
                output.print_json(serde_json::json!({
                    "command": "test",
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
