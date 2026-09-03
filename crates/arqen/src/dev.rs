//! Local development process supervision.
//!
//! [`run_up`] starts a set of long-running dev services (a database sidecar,
//! a backend, a frontend) defined in the `[[dev.services]]` sections of an
//! `arqen.toml`, forwards their output with a `[name]` prefix, and stops
//! everything when Ctrl+C is pressed or any service exits.
//!
//! ```toml
//! [[dev.services]]
//! name = "thingd"
//! command = "docker"
//! args = ["compose", "up"]
//! cwd = "."
//!
//! [[dev.services]]
//! name = "backend"
//! command = "cargo"
//! args = ["run"]
//! cwd = "backend"
//! ```

use std::collections::HashMap;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, watch};
use tokio::time::sleep;

/// How long to wait for a service to exit before killing it.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// Loaded `[[dev.services]]` configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DevConfig {
    #[serde(default)]
    pub dev: DevSection,
}

/// The `[dev]` table from an `arqen.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DevSection {
    #[serde(default)]
    pub services: Vec<DevService>,
}

/// A single dev service definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevService {
    /// Unique name used for logging and selection.
    pub name: String,
    /// Executable to run.
    pub command: String,
    /// Arguments passed to the executable.
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory for the process (defaults to the current directory).
    #[serde(default)]
    pub cwd: Option<PathBuf>,
    /// Extra environment variables for the process.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Optional HTTP URL used by `arqen up --wait-ready`.
    #[serde(default)]
    pub ready_url: Option<String>,
    /// Maximum seconds to wait for `ready_url` (defaults to 60).
    #[serde(default)]
    pub ready_timeout_seconds: Option<u64>,
}

/// Load dev services from a TOML file. Unknown tables (for example `[server]`)
/// are ignored, so the file can double as the application's `arqen.toml`.
pub fn load(path: &Path) -> anyhow::Result<DevConfig> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("failed to read dev config '{}': {}", path.display(), e))?;
    toml::from_str(&text)
        .map_err(|e| anyhow::anyhow!("failed to parse dev config '{}': {}", path.display(), e))
}

/// Supervise the dev services in `path`, optionally restricted to `selection`.
///
/// If `dry_run` is set, prints the plan and returns without starting anything.
pub async fn run_up(
    path: &Path,
    selection: &[String],
    dry_run: bool,
    raw: bool,
    wait_ready: bool,
) -> anyhow::Result<()> {
    let config = load(path)?;

    let services = select_services(&config, selection)?;
    if services.is_empty() {
        anyhow::bail!("no [[dev.services]] found in '{}'", path.display());
    }

    let console = Console::new();
    console.header(services.len());
    for service in &services {
        let args = service.args.join(" ");
        let cwd = service
            .cwd
            .as_deref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| ".".to_string());
        console.plan(&service.name, &service.command, &args, &cwd);
    }
    console.footer();

    if dry_run {
        return Ok(());
    }

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let (exit_tx, mut exit_rx) = mpsc::channel::<ExitInfo>(services.len());

    let mut spawned = 0usize;
    let mut spawn_error = None;
    let readiness = services
        .iter()
        .filter_map(|service| {
            service.ready_url.as_ref().map(|url| {
                (
                    service.name.clone(),
                    url.clone(),
                    service.ready_timeout_seconds.unwrap_or(60),
                )
            })
        })
        .collect::<Vec<_>>();

    for service in services {
        let mut cmd = Command::new(&service.command);
        cmd.args(&service.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if let Some(cwd) = &service.cwd {
            cmd.current_dir(cwd);
        }
        for (key, value) in &service.env {
            cmd.env(key, value);
        }

        match cmd.spawn() {
            Ok(child) => {
                spawned += 1;
                let name = service.name.clone();
                let rx = shutdown_rx.clone();
                let tx = exit_tx.clone();
                tokio::spawn(async move {
                    supervise(&name, child, rx, tx, raw).await;
                });
            }
            Err(e) => {
                spawn_error = Some(anyhow::anyhow!("failed to start '{}': {}", service.name, e));
                break;
            }
        }
    }

    drop(exit_tx);
    drop(shutdown_rx);

    if wait_ready && let Err(error) = wait_for_readiness(&readiness).await {
        let _ = shutdown_tx.send(true);
        drain(&mut exit_rx).await;
        return Err(error);
    }

    if let Some(err) = spawn_error {
        if spawned > 0 {
            let _ = shutdown_tx.send(true);
            drain(&mut exit_rx).await;
        }
        return Err(err);
    }

    let mut saw_shutdown = false;
    let mut failure: Option<String> = None;
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                console.info("stopping services");
                if !saw_shutdown {
                    saw_shutdown = true;
                    let _ = shutdown_tx.send(true);
                }
            }
            info = exit_rx.recv() => {
                let Some(info) = info else { break };
                report_exit(&info);
                let exited_on_its_own = !saw_shutdown;
                if exited_on_its_own {
                    saw_shutdown = true;
                    console.warn(&format!("{} stopped; shutting down the rest", info.name));
                    let _ = shutdown_tx.send(true);
                }
                if exited_on_its_own && info.status.is_none_or(|status| !status.success()) {
                    failure = Some(info.name);
                }
            }
        }
    }
    drain(&mut exit_rx).await;

    match failure {
        Some(name) => Err(anyhow::anyhow!(
            "dev service '{}' exited with an error",
            name
        )),
        None => Ok(()),
    }
}

