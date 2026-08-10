use std::io::IsTerminal;

use tracing_subscriber::{EnvFilter, fmt};

use crate::config::{LogFormat, LoggingConfig};

pub fn init_logging(log_level: &str, log_format: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));
    let ansi = std::io::stderr().is_terminal();

    match log_format {
        "json" => {
            fmt()
                .with_env_filter(filter)
                .with_target(false)
                .with_ansi(false)
                .json()
                .init();
        }
        "pretty" => {
            fmt()
                .with_env_filter(filter)
                .with_target(false)
                .with_ansi(ansi)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_timer(fmt::time::SystemTime)
                .pretty()
                .init();
        }
        _ => {
            fmt()
                .with_env_filter(filter)
                .with_target(false)
                .with_ansi(ansi)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_timer(fmt::time::SystemTime)
                .compact()
                .init();
        }
    }
}

pub fn init_logging_with_config(config: &LoggingConfig) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));
    let ansi = std::io::stderr().is_terminal();

    match config.format {
        LogFormat::Json => {
            fmt()
                .with_env_filter(filter)
                .with_target(false)
                .with_ansi(false)
                .json()
                .init();
        }
        LogFormat::Pretty => {
            fmt()
                .with_env_filter(filter)
                .with_target(false)
                .with_ansi(ansi)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_timer(fmt::time::SystemTime)
                .pretty()
                .init();
        }
        LogFormat::Compact => {
            fmt()
                .with_env_filter(filter)
                .with_target(false)
                .with_ansi(ansi)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_timer(fmt::time::SystemTime)
                .compact()
                .init();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LogFormat;

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert!(matches!(config.format, LogFormat::Pretty));
    }

    #[test]
    fn test_logging_config_custom() {
        let config = LoggingConfig {
            level: "debug".to_string(),
            format: LogFormat::Json,
        };
        assert_eq!(config.level, "debug");
        assert!(matches!(config.format, LogFormat::Json));
    }
}
