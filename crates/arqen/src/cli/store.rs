//! Native store export/import helpers for deployment workflows.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::{exit, output::Output};

fn configured_data_dir(explicit: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(path) = explicit {
        return Ok(path.to_path_buf());
    }
    let config = crate::config::AppConfig::load(crate::config::CliOverrides::default())
        .map_err(|error| error.to_string())?;
    config
        .storage
        .persistent_path
        .ok_or_else(|| "native store path is not configured; pass --data-dir".to_string())
}

pub fn export(data_dir: Option<&Path>, output_path: &Path, output: &Output) -> i32 {
    let data_dir = match configured_data_dir(data_dir) {
        Ok(path) => path,
        Err(error) => {
            output.print_error(&error);
            return exit::CONFIG;
        }
    };
    if !data_dir.is_dir() {
        output.print_error(&format!(
            "native store directory does not exist: {}",
            data_dir.display()
        ));
        return exit::RUNTIME;
    }
    if let Some(parent) = output_path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        output.print_error(&error.to_string());
        return exit::RUNTIME;
    }

    let output_path =
        std::fs::canonicalize(output_path).unwrap_or_else(|_| output_path.to_path_buf());
    let status = Command::new("tar")
        .args(["--exclude=._*", "--exclude=.DS_Store", "-czf"])
        .arg(&output_path)
        .arg("-C")
        .arg(&data_dir)
        .arg(".")
        .status();
    match status {
        Ok(status) if status.success() => {
            output.print(&format!(
                "exported {} to {}",
                data_dir.display(),
                output_path.display()
            ));
            exit::SUCCESS
        }
        Ok(status) => {
            output.print_error(&format!("tar export failed with status {status}"));
            exit::RUNTIME
        }
        Err(error) => {
            output.print_error(&format!("failed to run tar: {error}"));
            exit::RUNTIME
        }
    }
}

pub fn import(data_dir: Option<&Path>, input_path: &Path, output: &Output) -> i32 {
    let data_dir = match configured_data_dir(data_dir) {
        Ok(path) => path,
        Err(error) => {
            output.print_error(&error);
            return exit::CONFIG;
        }
    };
    if !input_path.is_file() {
        output.print_error(&format!(
            "backup archive does not exist: {}",
            input_path.display()
        ));
        return exit::CONFIG;
    }
    if let Err(error) = std::fs::create_dir_all(&data_dir) {
        output.print_error(&error.to_string());
        return exit::RUNTIME;
    }
    let status = Command::new("tar")
        .args([
            "--no-same-owner",
            "--exclude=._*",
            "--exclude=.DS_Store",
            "-xzf",
        ])
        .arg(input_path)
        .arg("-C")
        .arg(&data_dir)
        .status();
    match status {
        Ok(status) if status.success() => {
            output.print(&format!(
                "imported {} into {}",
                input_path.display(),
                data_dir.display()
            ));
            exit::SUCCESS
        }
        Ok(status) => {
            output.print_error(&format!("tar import failed with status {status}"));
            exit::RUNTIME
        }
        Err(error) => {
            output.print_error(&format!("failed to run tar: {error}"));
            exit::RUNTIME
        }
    }
}
