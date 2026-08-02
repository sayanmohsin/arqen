//! The public Arqen facade crate.
//!
//! Most applications can depend on `arqen` and use these modules directly.
//! The underlying crates remain separately publishable for smaller services
//! and advanced integrations.

pub use arqen_agent as agent;
pub use arqen_core as core;
pub use arqen_http as http;
pub use arqen_jobs as jobs;
pub use arqen_logging as logging;
pub use arqen_thingd as thingd;

pub use arqen_agent::{AgentManifest, EndpointMetadata, JobMetadata, ToolEffect, ToolMetadata};
pub use arqen_core::{AppError, ErrorKind};
pub use arqen_http::{RuntimeInfo, create_router, create_router_with_runtime, start_server};
pub use arqen_jobs::{JobConfig, JobHandler, JobWorker, Worker};
pub use arqen_thingd::{
    HttpThingdBackend, MemoryThingdBackend, NativeThingdEngine, NativeThingdStore, ThingdBackend,
};
