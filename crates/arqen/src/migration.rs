//! Logical migration from an embedded Thingd store to a Thingd HTTP server.
//!
//! Migration is deliberately implemented as a versioned JSONL spool file. The
//! spool is bounded in memory, can be inspected by an operator, and makes a
//! retry after a network interruption safe: Thingd imports objects by key,
//! events by idempotency key, and jobs by stable ID.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;
use tokio_util::io::ReaderStream;

use thingd::{ListEventsOptions, ListObjectsOptions};

use crate::thingd::{NativeThingdEngine, NativeThingdStore};

const SNAPSHOT_VERSION: &str = "2.0.0";
const DEFAULT_BATCH_SIZE: usize = 100;

/// Configuration for an explicit native-to-HTTP migration.
#[derive(Debug, Clone)]
pub struct ThingdMigrationOptions {
    pub source_path: PathBuf,
    pub destination_url: String,
    pub destination_auth_token: Option<String>,
    pub dry_run: bool,
    pub resume: bool,
    pub include_replication: bool,
    pub batch_size: usize,
    /// Optional path for the resumable JSONL spool. It is retained after a
    /// successful run so operators can archive or inspect the logical export.
    pub snapshot_path: Option<PathBuf>,
    /// Hex encoded Thingd encryption key for an encrypted native source.
    pub source_encryption_key: Option<String>,
}

impl Default for ThingdMigrationOptions {
    fn default() -> Self {
        Self {
            source_path: PathBuf::new(),
            destination_url: String::new(),
            destination_auth_token: None,
            dry_run: false,
            resume: false,
            include_replication: false,
            batch_size: DEFAULT_BATCH_SIZE,
            snapshot_path: None,
            source_encryption_key: None,
        }
    }
}

/// Counts and source metadata collected during migration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationReport {
    pub objects: u64,
    pub events: u64,
    pub jobs: u64,
    pub indexes: u64,
    pub records: u64,
    pub dry_run: bool,
    pub resumed: bool,
    pub snapshot_path: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("source path does not exist: {0}")]
    MissingSource(PathBuf),
    #[error("source store is invalid or incompatible: {0}")]
    InvalidSource(String),
    #[error("destination URL is required")]
    MissingDestination,
    #[error("destination is not empty; migration requires an empty destination")]
    NonEmptyDestination,
    #[error("destination request failed ({status}): {message}")]
    Destination { status: StatusCode, message: String },
    #[error("network interruption: {0}")]
    Network(#[from] reqwest::Error),
    #[error("migration snapshot I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("malformed migration record {record}: {message}")]
    MalformedRecord { record: u64, message: String },
    #[error("Thingd migration failed for {kind} {identifier}: {message}")]
    Thingd {
        kind: &'static str,
        identifier: String,
        message: String,
    },
    #[error("source encryption key is invalid: {0}")]
    Encryption(String),
}

#[derive(Debug, Clone, Default)]
struct SourceMetadata {
    indexes: Vec<(String, String)>,
    schema_json: Option<String>,
    schema_hash: Option<String>,
    schema_updated_at: Option<String>,
}

/// Migrates one embedded native store to one standalone Thingd server.
#[derive(Debug, Default, Clone, Copy)]
pub struct NativeToHttpMigrator;

impl NativeToHttpMigrator {
    pub async fn validate(
        &self,
        options: &ThingdMigrationOptions,
    ) -> Result<MigrationReport, MigrationError> {
        let normalized = validate_options(options)?;
        let (report, _) = self.create_snapshot(&normalized).await?;
        self.ensure_destination_empty(&normalized).await?;
        Ok(MigrationReport {
            dry_run: true,
            snapshot_path: Some(snapshot_path(&normalized)),
            ..report
        })
    }

