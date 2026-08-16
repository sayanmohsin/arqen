//! Ergonomic re-exports for common Arqen types.
//!
//! Import everything with:
//!
//! ```rust,ignore
//! use arqen::prelude::*;
//! ```

pub use crate::agent::{
    JobMetadata, ToolContext, ToolEffect, ToolHandler, ToolMetadata, ToolOutcome, ToolRegistry,
};
#[cfg(feature = "http-server")]
pub use crate::app::ArqenApp;
pub use crate::config::{AppConfig, StorageMode, ThingdSyncMode};
#[cfg(feature = "http-server")]
pub use crate::context::RequestContext;
pub use crate::core::{AppError, ErrorKind};
pub use crate::health::ThingdSyncHealth;
#[cfg(feature = "http-server")]
pub use crate::http::{HttpCachePolicy, jsonl_response};
pub use crate::jobs::JobHandler;
pub use crate::module::{Module, ModuleContext, ModuleError, ModuleHealth};
pub use crate::observability::{MetricsSink, NoopMetricsSink, SyncMetric};
pub use crate::schema::SchemaReport;
pub use crate::state::AppState;
#[cfg(feature = "http-client")]
pub use crate::thingd::{
    ApplyResult, FileSyncCheckpointStore, HttpClientPolicy, ReplicationChange, ReplicationSnapshot,
    ReplicationStatus, SyncCheckpointStore, SyncClientPolicy, SyncEndpoint, SyncPage,
    SyncRuntimeStatus, THINGD_HTTP_API_VERSION, ThingdCompatibilityReport, ThingdSyncClient,
    ThingdSyncWorker,
};
pub use crate::thingd::{
    BootstrapPolicy, CachePolicy, CachingThingdBackend, ScopedThingdBackend, StorageFactory,
    StorageScope, ThingdBackend, retry_bootstrap, seed_with_retry,
};
#[cfg(feature = "thingd-native")]
pub use crate::thingd::{NativeThingdBackend, NativeThingdEngine, NativeThingdStore};

pub use async_trait::async_trait;
