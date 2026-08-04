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
//!   exposes its full API through [`with_engine`](Self::with_engine) and
//!   [`lock`](Self::lock). This gives application code direct access to
//!   thingd's storage traits (ObjectStore, EventLog, QueueStore, LinkStore)
//!   without lossy type conversions.
//!
//! Use `NativeThingdStore` when you need the complete thingd feature set
//! (search, aggregation, vector search, links, etc.). Use `MemoryThingdBackend`
//! or `HttpThingdBackend` when you need async trait-object polymorphism.

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::{AppError, ErrorKind};

/// An embedded thingd engine selected by deployment mode.
pub enum NativeThingdEngine {
    /// Ephemeral engine for local development and tests.
    Memory(thingd::MemoryEngine),
    /// Durable persistent engine for a single Arqen process.
    Persistent(thingd::PersistentEngine),
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
        let engine = thingd::PersistentEngine::open(path)
            .map_err(|error| thingd::ThingdError::Storage(error.to_string()))?;
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
}