    pub async fn migrate(
        &self,
        options: &ThingdMigrationOptions,
    ) -> Result<MigrationReport, MigrationError> {
        let normalized = validate_options(options)?;
        let path = snapshot_path(&normalized);
        let resumed = normalized.resume && valid_snapshot_header(&path)?;
        let (mut report, metadata) = if resumed {
            (
                count_snapshot(&path)?,
                self.read_metadata(&normalized).await?,
            )
        } else {
            self.create_snapshot(&normalized).await?
        };
        report.snapshot_path = Some(path.clone());
        report.resumed = resumed;
        report.dry_run = normalized.dry_run;
        self.ensure_destination_empty(&normalized).await?;
        if normalized.dry_run {
            return Ok(report);
        }
        self.import_snapshot(&normalized, &path, &mut report)
            .await?;
        self.import_metadata(&normalized, &metadata).await?;
        Ok(report)
    }

    async fn create_snapshot(
        &self,
        options: &ThingdMigrationOptions,
    ) -> Result<(MigrationReport, SourceMetadata), MigrationError> {
        let path = snapshot_path(options);
        let source = open_source(options)?;
        let batch_size = options.batch_size;
        let include_replication = options.include_replication;
        let path_for_worker = path.clone();
        let (report, metadata) = tokio::task::spawn_blocking(move || {
            write_snapshot(source, &path_for_worker, batch_size, include_replication)
        })
        .await
        .map_err(|error| MigrationError::InvalidSource(error.to_string()))??;
        Ok((
            MigrationReport {
                snapshot_path: Some(path),
                ..report
            },
            metadata,
        ))
    }

    async fn ensure_destination_empty(
        &self,
        options: &ThingdMigrationOptions,
    ) -> Result<(), MigrationError> {
        let client = http_client(options)?;
        let response = client
            .get(format!(
                "{}/v1/snapshot",
                base_url(&options.destination_url)
            ))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(response_error(response).await);
        }
        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();
        let mut header_seen = false;
        while let Some(chunk) = futures_next(&mut stream).await {
            let chunk = chunk?;
            buffer.extend_from_slice(&chunk);
            while let Some(newline) = buffer.iter().position(|byte| *byte == b'\n') {
                let line = buffer.drain(..=newline).collect::<Vec<_>>();
                if line.iter().all(u8::is_ascii_whitespace) {
                    continue;
                }
                let value: Value = serde_json::from_slice(&line).map_err(|error| {
                    MigrationError::MalformedRecord {
                        record: 0,
                        message: error.to_string(),
                    }
                })?;
                if !header_seen {
                    if value["type"] != "thingd.snapshot" || value["version"] != SNAPSHOT_VERSION {
                        return Err(MigrationError::InvalidSource(
                            "destination does not support Thingd snapshot 2.0.0".into(),
                        ));
                    }
                    header_seen = true;
                } else if value["type"].as_str() != Some("error") {
                    return Err(MigrationError::NonEmptyDestination);
                }
            }
        }
        if !header_seen {
            return Err(MigrationError::InvalidSource(
                "destination snapshot is missing its header".into(),
            ));
        }
        Ok(())
    }

    async fn import_snapshot(
        &self,
        options: &ThingdMigrationOptions,
        path: &Path,
        report: &mut MigrationReport,
    ) -> Result<(), MigrationError> {
        let client = http_client(options)?;
        let file = tokio::fs::File::open(path).await?;
        let response = client
            .post(format!(
                "{}/v1/snapshot",
                base_url(&options.destination_url)
            ))
            .header(reqwest::header::CONTENT_TYPE, "application/x-ndjson")
            .body(reqwest::Body::wrap_stream(ReaderStream::new(file)))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(response_error(response).await);
        }
        let body: Value = response.json().await.map_err(MigrationError::Network)?;
        let data = body.get("data").unwrap_or(&body);
        report.records = data["records"].as_u64().unwrap_or(report.records);
        report.objects = data["objects"].as_u64().unwrap_or(report.objects);
        report.events = data["events"].as_u64().unwrap_or(report.events);
        report.jobs = data["queues"].as_u64().unwrap_or(report.jobs);
        Ok(())
    }

    async fn read_metadata(
        &self,
        options: &ThingdMigrationOptions,
    ) -> Result<SourceMetadata, MigrationError> {
        let source = open_source(options)?;
        let result = tokio::task::spawn_blocking(move || source.with_engine(metadata_from_store))
            .await
            .map_err(|error| MigrationError::InvalidSource(error.to_string()))?;
        result.map_err(|error| MigrationError::InvalidSource(error.to_string()))?
    }

    async fn import_metadata(
        &self,
        options: &ThingdMigrationOptions,
        metadata: &SourceMetadata,
    ) -> Result<(), MigrationError> {
        let client = http_client(options)?;
        for (collection, field) in &metadata.indexes {
            let response = client
                .post(format!("{}/v1/indexes", base_url(&options.destination_url)))
                .json(&json!({"collection": collection, "field": field, "unique": false}))
                .send()
                .await?;
            if !response.status().is_success() && response.status() != StatusCode::CONFLICT {
                return Err(response_error(response).await);
            }
        }
        if let (Some(schema_json), Some(hash), Some(updated_at)) = (
            &metadata.schema_json,
            &metadata.schema_hash,
            &metadata.schema_updated_at,
        ) {
            let response = client
                .put(format!(
                    "{}/v1/schema/current",
                    base_url(&options.destination_url)
                ))
                .json(&json!({"schemaJson": schema_json, "hash": hash, "updatedAt": updated_at}))
                .send()
                .await?;
            if !response.status().is_success() {
                return Err(response_error(response).await);
            }
        }
        Ok(())
    }
}

