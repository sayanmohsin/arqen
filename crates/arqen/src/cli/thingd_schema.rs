use super::{ThingdCommand, exit, output::Output};

pub fn run(command: &ThingdCommand, output: &Output) -> i32 {
    match command {
        #[cfg(feature = "thingd-migration")]
        ThingdCommand::Migrate { .. } => unreachable!("migration is dispatched by cli::dispatch"),
        ThingdCommand::SchemaValidate { path, url, token } => {
            match crate::schema::validate_file(path) {
                Ok(report) => {
                    if let Some(url) = url {
                        let client = crate::thingd::ThingdSyncClient::new(url);
                        let client = token
                            .as_deref()
                            .map(|token| client.clone().with_auth(token))
                            .unwrap_or(client);
                        let validation = tokio::runtime::Runtime::new()
                            .map_err(|error| {
                                crate::AppError::new(crate::ErrorKind::Internal, error.to_string())
                            })
                            .and_then(|runtime| {
                                runtime.block_on(client.validate_schema(&report.source))
                            });
                        match validation {
                            Ok(value) => output.print_json(
                                serde_json::json!({ "local": report, "remote": value }),
                            ),
                            Err(error) => {
                                output.print_error(&error.to_string());
                                return exit::RUNTIME;
                            }
                        }
                    } else if output.is_json() {
                        output.print_json(serde_json::to_value(report).unwrap_or_default());
                    } else {
                        output.print(&format!(
                        "schema source loaded; hash: {}\nThingd URL not supplied, so authoritative syntax validation was not performed",
                        report.hash
                    ));
                    }
                    exit::SUCCESS
                }
                Err(error) => {
                    output.print_error(&error.to_string());
                    exit::RUNTIME
                }
            }
        }
        ThingdCommand::SchemaRemote { url, token } => {
            let client = crate::thingd::ThingdSyncClient::new(url);
            let client = token
                .as_deref()
                .map(|token| client.clone().with_auth(token))
                .unwrap_or(client);
            let result = tokio::runtime::Runtime::new()
                .map_err(|error| {
                    crate::AppError::new(crate::ErrorKind::Internal, error.to_string())
                })
                .and_then(|runtime| {
                    runtime.block_on(async {
                        Ok::<_, crate::AppError>(serde_json::json!({
                            "schema": client.current_schema().await?,
                            "migrations": client.migrations().await?,
                        }))
                    })
                });
            match result {
                Ok(value) => {
                    output.print_json(value);
                    exit::SUCCESS
                }
                Err(error) => {
                    output.print_error(&error.to_string());
                    exit::RUNTIME
                }
            }
        }
        ThingdCommand::Seed {
            url,
            token,
            attempts,
        } => {
            let mut backend = crate::thingd::HttpThingdBackend::new(url);
            if let Some(token) = token {
                backend = backend.with_auth(token);
            }
            let policy = crate::thingd::BootstrapPolicy {
                max_attempts: *attempts,
                ..Default::default()
            };
            let result = tokio::runtime::Runtime::new()
                .map_err(|error| {
                    crate::AppError::new(crate::ErrorKind::Internal, error.to_string())
                })
                .and_then(|runtime| {
                    runtime.block_on(crate::thingd::seed_with_retry(
                        std::sync::Arc::new(backend),
                        policy,
                    ))
                });
            match result {
                Ok(()) => {
                    output.print("Thingd seed completed");
                    exit::SUCCESS
                }
                Err(error) => {
                    output.print_error(&error.to_string());
                    exit::RUNTIME
                }
            }
        }
    }
}
