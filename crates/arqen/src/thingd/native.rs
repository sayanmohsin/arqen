//! Direct integration with the public `thingd` Rust crate.
//!
//! This is the canonical embedded storage entry point for Arqen. It keeps the
//! engine behind a small Arqen-owned handle while exposing the real thingd
//! engine to application code. Improvements made in thingd therefore become
//! available to Arqen through the normal dependency update path; Arqen does
//! not reimplement object, event, queue, or link semantics here.
//!
//! # Relationship to `ThingdBackend`
//!
//! `NativeThingdStore` intentionally does **not** implement the Arqen
//! [`ThingdBackend`](super::traits::ThingdBackend) trait. The two APIs serve
//! different purposes:
//!
//! - **`ThingdBackend`** is an async trait with Arqen-owned types
//!   (`ThingdObject`, `ThingdEvent`, …). It is designed for trait-object
//!   polymorphism and HTTP/worker integration.
//! - **`NativeThingdStore`** wraps the real thingd engine synchronously and
//!   exposes its full API through [`NativeThingdStore::with_engine`] and
//!   [`NativeThingdStore::lock`]. This gives application code direct access to
//!   thingd's storage traits (ObjectStore, EventLog, QueueStore, LinkStore)
//!   without lossy type conversions.
//!
//! Use `NativeThingdStore` when you need the complete thingd feature set
//! (search, aggregation, vector search, links, etc.). Use `MemoryThingdBackend`
//! or `HttpThingdBackend` when you need async trait-object polymorphism.
//!
//! The replication-aware mutation helpers below are the normal path for a
//! native source that feeds `ThingdSyncWorker`. Callers using
//! [`NativeThingdStore::with_engine`] or [`NativeThingdStore::lock`] directly
//! are taking an advanced escape hatch and must call Thingd's public
//! `ReplicationService::record_*` methods themselves after source mutations.

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::{AppError, ErrorKind};

use thingd::{
    MemoryEvent, MemoryObject, ReplicationConfig, ReplicationService, ThingStore, ThingdResult,
};

/// An embedded thingd engine selected by deployment mode.
pub enum NativeThingdEngine {
    /// Ephemeral engine for local development and tests.
    Memory(thingd::MemoryEngine),
    /// Durable persistent engine for a single Arqen process.
    Persistent(thingd::PersistentEngine),
}

impl NativeThingdEngine {
    pub(crate) fn with_store<R>(&mut self, operation: impl FnOnce(&mut dyn ThingStore) -> R) -> R {
        match self {
            Self::Memory(engine) => operation(engine),
            Self::Persistent(engine) => operation(engine),
        }
    }

    pub(crate) fn with_replication_service<R>(
        &mut self,
        config: ReplicationConfig,
        operation: impl FnOnce(&mut ReplicationService<'_>) -> ThingdResult<R>,
    ) -> ThingdResult<R> {
        self.with_store(|store| operation(&mut ReplicationService::new(store, config)))
    }
}

/// Thread-safe handle for an embedded native thingd engine.
///
/// thingd's storage traits are intentionally synchronous and mutable. The
/// handle serializes access at this boundary so Axum handlers and workers can
/// share one engine without inventing a second storage contract.
#[derive(Clone)]
pub struct NativeThingdStore {
    engine: Arc<Mutex<NativeThingdEngine>>,
}

fn lock_engine(
    store: &Mutex<NativeThingdEngine>,
) -> Result<MutexGuard<'_, NativeThingdEngine>, AppError> {
    store.lock().map_err(|e| {
        AppError::new(
            ErrorKind::Internal,
            format!("thingd engine mutex poisoned: {e}"),
        )
    })
}

impl NativeThingdStore {
    /// Create a new in-memory native thingd store.
    #[must_use]
    pub fn memory() -> Self {
        Self {
            engine: Arc::new(Mutex::new(NativeThingdEngine::Memory(
                thingd::MemoryEngine::new(),
            ))),
        }
    }

