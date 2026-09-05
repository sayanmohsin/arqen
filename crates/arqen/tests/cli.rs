#![cfg(feature = "cli")]

use std::process::Command;

fn arqen_bin() -> String {
    env!("CARGO_BIN_EXE_arqen").to_string()
}

#[test]
#[cfg(unix)]
fn watch_is_optional_and_change_details_are_verbose_only() {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("arqen-watch-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname='fixture'\nversion='0.1.0'\n[dependencies]\narqen = '0.18'\n",
    )
    .unwrap();
    let cargo = dir.join("cargo");
    std::fs::write(&cargo, "#!/bin/sh\nif [ \"$2\" = --version ]; then exit \"${WATCH_MISSING:-0}\"; fi\nprintf '%s\\n' \"$@\"\n").unwrap();
    std::fs::set_permissions(&cargo, std::fs::Permissions::from_mode(0o755)).unwrap();
    for verbose in [false, true] {
        let mut command = Command::new(arqen_bin());
        command
            .current_dir(&dir)
            .env("PATH", &dir)
            .args(["dev", "--watch"]);
        if verbose {
            command.arg("--verbose");
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).contains("--why"),
            verbose
        );
    }
    let output = Command::new(arqen_bin())
        .current_dir(&dir)
        .env("PATH", &dir)
        .env("WATCH_MISSING", "1")
        .args(["dev", "--watch"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let message = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(message.contains("cargo install cargo-watch"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(unix)]
fn cancellation_during_readiness_stops_service() {
    use std::time::{Duration, Instant};
    let dir = std::env::temp_dir().join(format!("arqen-cancel-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let pid_file = dir.join("pid");
    let config = dir.join("arqen.toml");
    std::fs::write(
        &config,
        format!(
            r#"[[dev.services]]
name = "waiting"
command = "sh"
args = ["-c", "echo $$ > '{}'; exec sleep 60"]
ready_url = "http://127.0.0.1:1/ready"
ready_timeout_seconds = 60
"#,
            pid_file.display()
        ),
    )
    .unwrap();
    let mut child = Command::new(arqen_bin())
        .args(["up", "--wait-ready", "--file"])
        .arg(&config)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !pid_file.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(25));
    }
    if !pid_file.exists() {
        let _ = child.kill();
        panic!("service did not start");
    }
    let pid: i32 = std::fs::read_to_string(&pid_file)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    // Safety: signal only the supervisor spawned by this test.
    unsafe {
        libc::kill(child.id() as i32, libc::SIGINT);
    }
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            unsafe {
                libc::kill(-pid, libc::SIGKILL);
            }
            panic!("supervisor ignored cancellation during readiness");
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    assert_eq!(
        unsafe { libc::kill(pid, 0) },
        -1,
        "service survived shutdown"
    );
    std::fs::remove_dir_all(dir).unwrap();
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
fn migration_command_requires_explicit_feature() {
    let output = Command::new(arqen_bin())
        .args(["thingd", "migrate", "--help"])
        .output()
        .unwrap();
    assert_eq!(output.status.success(), cfg!(feature = "thingd-migration"));
}

#[test]
fn help_flag_works() {
    let output = Command::new(arqen_bin()).arg("--help").output().unwrap();
    assert!(output.status.success());
}

#[test]
fn dev_and_up_expose_runtime_neutral_operations() {
    let dev = Command::new(arqen_bin())
        .args(["dev", "--help"])
        .output()
        .unwrap();
    assert!(dev.status.success());
    let dev_help = String::from_utf8_lossy(&dev.stdout);
    assert!(dev_help.contains("--watch"));

    let up = Command::new(arqen_bin())
        .args(["up", "--help"])
        .output()
        .unwrap();
    assert!(up.status.success());
    let up_help = String::from_utf8_lossy(&up.stdout);
    assert!(up_help.contains("--raw"));
    assert!(up_help.contains("--wait-ready"));
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
fn thingd_seed_help_works() {
    let output = Command::new(arqen_bin())
        .args(["thingd", "seed", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("bounded startup retries"));
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

#[test]
fn new_yes_generates_current_minimal_project() {
    let dir = std::env::temp_dir().join(format!("arqen-test-new-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let output = Command::new(arqen_bin())
        .args(["new", dir.to_str().unwrap(), "--yes"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let cargo = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
    assert!(cargo.contains(&format!("version = \"{}\"", env!("CARGO_PKG_VERSION"))));
    assert!(cargo.contains("http-server"));
    assert!(cargo.contains("logging"));
    assert!(!cargo.contains("thingd-native"));
    assert!(dir.join("AGENTS.md").exists());
    assert!(dir.join("arqen.toml").exists());
    assert!(dir.join("src/app/mod.rs").exists());
    assert!(!cargo.contains("tokio"));
    assert!(
        std::fs::read_to_string(dir.join("README.md"))
            .unwrap()
            .contains("arqen dev")
    );
    assert!(
        std::fs::read_to_string(dir.join("src/main.rs"))
            .unwrap()
            .contains("arqen::run")
    );
    assert!(!dir.join("NICE_CODE.md").exists());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn new_optional_flags_generate_optional_files() {
    let dir = std::env::temp_dir().join(format!("arqen-test-new-options-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let output = Command::new(arqen_bin())
        .args([
            "new",
            dir.to_str().unwrap(),
            "--yes",
            "--thingd",
            "--examples",
            "--nice-code",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let cargo = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
    assert!(cargo.contains("thingd-native"));
    assert!(
        std::fs::read_to_string(dir.join("arqen.toml"))
            .unwrap()
            .contains("mode = \"native\"")
    );
    assert!(dir.join("examples/README.md").exists());
    assert!(dir.join("NICE_CODE.md").exists());
    assert!(dir.join(".github/workflows/nice-code.yml").exists());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn new_no_http_generates_non_http_app() {
    let dir = std::env::temp_dir().join(format!("arqen-test-new-no-http-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let output = Command::new(arqen_bin())
        .args([
            "new",
            dir.to_str().unwrap(),
            "--yes",
            "--no-http",
            "--no-logging",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let cargo = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
    assert!(!cargo.contains("http-server"));
    assert!(!cargo.contains("logging"));
    assert!(!dir.join("src/app/mod.rs").exists());
    std::fs::remove_dir_all(&dir).unwrap();
}
