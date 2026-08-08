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
pub use crate::app::ArqenApp;
pub use crate::config::{AppConfig, StorageMode};
#[cfg(feature = "http-server")]
pub use crate::context::RequestContext;
pub use crate::core::{AppError, ErrorKind};
pub use crate::jobs::JobHandler;
pub use crate::module::{Module, ModuleContext, ModuleError, ModuleHealth};
pub use crate::observability::{MetricsSink, NoopMetricsSink};
pub use crate::state::AppState;
#[cfg(feature = "http-client")]
pub use crate::thingd::HttpClientPolicy;
pub use crate::thingd::{
    CachePolicy, CachingThingdBackend, ScopedThingdBackend, StorageFactory, StorageScope,
    ThingdBackend,
};

pub use async_trait::async_trait;