fn select_services<'a>(
    config: &'a DevConfig,
    selection: &[String],
) -> anyhow::Result<Vec<&'a DevService>> {
    if selection.is_empty() {
        return Ok(config.dev.services.iter().collect());
    }
    selection
        .iter()
        .map(|name| {
            config
                .dev
                .services
                .iter()
                .find(|s| &s.name == name)
                .ok_or_else(|| anyhow::anyhow!("unknown dev service '{}'", name))
        })
        .collect()
}

fn report_exit(info: &ExitInfo) {
    let console = Console::new();
    match info.status.and_then(|s| s.code()) {
        Some(0) => console.success(&format!("{} exited cleanly", info.name)),
        Some(code) => console.error(&format!("{} exited with code {}", info.name, code)),
        None => console.error(&format!("{} terminated by signal", info.name)),
    }
}

/// How a service finished.
#[derive(Debug)]
struct ExitInfo {
    name: String,
    status: Option<ExitStatus>,
}

async fn drain(rx: &mut mpsc::Receiver<ExitInfo>) {
    while rx.recv().await.is_some() {}
}

async fn supervise(
    name: &str,
    mut child: Child,
    mut shutdown: watch::Receiver<bool>,
    exit_tx: mpsc::Sender<ExitInfo>,
    raw: bool,
) {
    let prefix = name.to_string();
    if let Some(stdout) = child.stdout.take() {
        let prefix = prefix.clone();
        tokio::spawn(async move {
            forward_output(&prefix, stdout, raw).await;
        });
    }
    if let Some(stderr) = child.stderr.take() {
        let prefix = prefix.clone();
        tokio::spawn(async move {
            forward_output(&prefix, stderr, raw).await;
        });
    }

    let status = tokio::select! {
        _ = shutdown.changed() => {
            request_stop(&mut child).await;
            tokio::select! {
                status = child.wait() => status.ok(),
                _ = sleep(SHUTDOWN_TIMEOUT) => {
                    let _ = child.kill().await;
                    child.wait().await.ok()
                }
            }
        }
        status = child.wait() => status.ok(),
    };

    let _ = exit_tx
        .send(ExitInfo {
            name: name.to_string(),
            status,
        })
        .await;
}

async fn forward_output(prefix: &str, stream: impl AsyncRead + Unpin, raw: bool) {
    let mut lines = BufReader::new(stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if raw {
            println!("{line}");
        } else {
            Console::new().child_line(prefix, &line);
        }
    }
}

async fn wait_for_readiness(readiness: &[(String, String, u64)]) -> anyhow::Result<()> {
    for (name, url, timeout_seconds) in readiness {
        let deadline = tokio::time::Instant::now() + Duration::from_secs((*timeout_seconds).max(1));
        loop {
            match reqwest::get(url).await {
                Ok(response) if response.status().is_success() => {
                    Console::new().success(&format!("{name} ready ({url})"));
                    break;
                }
                _ if tokio::time::Instant::now() >= deadline => {
                    anyhow::bail!("service '{name}' did not become ready at {url}");
                }
                _ => sleep(Duration::from_millis(250)).await,
            }
        }
    }
    Ok(())
}

struct Console {
    color: bool,
}

impl Console {
    fn new() -> Self {
        Self {
            color: std::io::stdout().is_terminal(),
        }
    }

    fn header(&self, count: usize) {
        println!(
            "{} arqen dev {}· {} service{}",
            self.paint("◆", 36),
            self.dim(""),
            count,
            if count == 1 { "" } else { "s" }
        );
    }

    fn plan(&self, name: &str, command: &str, args: &str, cwd: &str) {
        let command_line = if args.is_empty() {
            command.to_string()
        } else {
            format!("{command} {args}")
        };
        println!(
            "  {} {:<12} {} {}",
            self.paint("│", 90),
            self.service(name),
            command_line,
            self.dim(&format!("· {cwd}"))
        );
    }

