#![cfg(feature = "cli")]

use std::process::Command;

fn arqen_bin() -> String {
    env!("CARGO_BIN_EXE_arqen").to_string()
}

#[test]
fn version_flag_works() {
    let output = Command::new(arqen_bin()).arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("arqen"));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_flag_works() {
    let output = Command::new(arqen_bin()).arg("--help").output().unwrap();
    assert!(output.status.success());
}

#[test]
fn no_args_shows_help() {
    let bin = arqen_bin();
    let output = Command::new(&bin).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        combined.contains("Usage") || combined.contains("arqen"),
        "stdout: {:?}\nstderr: {:?}",
        stdout,
        stderr
    );
}

#[test]
fn new_refuses_existing_dir() {
    let dir = std::env::temp_dir().join("arqen-test-existing");
    std::fs::create_dir_all(&dir).unwrap();
    let output = Command::new(arqen_bin())
        .arg("new")
        .arg(dir.to_str().unwrap())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn invalid_subcommand_fails_with_usage_code() {
    let output = Command::new(arqen_bin())
        .arg("nonexistent")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn json_output_flag_works() {
    let output = Command::new(arqen_bin())
        .arg("--json")
        .arg("check")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok());
}

#[test]
fn lint_help_works() {
    let output = Command::new(arqen_bin())
        .arg("lint")
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn format_help_works() {
    let output = Command::new(arqen_bin())
        .arg("format")
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn test_help_works() {
    let output = Command::new(arqen_bin())
        .arg("test")
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn build_help_works() {
    let output = Command::new(arqen_bin())
        .arg("build")
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn doc_help_works() {
    let output = Command::new(arqen_bin())
        .arg("doc")
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn new_json_includes_rustfmt_and_clippy_toml() {
    let dir = std::env::temp_dir().join("arqen-test-new-json-lifecycle");
    let _ = std::fs::remove_dir_all(&dir);
    let output = Command::new(arqen_bin())
        .arg("--json")
        .arg("new")
        .arg(dir.to_str().unwrap())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let files = val["files"].as_array().unwrap();
    let file_names: Vec<&str> = files.iter().filter_map(|f| f.as_str()).collect();
    assert!(file_names.contains(&"rustfmt.toml"));
    assert!(file_names.contains(&"clippy.toml"));
    std::fs::remove_dir_all(&dir).unwrap();
}
