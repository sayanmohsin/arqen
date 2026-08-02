use crate::agent::{AgentManifest, EndpointMetadata, JobMetadata, ToolMetadata};
use std::collections::HashMap;

pub struct ToolRegistry {
    tools: HashMap<String, ToolMetadata>,
    jobs: Vec<JobMetadata>,
    endpoints: Vec<EndpointMetadata>,
    app_name: String,
    app_version: String,
    app_description: String,
    storage_mode: String,
}

impl ToolRegistry {
    pub fn new(
        app_name: &str,
        app_version: &str,
        app_description: &str,
        storage_mode: &str,
    ) -> Self {
        Self {
            tools: HashMap::new(),
            jobs: Vec::new(),
            endpoints: Vec::new(),
            app_name: app_name.to_string(),
            app_version: app_version.to_string(),
            app_description: app_description.to_string(),
            storage_mode: storage_mode.to_string(),
        }
    }

    pub fn register_tool(&mut self, tool: ToolMetadata) {
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn register_job(&mut self, job: JobMetadata) {
        self.jobs.push(job);
    }

    pub fn register_endpoint(&mut self, endpoint: EndpointMetadata) {
        self.endpoints.push(endpoint);
    }

    pub fn get_tool(&self, name: &str) -> Option<&ToolMetadata> {
        self.tools.get(name)
    }

    pub fn list_tools(&self) -> Vec<&ToolMetadata> {
        self.tools.values().collect()
    }

    pub fn generate_manifest(&self) -> AgentManifest {
        AgentManifest {
            name: self.app_name.clone(),
            version: self.app_version.clone(),
            description: self.app_description.clone(),
            storage_mode: self.storage_mode.clone(),
            tools: self.tools.values().cloned().collect(),
            jobs: self.jobs.clone(),
            endpoints: self.endpoints.clone(),
        }
    }

    pub fn generate_tool_schema(&self, tool_name: &str) -> Option<serde_json::Value> {
        self.tools.get(tool_name).map(|tool| {
            serde_json::json!({
                "name": tool.name,
                "description": tool.description,
                "input": tool.input,
                "output": tool.output,
                "scopes": tool.scopes,
                "effect": tool.effect,
                "idempotent": tool.idempotent,
                "enqueues_job": tool.enqueues_job,
                "timeout": tool.timeout,
            })
        })
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new("arqen-app", "0.1.0", "An Arqen application", "memory")
    }
}

// Helper macro for registering tools
#[macro_export]
macro_rules! register_tool {
    ($registry:expr, $name:expr, $description:expr, $input:expr, $output:expr, $scopes:expr, $effect:expr, $idempotent:expr) => {
        $registry.register_tool($crate::ToolMetadata {
            name: $name.to_string(),
            description: $description.to_string(),
            input: $input,
            output: $output,
            scopes: $scopes,
            effect: $effect,
            idempotent: $idempotent,
            enqueues_job: None,
            timeout: None,
        });
    };
    ($registry:expr, $name:expr, $description:expr, $input:expr, $output:expr, $scopes:expr, $effect:expr, $idempotent:expr, $job:expr) => {
        $registry.register_tool($crate::ToolMetadata {
            name: $name.to_string(),
            description: $description.to_string(),
            input: $input,
            output: $output,
            scopes: $scopes,
            effect: $effect,
            idempotent: $idempotent,
            enqueues_job: Some($job.to_string()),
            timeout: None,
        });
    };
}
