//! Optional integration with open-envault and its `oenv` CLI.
use crate::core::{AppError, ErrorKind};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OenvConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_environment")]
    pub environment: String,
    #[serde(default = "default_project_file")]
    pub project_file: PathBuf,
    #[serde(default)]
    pub required: bool,
    #[serde(default = "default_executable")]
    pub executable: PathBuf,
}
fn default_environment() -> String {
    "dev".into()
}
fn default_project_file() -> PathBuf {
    PathBuf::from("open-envault.yaml")
}
fn default_executable() -> PathBuf {
    PathBuf::from("oenv")
}
impl Default for OenvConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            environment: default_environment(),
            project_file: default_project_file(),
            required: false,
            executable: default_executable(),
        }
    }
}
impl OenvConfig {
    pub fn required(environment: impl Into<String>) -> Self {
        Self {
            enabled: true,
            required: true,
            environment: environment.into(),
            ..Self::default()
        }
    }
}

/// Decrypted values held in memory without Debug, Display, or Serialize.
pub struct SecretEnvironment(BTreeMap<String, String>);
impl SecretEnvironment {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0.get(name).map(String::as_str)
    }
    pub fn merge_into(&self, target: &mut std::collections::HashMap<String, String>) {
        target.extend(self.0.iter().map(|(k, v)| (k.clone(), v.clone())));
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct SecretProviderStatus {
    pub available: bool,
    pub environment: String,
    pub required: bool,
}
pub trait SecretProvider: Send + Sync {
    fn load(&self, environment: &str) -> Result<SecretEnvironment, AppError>;
    fn check(&self, environment: &str) -> Result<SecretProviderStatus, AppError>;
}

#[cfg(feature = "oenv")]
fn native_load(environment: &str) -> Result<SecretEnvironment, AppError> {
    open_envault::load_environment(environment)
        .map(SecretEnvironment)
        .map_err(|_| {
            AppError::new(
                ErrorKind::Dependency,
                "open-envault could not load the selected environment",
            )
        })
}
#[derive(Debug, Clone)]
pub struct OenvProvider {
    config: OenvConfig,
}
impl OenvProvider {
    pub fn from_config(config: &OenvConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
    fn command(&self, environment: &str) -> Command {
        let mut command = Command::new(&self.config.executable);
        command
            .args(["exec", environment, "--", "env"])
            .stdin(Stdio::null())
            .stderr(Stdio::piped());
        if self.config.project_file.file_name() == Some(std::ffi::OsStr::new("open-envault.yaml"))
            && let Some(parent) = self.config.project_file.parent()
            && !parent.as_os_str().is_empty()
        {
            command.current_dir(parent);
        }
        command
    }
}
impl SecretProvider for OenvProvider {
    fn load(&self, environment: &str) -> Result<SecretEnvironment, AppError> {
        #[cfg(feature = "oenv")]
        if self.config.project_file.as_path() == Path::new("open-envault.yaml") {
            return native_load(environment);
        }
        let output = self.command(environment).output().map_err(|e| {
            AppError::new(ErrorKind::Dependency, format!("failed to run oenv: {e}"))
        })?;
        if !output.status.success() {
            return Err(AppError::new(
                ErrorKind::Dependency,
                "oenv could not load the selected environment",
            ));
        }
        let text = String::from_utf8(output.stdout).map_err(|_| {
            AppError::new(
                ErrorKind::Dependency,
                "oenv returned invalid environment data",
            )
        })?;
        Ok(SecretEnvironment(
            text.lines()
                .filter_map(|line| {
                    line.split_once('=')
                        .map(|(k, v)| (k.to_owned(), v.to_owned()))
                })
                .collect(),
        ))
    }
    fn check(&self, environment: &str) -> Result<SecretProviderStatus, AppError> {
        let status = Command::new(&self.config.executable)
            .arg("--version")
            .output()
            .map_err(|e| {
                AppError::new(ErrorKind::Dependency, format!("oenv is unavailable: {e}"))
            })?;
        if !status.status.success() {
            return Err(AppError::new(
                ErrorKind::Dependency,
                "oenv version check failed",
            ));
        }
        Ok(SecretProviderStatus {
            available: true,
            environment: environment.into(),
            required: self.config.required,
        })
    }
}
pub(crate) fn load_configured(config: &OenvConfig) -> Result<Option<SecretEnvironment>, AppError> {
    if !config.enabled {
        return Ok(None);
    }
    match OenvProvider::from_config(config).load(&config.environment) {
        Ok(values) => Ok(Some(values)),
        Err(error) if config.required => Err(error),
        Err(_) => Ok(None),
    }
}