fn validate_options(
    options: &ThingdMigrationOptions,
) -> Result<ThingdMigrationOptions, MigrationError> {
    if !options.source_path.exists() {
        return Err(MigrationError::MissingSource(options.source_path.clone()));
    }
    if options.destination_url.trim().is_empty() {
        return Err(MigrationError::MissingDestination);
    }
    let mut options = options.clone();
    options.batch_size = options.batch_size.max(1);
    Ok(options)
}

fn snapshot_path(options: &ThingdMigrationOptions) -> PathBuf {
    options
        .snapshot_path
        .clone()
        .unwrap_or_else(|| options.source_path.with_extension("arqen-migration.jsonl"))
}

fn base_url(url: &str) -> String {
    url.trim_end_matches('/')
        .trim_end_matches("/v1")
        .to_string()
}

fn http_client(options: &ThingdMigrationOptions) -> Result<Client, MigrationError> {
    let mut builder = Client::builder().timeout(std::time::Duration::from_secs(60));
    if let Some(token) = &options.destination_auth_token {
        builder = builder.default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {token}").parse().map_err(|_| {
                    MigrationError::InvalidSource("invalid destination auth token".into())
                })?,
            );
            headers
        });
    }
    builder.build().map_err(MigrationError::Network)
}

async fn response_error(response: reqwest::Response) -> MigrationError {
    let status = response.status();
    let message = response
        .text()
        .await
        .unwrap_or_else(|_| "request failed".into());
    MigrationError::Destination { status, message }
}

async fn futures_next<S>(stream: &mut S) -> Option<Result<bytes::Bytes, reqwest::Error>>
where
    S: futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
{
    futures_util::StreamExt::next(stream).await
}

fn open_source(options: &ThingdMigrationOptions) -> Result<NativeThingdStore, MigrationError> {
    let open_options = if let Some(key) = &options.source_encryption_key {
        let bytes =
            hex::decode(key).map_err(|error| MigrationError::Encryption(error.to_string()))?;
        let encryption = thingd::EncryptionConfig::from_key(&bytes)
            .map_err(|error| MigrationError::Encryption(error.to_string()))?;
        thingd::PersistentOpenOptions {
            encryption: Some(encryption),
            ..Default::default()
        }
    } else {
        thingd::PersistentOpenOptions::default()
    };
    NativeThingdStore::persistent_with_options(&options.source_path, open_options)
        .map_err(|error| MigrationError::InvalidSource(error.to_string()))
}

