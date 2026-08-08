//! Runtime construction of Arqen storage backends.

use std::sync::Arc;

use crate::config::{AppConfig, ConfigError, StorageMode};
use crate::thingd::ThingdBackend;

/// Builds the configured thingd backend for an application.
pub struct StorageFactory;

impl StorageFactory {
    /// Construct the backend selected by `config.storage.mode`.
    pub fn build(config: &AppConfig) -> Result<Arc<dyn ThingdBackend>, ConfigError> {
        match config.storage.mode {
            StorageMode::Memory => Ok(Arc::new(crate::thingd::MemoryThingdBackend::new())),
            StorageMode::Native | StorageMode::Persistent => {
                #[cfg(feature = "thingd-native")]
                {
                    let path = config.storage.persistent_path.as_ref().ok_or_else(|| {
                        ConfigError::MissingField {
                            field: "persistent_path".to_string(),
                            context: "required for native storage".to_string(),
                        }
                    })?;
                    let backend =
                        crate::thingd::NativeThingdBackend::persistent(path).map_err(|error| {
                            ConfigError::InvalidValue {
                                field: "persistent_path".to_string(),
                                value: path.display().to_string(),
                                expected: error.to_string(),
                            }
                        })?;
                    Ok(Arc::new(backend))
                }
                #[cfg(not(feature = "thingd-native"))]
                {
                    Err(ConfigError::InvalidValue {
                        field: "storage.mode".to_string(),
                        value: "native".to_string(),
                        expected: "Arqen built with the thingd-native feature".to_string(),
                    })
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
                    let mut backend = crate::thingd::HttpThingdBackend::new(url);
                    if let Some(token) = &config.storage.auth_token {
                        backend = backend.with_auth(token.inner());
                    }
                    Ok(Arc::new(backend))
                }
                #[cfg(not(feature = "http-client"))]
                {
                    Err(ConfigError::InvalidValue {
                        field: "storage.mode".to_string(),
                        value: "http".to_string(),
                        expected: "Arqen built with the http-client feature".to_string(),
                    })
                }
            }
            StorageMode::Cloud => Err(ConfigError::InvalidValue {
                field: "storage.mode".to_string(),
                value: "cloud".to_string(),
                expected: "a future public thingd-cloud adapter".to_string(),
            }),
        }
    }
}

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
