//! # Arqen
//!
//! Backend infrastructure for agent-ready applications.
//!
//! Arqen provides a complete backend toolkit including:
//! - HTTP server with Axum
//! - thingd integration for storage, events, search, and queues
//! - Typed agent tools and manifest generation
//! - Durable job workers with graceful shutdown
//! - Structured logging with tracing
//!
//! ## Quick Start
//!
//! ```rust,no_run
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
pub mod core;
pub mod http;
pub mod jobs;
pub mod logging;
pub mod thingd;

// Re-export commonly used types at crate root
pub use agent::{
    AgentManifest, EndpointMetadata, JobMetadata as AgentJobMetadata, ToolEffect, ToolMetadata,
    ToolRegistry,
};
pub use core::{AppError, ErrorKind};
pub use http::{create_router, start_server};
pub use jobs::{JobConfig, JobHandler, JobWorker, Worker};
pub use logging::init_logging;
pub use thingd::{HttpThingdBackend, MemoryThingdBackend, ThingdBackend};
