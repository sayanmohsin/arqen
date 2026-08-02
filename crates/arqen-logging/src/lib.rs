use tracing_subscriber::{EnvFilter, fmt};

pub fn init_logging(log_level: &str, log_format: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    match log_format {
        "json" => {
            fmt().with_env_filter(filter).json().init();
        }
        "pretty" => {
            fmt().with_env_filter(filter).pretty().init();
        }
        _ => {
            fmt().with_env_filter(filter).init();
        }
    }
}
