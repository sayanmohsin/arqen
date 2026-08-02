//! Direct integration with the public `thingd` Rust crate.
//!
//! This is the canonical embedded storage entry point for Arqen. It keeps the
//! engine behind a small Arqen-owned handle while exposing the real thingd
//! engine to application code. Improvements made in thingd therefore become
//! available to Arqen through the normal dependency update path; Arqen does
//! not reimplement object, event, queue, or link semantics here.

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

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

    /// Open a durable native thingd store.
    pub fn fjall(path: impl AsRef<Path>) -> Result<Self, thingd::ThingdError> {
        let engine = thingd::PersistentEngine::open(path)?;
        Ok(Self {
            engine: Arc::new(Mutex::new(NativeThingdEngine::Persistent(engine))),
        })
    }

    /// Borrow the underlying native engine for one synchronous operation.
    ///
    /// Keep the closure short: it holds the shared engine lock for its full
    /// duration. Application code should use this for thingd operations, not
    /// for network calls or other blocking work.
    pub fn with_engine<R>(&self, operation: impl FnOnce(&mut NativeThingdEngine) -> R) -> R {
        let mut engine = self.engine.lock().expect("thingd engine mutex poisoned");
        operation(&mut engine)
    }

    /// Lock the underlying engine for advanced integrations.
    pub fn lock(&self) -> MutexGuard<'_, NativeThingdEngine> {
        self.engine.lock().expect("thingd engine mutex poisoned")
    }
}
