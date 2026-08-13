//! # Arqen
//!
//! Backend infrastructure for agent-ready applications.
//!
//! Arqen provides a complete backend toolkit including:
//! - HTTP server with Axum (feature: `http-server`)
//! - thingd integration for storage, events, search, and queues (feature: `thingd-native`)
//! - Typed agent tools and manifest generation
//! - Durable job workers with graceful shutdown
//! - Structured logging with tracing (feature: `logging`)
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use arqen::http::{create_router, start_server};
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//!     let addr: SocketAddr = "127.0.0.1:8888".parse()?;
//!     let router = create_router();
//!     start_server(addr, router).await?;
//!     Ok(())
//! }
//! ```

pub mod agent;
#[cfg(feature = "http-server")]
pub mod app;
pub mod auth;
#[cfg(feature = "cli")]
pub mod cli;
pub mod config;
#[cfg(feature = "http-server")]
pub mod context;
pub mod core;
#[cfg(feature = "cli")]
pub mod dev;
pub mod health;
pub mod jobs;
#[cfg(feature = "thingd-migration")]
pub mod migration;
pub mod module;
pub mod observability;
pub mod openapi;
pub mod prelude;
pub mod schema;
pub mod state;
pub mod thingd;

#[cfg(feature = "http-server")]
pub mod validation;

#[cfg(any(test, feature = "test-util"))]
pub mod testutil;

#[cfg(feature = "http-server")]
pub mod http;
#[cfg(feature = "logging")]
pub mod logging;

// Re-export commonly used types at crate root
pub use agent::{
    AgentManifest, EndpointMetadata, JobMetadata, ToolContext, ToolEffect, ToolHandler,
    ToolMetadata, ToolOutcome, ToolRegistry,
};
pub use config::{
    AppConfig, AuthConfig, CliOverrides, HealthConfig, LogFormat, LoggingConfig, Secret,
    ServerConfig, StorageConfig, StorageMode, ThingdSyncMode, WorkerConfig,
};
#[cfg(feature = "http-server")]
pub use context::RequestContext;
pub use core::{AppError, ErrorKind};
pub use jobs::{JobConfig, JobHandler, JobWorker, Worker};
#[cfg(feature = "thingd-migration")]
pub use migration::{
    MigrationError, MigrationReport, NativeToHttpMigrator, ThingdMigrationOptions,
};
pub use module::{
    Module, ModuleBuilder, ModuleContext, ModuleError, ModuleGraphError, ModuleHealth,
};
#[cfg(feature = "http-server")]
pub use observability::{
    CacheMetric, JobMetric, MetricsSink, NoopMetricsSink, RequestMetric, SharedMetricsSink,
    StorageMetric, SyncMetric,
};
pub use state::{AppState, AppStateBuilder};
#[cfg(feature = "http-client")]
pub use thingd::{
    ApplyResult, FileSyncCheckpointStore, HttpClientPolicy, HttpThingdBackend, ReplicationChange,
    ReplicationSnapshot, ReplicationStatus, SyncCheckpointStore, SyncClientPolicy, SyncEndpoint,
    SyncPage, SyncRuntimeStatus, ThingdSyncClient, ThingdSyncWorker,
};
pub use thingd::{
    BootstrapPolicy, CachePolicy, CachingThingdBackend, MemoryThingdBackend, ScopeSubject,
    ScopedThingdBackend, StorageFactory, StorageScope, ThingdBackend, retry_bootstrap,
    seed_with_retry,
};

#[cfg(feature = "http-server")]
pub use auth::{AuthContext, AuthError, Authentication};
#[cfg(feature = "http-server")]
pub use health::{
    HealthCheck, HealthRegistry, HealthReport, HealthStatus, ProbeType, ThingdSyncHealth,
};
#[cfg(feature = "http-server")]
pub use observability::{MetricsReport, RequestMetrics};
#[cfg(feature = "http-server")]
pub use validation::{FieldError, Validate, Validated, ValidationErrors};

#[cfg(feature = "http-server")]
pub use http::{
    AuthGuard, Authenticated, HttpCachePolicy, HttpModule, RequireAuth, auth_middleware,
    builtin_routes, create_router, create_router_with_state, create_router_with_state_and_routes,
    jsonl_response, merge_module_routes, nest_routes, optional_auth_middleware,
    require_auth_middleware,
};
#[cfg(feature = "logging")]
pub use logging::{init_logging, init_logging_with_config};
#[cfg(all(feature = "thingd-native", feature = "http-client"))]
pub use thingd::NativeThingdSyncEndpoint;
#[cfg(feature = "thingd-native")]
pub use thingd::{NativeThingdBackend, NativeThingdStore};
