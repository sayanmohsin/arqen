use std::process::Command;

use super::exit;
use super::output::Output;

pub fn run_doc(output: &Output) -> i32 {
    if !output.is_quiet() {
        output.print("Generating docs...");
    }

    match Command::new("cargo").args(["doc", "--no-deps"]).status() {
        Ok(status) if status.success() => {
            if output.is_json() {
                output.print_json(serde_json::json!({
                    "command": "doc",
                    "status": "ok"
                }));
            } else if !output.is_quiet() {
                output.print("Docs generated");
            }
            exit::SUCCESS
        }
        Ok(status) => {
            let code = status.code().unwrap_or(1);
            if output.is_json() {
                output.print_json(serde_json::json!({
                    "command": "doc",
                    "status": "fail",
                    "exit_code": code
                }));
            } else if !output.is_quiet() {
                output.print(&format!("Docs failed (exit code {})", code));
            }
            exit::RUNTIME
        }
        Err(e) => {
            if output.is_json() {
                output.print_json(serde_json::json!({
                    "command": "doc",
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
