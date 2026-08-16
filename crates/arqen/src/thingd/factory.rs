//! Runtime construction of Arqen storage backends.

use std::sync::Arc;

use crate::config::{AppConfig, ConfigError, StorageMode};
use crate::thingd::ThingdBackend;

/// Builds the configured thingd backend for an application.
pub struct StorageFactory;

impl StorageFactory {
    /// Construct the backend selected by `config.storage.mode`.
    pub fn build(config: &AppConfig) -> Result<Arc<dyn ThingdBackend>, ConfigError> {
        let backend = match config.storage.mode {
            StorageMode::Memory => {
                Arc::new(crate::thingd::MemoryThingdBackend::new()) as Arc<dyn ThingdBackend>
            }
            StorageMode::Native | StorageMode::Persistent => {
                #[cfg(feature = "thingd-native")]
                {
                    warn_if_low_native_memory();
                    let path = config.storage.persistent_path.as_ref().ok_or_else(|| {
                        ConfigError::MissingField {
                            field: "persistent_path".to_string(),
                            context: "required for native storage".to_string(),
                        }
                    })?;
                    let options = if let Some(key) = &config.storage.encryption_key {
                        let bytes = hex::decode(key.inner()).map_err(|error| {
                            ConfigError::InvalidValue {
                                field: "storage.encryption_key".to_string(),
                                value: "[REDACTED]".to_string(),
                                expected: format!("64 hexadecimal characters: {error}"),
                            }
                        })?;
                        let encryption =
                            thingd::EncryptionConfig::from_key(&bytes).map_err(|error| {
                                ConfigError::InvalidValue {
                                    field: "storage.encryption_key".to_string(),
                                    value: "[REDACTED]".to_string(),
                                    expected: error.to_string(),
                                }
                            })?;
                        thingd::PersistentOpenOptions {
                            encryption: Some(encryption),
                            ..Default::default()
                        }
                    } else {
                        thingd::PersistentOpenOptions::default()
                    };
                    let backend =
                        crate::thingd::NativeThingdBackend::persistent_with_options(path, options)
                            .map_err(|error| ConfigError::InvalidValue {
                                field: "persistent_path".to_string(),
                                value: path.display().to_string(),
                                expected: error.to_string(),
                            })?;
                    Arc::new(backend)
                }
                #[cfg(not(feature = "thingd-native"))]
                {
                    return Err(ConfigError::InvalidValue {
                        field: "storage.mode".to_string(),
                        value: "native".to_string(),
                        expected: "Arqen built with the thingd-native feature".to_string(),
                    });
                }
            }
            StorageMode::Http => {
                #[cfg(feature = "http-client")]
                {
                    let url = config.storage.http_url.as_deref().ok_or_else(|| {
                        ConfigError::MissingField {
                            field: "http_url".to_string(),
                            context: "required for HTTP storage".to_string(),
                        }
                    })?;
                    let mut policy = crate::thingd::HttpClientPolicy::default();
                    let mut max_concurrency = 16usize;
                    if let Ok(value) = std::env::var("ARQEN_THINGD_MAX_CONCURRENCY") {
                        max_concurrency = value.parse().map_err(|_| ConfigError::InvalidValue {
                            field: "thingd.max_concurrency".to_string(),
                            value,
                            expected: "a positive usize".to_string(),
                        })?;
                        if max_concurrency == 0 {
                            return Err(ConfigError::InvalidValue {
                                field: "thingd.max_concurrency".to_string(),
                                value: "0".to_string(),
                                expected: "a positive usize".to_string(),
                            });
                        }
                    }
                    if let Ok(value) = std::env::var("ARQEN_THINGD_REQUEST_TIMEOUT") {
                        policy.request_timeout =
                            std::time::Duration::from_secs(value.parse().map_err(|_| {
                                ConfigError::InvalidValue {
                                    field: "thingd.request_timeout".to_string(),
                                    value,
                                    expected: "a valid u64 (seconds)".to_string(),
                                }
                            })?);
                    }
                    if let Ok(value) = std::env::var("ARQEN_THINGD_MAX_RETRIES") {
                        policy.max_retries =
                            value.parse().map_err(|_| ConfigError::InvalidValue {
                                field: "thingd.max_retries".to_string(),
                                value,
                                expected: "a valid u32".to_string(),
                            })?;
                    }
                    if let Ok(value) = std::env::var("ARQEN_THINGD_MAX_RETRY_DURATION") {
                        policy.max_retry_duration =
                            std::time::Duration::from_secs(value.parse().map_err(|_| {
                                ConfigError::InvalidValue {
                                    field: "thingd.max_retry_duration".to_string(),
                                    value,
                                    expected: "a valid u64 (seconds)".to_string(),
                                }
                            })?);
                    }
                    if let Ok(value) = std::env::var("ARQEN_THINGD_MAX_QUERY_SCAN_OBJECTS") {
                        policy.max_query_scan_objects =
                            value.parse().map_err(|_| ConfigError::InvalidValue {
                                field: "thingd.max_query_scan_objects".to_string(),
                                value,
                                expected: "a positive usize".to_string(),
                            })?;
                        if policy.max_query_scan_objects == 0 {
                            return Err(ConfigError::InvalidValue {
                                field: "thingd.max_query_scan_objects".to_string(),
                                value: "0".to_string(),
                                expected: "a positive usize".to_string(),
                            });
                        }
                    }
                    let mut backend = crate::thingd::HttpThingdBackend::with_policy(url, policy)
                        .with_max_concurrency(max_concurrency);
                    if let Some(token) = &config.storage.auth_token {
                        backend = backend.with_auth(token.inner());
                    }
                    Arc::new(backend)
                }
                #[cfg(not(feature = "http-client"))]
                {
                    return Err(ConfigError::InvalidValue {
                        field: "storage.mode".to_string(),
                        value: "http".to_string(),
                        expected: "Arqen built with the http-client feature".to_string(),
                    });
                }
            }
            StorageMode::Cloud => {
                return Err(ConfigError::InvalidValue {
                    field: "storage.mode".to_string(),
                    value: "cloud".to_string(),
                    expected: "a future public thingd-cloud adapter".to_string(),
                });
            }
        };

        if config.storage.cache_enabled {
            let cache: Arc<dyn ThingdBackend> = Arc::new(crate::thingd::MemoryThingdBackend::new());
            Ok(Arc::new(crate::thingd::CachingThingdBackend::new_catalog(
                backend,
                cache,
                crate::thingd::CachePolicy::default(),
                config.storage.cache_collections.clone(),
            )))
        } else {
            Ok(backend)
        }
    }
}

