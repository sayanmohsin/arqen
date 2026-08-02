pub mod registry;
pub mod schema;

pub use registry::ToolRegistry;
pub use schema::{Schema, SchemaGenerator};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub scopes: Vec<String>,
    pub effect: ToolEffect,
    pub idempotent: bool,
    pub enqueues_job: Option<String>,
    pub timeout: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolEffect {
    Read,
    Write,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub storage_mode: String,
    pub tools: Vec<ToolMetadata>,
    pub jobs: Vec<JobMetadata>,
    pub endpoints: Vec<EndpointMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    pub name: String,
    pub description: String,
    pub payload: serde_json::Value,
    pub queue: String,
    pub max_retries: u32,
    pub timeout: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetadata {
    pub path: String,
    pub method: String,
    pub description: String,
    pub authenticated: bool,
}
