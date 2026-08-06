use std::process::Command;

use super::exit;
use super::output::Output;

pub fn run_format(output: &Output) -> i32 {
    if !output.is_quiet() {
        output.print("Formatting code...");
    }

    match Command::new("cargo").args(["fmt", "--all"]).output() {
        Ok(o) if o.status.success() => {
            if output.is_json() {
                output.print_json(serde_json::json!({
                    "command": "format",
                    "status": "ok"
                }));
            } else if !output.is_quiet() {
                output.print("Formatted successfully");
            }
            exit::SUCCESS
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            if output.is_json() {
                output.print_json(serde_json::json!({
                    "command": "format",
                    "status": "fail",
                    "detail": stderr.trim()
                }));
            } else {
                output.print(&format!("Format failed: {}", stderr.trim()));
            }
            exit::RUNTIME
        }
        Err(e) => {
            if output.is_json() {
                output.print_json(serde_json::json!({
                    "command": "format",
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