#[cfg(target_os = "linux")]
fn warn_if_low_native_memory() {
    let total_kb = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|contents| {
            contents
                .lines()
                .find(|line| line.starts_with("MemTotal:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|value| value.parse::<u64>().ok())
        });

    if let Some(total_kb) = total_kb
        && total_kb < 2_000_000
    {
        tracing::warn!(
            available_mb = total_kb / 1024,
            "low memory for native storage; recommend at least 2 GB or ARQEN_STORAGE_MODE=http"
        );
    }
}

#[cfg(all(not(target_os = "linux"), feature = "thingd-native"))]
fn warn_if_low_native_memory() {}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn memory_mode_constructs_standalone_backend() {
        let backend = StorageFactory::build(&AppConfig::default()).unwrap();
        backend
            .put_object("test", "one", serde_json::json!({"ok":true}))
            .await
            .unwrap();
        assert!(backend.get_object("test", "one").await.unwrap().is_some());
    }

    #[test]
    fn cloud_mode_is_explicitly_not_silently_memory_backed() {
        let mut config = AppConfig::default();
        config.storage.mode = StorageMode::Cloud;
        config.storage.cloud_url = Some("https://cloud.example".to_string());
        let error = match StorageFactory::build(&config) {
            Ok(_) => panic!("cloud mode must not construct a memory backend"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("future public thingd-cloud"));
    }
}