    /// Open a durable persistent native thingd store.
    pub fn persistent(path: impl AsRef<Path>) -> Result<Self, thingd::ThingdError> {
        Self::persistent_with_options(path, thingd::PersistentOpenOptions::default())
    }

    /// Open a durable store with Thingd's explicit persistence options.
    pub fn persistent_with_options(
        path: impl AsRef<Path>,
        options: thingd::PersistentOpenOptions,
    ) -> Result<Self, thingd::ThingdError> {
        let engine = thingd::PersistentEngine::open_with_options(path, options)?;
        Ok(Self {
            engine: Arc::new(Mutex::new(NativeThingdEngine::Persistent(engine))),
        })
    }

    /// Borrow the underlying native engine for one synchronous operation.
    ///
    /// Keep the closure short: it holds the shared engine lock for its full
    /// duration. Application code should use this for thingd operations, not
    /// for network calls or other blocking work.
    pub fn with_engine<R>(
        &self,
        operation: impl FnOnce(&mut NativeThingdEngine) -> R,
    ) -> Result<R, AppError> {
        let mut engine = lock_engine(&self.engine)?;
        Ok(operation(&mut engine))
    }

    /// Lock the underlying engine for advanced integrations.
    pub fn lock(&self) -> Result<MutexGuard<'_, NativeThingdEngine>, AppError> {
        lock_engine(&self.engine)
    }

    /// Put an object and record its source replication change while holding
    /// the same native engine lock.
    pub fn put_object_replicated(
        &self,
        object: MemoryObject,
        config: &ReplicationConfig,
    ) -> Result<MemoryObject, AppError> {
        ensure_source_config(config)?;
        let config = config.clone();
        self.with_engine(|engine| {
            engine.with_store(|store| {
                let object = store
                    .put_object(object)
                    .map_err(|error| AppError::new(ErrorKind::Internal, error.to_string()))?;
                ReplicationService::new(store, config)
                    .record_object_upsert(&object)
                    .map_err(|error| AppError::new(ErrorKind::Internal, error.to_string()))?;
                Ok(object)
            })
        })?
    }

    /// Delete an object and record its source replication tombstone change.
    pub fn delete_object_replicated(
        &self,
        collection: &str,
        id: &str,
        config: &ReplicationConfig,
    ) -> Result<(), AppError> {
        ensure_source_config(config)?;
        let config = config.clone();
        self.with_engine(|engine| {
            engine.with_store(|store| {
                store
                    .delete_object(collection, id)
                    .map_err(|error| AppError::new(ErrorKind::Internal, error.to_string()))?;
                ReplicationService::new(store, config)
                    .record_object_delete(collection, id)
                    .map_err(|error| AppError::new(ErrorKind::Internal, error.to_string()))
            })
        })?
    }

    /// Append an application event and record its source replication change.
    pub fn append_event_replicated(
        &self,
        event: MemoryEvent,
        config: &ReplicationConfig,
    ) -> Result<MemoryEvent, AppError> {
        ensure_source_config(config)?;
        let config = config.clone();
        self.with_engine(|engine| {
            engine.with_store(|store| {
                let event = store
                    .append_event(event)
                    .map_err(|error| AppError::new(ErrorKind::Internal, error.to_string()))?;
                ReplicationService::new(store, config)
                    .record_event_append(&event)
                    .map_err(|error| AppError::new(ErrorKind::Internal, error.to_string()))?;
                Ok(event)
            })
        })?
    }
}

fn ensure_source_config(config: &ReplicationConfig) -> Result<(), AppError> {
    if config.role != thingd::ReplicationRole::Source {
        return Err(AppError::new(
            ErrorKind::Validation,
            "replication-aware native mutations require a source configuration",
        ));
    }
    if config.source_id.trim().is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "replication-aware native mutations require a source ID",
        ));
    }
    Ok(())
}