    fn footer(&self) {
        println!("  {} {}", self.paint("└", 90), self.dim("Ctrl+C to stop"));
    }

    fn child_line(&self, name: &str, line: &str) {
        println!("{} {} {}", self.service(name), self.paint("│", 90), line);
    }

    fn info(&self, message: &str) {
        println!("{} {}", self.paint("ℹ", 36), message);
    }

    fn success(&self, message: &str) {
        println!("{} {}", self.paint("✓", 32), message);
    }

    fn warn(&self, message: &str) {
        println!("{} {}", self.paint("!", 33), message);
    }

    fn error(&self, message: &str) {
        println!("{} {}", self.paint("×", 31), message);
    }

    fn service(&self, name: &str) -> String {
        let color = [36, 35, 33, 32, 34][name.bytes().map(usize::from).sum::<usize>() % 5];
        self.paint(&format!("{name:<12}"), color)
    }

    fn dim(&self, text: &str) -> String {
        if self.color {
            format!("\x1b[2m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    fn paint(&self, text: &str, color: u8) -> String {
        if self.color {
            format!("\x1b[{}m{text}\x1b[0m", color)
        } else {
            text.to_string()
        }
    }
}

#[cfg(unix)]
async fn request_stop(child: &mut Child) {
    let Some(pid) = child.id() else {
        return;
    };
    // Safety: `pid` comes from the OS for a process we spawned.
    unsafe {
        libc::kill(pid as libc::pid_t, libc::SIGINT);
    }
}

#[cfg(not(unix))]
async fn request_stop(child: &mut Child) {
    let _ = child.kill().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    fn write_temp_config(toml_text: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "arqen-dev-test-{}-{}.toml",
            std::process::id(),
            COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::write(&path, toml_text).unwrap();
        path
    }

    #[test]
    fn parses_dev_services() {
        let text = r#"
[server]
port = 3000

[[dev.services]]
name = "backend"
command = "cargo"
args = ["run"]
cwd = "backend"
env = { ARQEN_PORT = "3000" }

[[dev.services]]
name = "frontend"
command = "pnpm"
args = ["dev"]
"#;
        let config: DevConfig = toml::from_str(text).unwrap();
        assert_eq!(config.dev.services.len(), 2);
        let backend = &config.dev.services[0];
        assert_eq!(backend.name, "backend");
        assert_eq!(backend.command, "cargo");
        assert_eq!(backend.args, vec!["run"]);
        assert_eq!(backend.cwd.as_deref(), Some(Path::new("backend")));
        assert_eq!(
            backend.env.get("ARQEN_PORT").map(String::as_str),
            Some("3000")
        );
        assert_eq!(config.dev.services[1].cwd, None);
    }

    #[test]
    fn rejects_unknown_selection() {
        let config = DevConfig {
            dev: DevSection {
                services: vec![DevService {
                    name: "backend".into(),
                    command: "true".into(),
                    args: vec![],
                    cwd: None,
                    env: Default::default(),
                    ready_url: None,
                    ready_timeout_seconds: None,
                }],
            },
        };
        let err = select_services(&config, &["nope".to_string()]).unwrap_err();
        assert!(err.to_string().contains("unknown dev service"));
    }

    #[tokio::test]
    async fn dry_run_does_not_spawn() {
        let path = write_temp_config(
            r#"[[dev.services]]
name = "quick"
command = "false"
"#,
        );
        run_up(&path, &[], true, false, false).await.unwrap();
        std::fs::remove_file(&path).unwrap();
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn successful_service_stops_the_rest() {
        let path = write_temp_config(
            r#"[[dev.services]]
name = "quick"
command = "sh"
args = ["-c", "exit 0"]

[[dev.services]]
name = "slow"
command = "sleep"
args = ["30"]
"#,
        );
        let result = tokio::time::timeout(
            Duration::from_secs(15),
            run_up(&path, &[], false, false, false),
        )
        .await
        .expect("run_up should finish promptly");
        result.unwrap();
        std::fs::remove_file(&path).unwrap();
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn failing_service_returns_error() {
        let path = write_temp_config(
            r#"[[dev.services]]
name = "quick"
command = "sh"
args = ["-c", "exit 3"]

[[dev.services]]
name = "slow"
command = "sleep"
args = ["30"]
"#,
        );
        let result = tokio::time::timeout(
            Duration::from_secs(15),
            run_up(&path, &[], false, false, false),
        )
        .await
        .expect("run_up should finish promptly");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("quick"));
        std::fs::remove_file(&path).unwrap();
    }
}
