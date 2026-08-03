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
//!     let addr: SocketAddr = "127.0.0.1:3000".parse()?;
//!     let router = create_router();
//!     start_server(addr, router).await?;
//!     Ok(())
//! }
//! ```

pub mod agent;
pub mod auth;
pub mod config;
pub mod core;
pub mod health;
pub mod jobs;
pub mod state;
pub mod thingd;
pub mod validation;

#[cfg(feature = "http-server")]
pub mod http;
#[cfg(feature = "logging")]
pub mod logging;

// Re-export commonly used types at crate root
pub use agent::{
    AgentManifest, EndpointMetadata, JobMetadata, ToolEffect, ToolMetadata,
    ToolRegistry,
};
pub use config::{AppConfig, Secret, ServerConfig, StorageConfig, StorageMode};
pub use core::{AppError, ErrorKind};
pub use jobs::{JobConfig, JobHandler, JobWorker, Worker};
pub use state::{AppState, AppStateBuilder};
pub use thingd::{MemoryThingdBackend, ThingdBackend};

#[cfg(feature = "http-server")]
pub use http::{create_router, start_server};
#[cfg(feature = "logging")]
pub use logging::init_logging;
#[cfg(feature = "thingd-native")]
pub use thingd::NativeThingdStore;
#[cfg(feature = "http-client")]
pub use thingd::HttpThingdBackend;