fn write_snapshot(
    source: NativeThingdStore,
    path: &Path,
    batch_size: usize,
    include_replication: bool,
) -> Result<(MigrationReport, SourceMetadata), MigrationError> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)?;
    let mut output = BufWriter::new(file);
    writeln!(
        output,
        "{}",
        serde_json::to_string(
            &json!({"type":"thingd.snapshot","version":SNAPSHOT_VERSION,"format":"jsonl"})
        )
        .map_err(|error| MigrationError::InvalidSource(error.to_string()))?
    )?;
    let mut report = MigrationReport::default();
    let metadata = source
        .with_engine(|engine| -> Result<SourceMetadata, MigrationError> {
            let collections = engine
                .with_store(|store| store.list_collections())
                .map_err(|error| thingd_error("collections", error))?;
            for collection in collections {
                if !include_replication && collection.starts_with("__thingd") {
                    continue;
                }
                let mut offset = 0u64;
                loop {
                    let page = engine
                        .with_store(|store| {
                            store.list_objects(
                                Some(std::slice::from_ref(&collection)),
                                &ListObjectsOptions {
                                    limit: Some(batch_size as u64),
                                    offset: Some(offset),
                                    ..Default::default()
                                },
                            )
                        })
                        .map_err(|error| thingd_error("objects", error))?;
                    let len = page.len() as u64;
                    for object in page {
                        write_record(&mut output, json!({"type":"object","object":object}))?;
                        report.objects += 1;
                    }
                    if len < batch_size as u64 {
                        break;
                    }
                    offset += len;
                }
            }
            let mut from_sequence = 0;
            loop {
                let page = engine
                    .with_store(|store| {
                        store.list_events(
                            None,
                            ListEventsOptions {
                                from_sequence: Some(from_sequence),
                                limit: Some(batch_size as u64),
                                ..Default::default()
                            },
                        )
                    })
                    .map_err(|error| thingd_error("events", error))?;
                let len = page.len();
                let previous_sequence = from_sequence;
                for event in page {
                    from_sequence = event.sequence;
                    if include_replication || !event.stream.starts_with("__thingd") {
                        write_record(&mut output, json!({"type":"event","event":event}))?;
                        report.events += 1;
                    }
                }
                if len < batch_size || from_sequence <= previous_sequence {
                    break;
                }
            }
            let queues = engine
                .with_store(|store| store.list_queues())
                .map_err(|error| thingd_error("queues", error))?;
            for queue in queues {
                if !include_replication && queue.starts_with("__thingd") {
                    continue;
                }
                let jobs = engine
                    .with_store(|store| store.list_jobs(&queue))
                    .map_err(|error| thingd_error_with_id("queue", &queue, error))?;
                for job in jobs {
                    write_record(&mut output, json!({"type":"queue","job":job}))?;
                    report.jobs += 1;
                }
                let dead = engine
                    .with_store(|store| store.list_dead_jobs(&queue))
                    .map_err(|error| thingd_error_with_id("queue", &queue, error))?;
                for job in dead {
                    write_record(&mut output, json!({"type":"queue","job":job}))?;
                    report.jobs += 1;
                }
            }
            let metadata = metadata_from_store(engine)?;
            report.indexes = metadata.indexes.len() as u64;
            report.records = report.objects + report.events + report.jobs;
            Ok(metadata)
        })
        .map_err(|error| MigrationError::InvalidSource(error.to_string()))??;
    output.flush()?;
    Ok((report, metadata))
}

fn write_record(output: &mut BufWriter<File>, value: Value) -> Result<(), MigrationError> {
    serde_json::to_writer(&mut *output, &value)
        .map_err(|error| MigrationError::InvalidSource(error.to_string()))?;
    output.write_all(b"\n")?;
    Ok(())
}

fn metadata_from_store(engine: &mut NativeThingdEngine) -> Result<SourceMetadata, MigrationError> {
    engine.with_store(|store| {
        let indexes = store
            .list_index_definitions()
            .map_err(|error| thingd_error("indexes", error))?
            .into_iter()
            .map(|index| (index.collection, index.field))
            .collect();
        let schema = store
            .get_schema_document()
            .map_err(|error| thingd_error("schema", error))?;
        Ok(match schema {
            None => SourceMetadata {
                indexes,
                ..Default::default()
            },
            Some(schema) => SourceMetadata {
                indexes,
                schema_json: Some(schema.schema_json),
                schema_hash: Some(schema.hash),
                schema_updated_at: Some(schema.updated_at),
            },
        })
    })
}

fn thingd_error(kind: &'static str, error: impl std::fmt::Display) -> MigrationError {
    MigrationError::Thingd {
        kind,
        identifier: kind.into(),
        message: error.to_string(),
    }
}
fn thingd_error_with_id(
    kind: &'static str,
    identifier: &str,
    error: impl std::fmt::Display,
) -> MigrationError {
    MigrationError::Thingd {
        kind,
        identifier: identifier.into(),
        message: error.to_string(),
    }
}
fn valid_snapshot_header(path: &Path) -> Result<bool, MigrationError> {
    let file = File::open(path)?;
    let line = BufReader::new(file)
        .lines()
        .next()
        .transpose()?
        .unwrap_or_default();
    let value: Value =
        serde_json::from_str(&line).map_err(|error| MigrationError::MalformedRecord {
            record: 0,
            message: error.to_string(),
        })?;
    Ok(value["type"] == "thingd.snapshot" && value["version"] == SNAPSHOT_VERSION)
}
fn count_snapshot(path: &Path) -> Result<MigrationReport, MigrationError> {
    let file = File::open(path)?;
    let mut report = MigrationReport::default();
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value =
            serde_json::from_str(&line).map_err(|error| MigrationError::MalformedRecord {
                record: index as u64,
                message: error.to_string(),
            })?;
        match value["type"].as_str() {
            Some("object") => report.objects += 1,
            Some("event") => report.events += 1,
            Some("queue") => report.jobs += 1,
            Some("thingd.snapshot") => {}
            Some(other) => {
                return Err(MigrationError::MalformedRecord {
                    record: index as u64,
                    message: format!("unsupported snapshot record type '{other}'"),
                });
            }
            None => {
                return Err(MigrationError::MalformedRecord {
                    record: index as u64,
                    message: "missing record type".into(),
                });
            }
        }
    }
    report.records = report.objects + report.events + report.jobs;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use thingd::{MemoryEvent, MemoryObject};

    fn test_path(suffix: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "arqen-migration-test-{}-{suffix}",
            uuid::Uuid::new_v4()
        ))
    }

    #[test]
    fn native_export_preserves_keys_bodies_and_source() {
        let snapshot_path = test_path("snapshot.jsonl");
        let source = NativeThingdStore::memory();
        source
            .with_engine(|engine| {
                engine.with_store(|store| {
                    store
                        .put_object(MemoryObject::new("users", "stable-id", r#"{"name":"Ada"}"#))
                        .expect("object writes");
                    store
                        .append_event(MemoryEvent::new("audit", "user.created", r#"{"id":1}"#))
                        .expect("event writes");
                });
            })
            .expect("source access");

        let (report, _) =
            write_snapshot(source.clone(), &snapshot_path, 1, false).expect("snapshot writes");
        assert_eq!(report.objects, 1);
        assert_eq!(report.events, 1);
        let contents = std::fs::read_to_string(&snapshot_path).expect("snapshot reads");
        assert!(contents.contains(r#""id":"stable-id""#));
        assert!(contents.contains(r#""body":"{\"name\":\"Ada\"}""#));
        let source_id = source
            .with_engine(|engine| {
                engine
                    .with_store(|store| store.get_object("users", "stable-id"))
                    .expect("source reads")
                    .expect("object exists")
                    .key
                    .id
                    .clone()
            })
            .expect("source access");
        assert_eq!(source_id, "stable-id");

        let _ = std::fs::remove_file(snapshot_path);
    }

    #[test]
    fn snapshot_counts_are_bounded_by_records_not_file_size() {
        let path = test_path("counts.jsonl");
        std::fs::write(
            &path,
            "{\"type\":\"thingd.snapshot\",\"version\":\"2.0.0\"}\n{\"type\":\"object\",\"object\":{}}\n{\"type\":\"event\",\"event\":{}}\n{\"type\":\"queue\",\"job\":{}}\n",
        )
        .expect("snapshot writes");
        let report = count_snapshot(&path).expect("snapshot counts");
        assert_eq!(
            (report.objects, report.events, report.jobs, report.records),
            (1, 1, 1, 3)
        );
        let _ = std::fs::remove_file(path);
    }
}
