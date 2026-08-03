use crate::agent::{AgentManifest, EndpointMetadata, JobMetadata, ToolMetadata};
use std::collections::HashMap;

/// Registry for agent tools, jobs, and endpoints.
///
/// Use [`register_tool!`] macro for ergonomic tool registration.
#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToolEffect;

    fn sample_tool(name: &str) -> ToolMetadata {
        ToolMetadata {
            name: name.to_string(),
            description: format!("Tool {name}"),
            input: serde_json::json!({"type": "object"}),
            output: serde_json::json!({"type": "object"}),
            scopes: vec![],
            effect: ToolEffect::Read,
            idempotent: true,
            enqueues_job: None,
            timeout: None,
        }
    }

    #[test]
    fn test_register_and_get_tool() {
        let mut registry = ToolRegistry::default();
        registry.register_tool(sample_tool("tool_a"));
        registry.register_tool(sample_tool("tool_b"));

        assert!(registry.get_tool("tool_a").is_some());
        assert!(registry.get_tool("tool_b").is_some());
        assert!(registry.get_tool("tool_c").is_none());
    }

    #[test]
    fn test_list_tools() {
        let mut registry = ToolRegistry::default();
        registry.register_tool(sample_tool("a"));
        registry.register_tool(sample_tool("b"));
        registry.register_tool(sample_tool("c"));

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 3);
    }

    #[test]
    fn test_generate_manifest() {
        let mut registry = ToolRegistry::new("my-app", "2.0.0", "My App", "persistent");
        registry.register_tool(sample_tool("read"));
        registry.register_tool(sample_tool("write"));

        let manifest = registry.generate_manifest();
        assert_eq!(manifest.name, "my-app");
        assert_eq!(manifest.version, "2.0.0");
        assert_eq!(manifest.tools.len(), 2);
        assert_eq!(manifest.storage_mode, "persistent");
    }

    #[test]
    fn test_generate_tool_schema() {
        let mut registry = ToolRegistry::default();
        let tool = ToolMetadata {
            name: "get_user".to_string(),
            description: "Get user".to_string(),
            input: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}}}),
            output: serde_json::json!({"type": "object"}),
            scopes: vec!["read:users".to_string()],
            effect: ToolEffect::Read,
            idempotent: true,
            enqueues_job: None,
            timeout: Some(30),
        };
        registry.register_tool(tool);

        let schema = registry.generate_tool_schema("get_user").unwrap();
        assert_eq!(schema["name"], "get_user");
        assert_eq!(schema["timeout"], 30);

        assert!(registry.generate_tool_schema("nonexistent").is_none());
    }

    #[test]
    fn test_register_tool_overwrites_existing() {
        let mut registry = ToolRegistry::default();
        registry.register_tool(sample_tool("tool"));
        registry.register_tool(ToolMetadata {
            name: "tool".to_string(),
            description: "Updated".to_string(),
            input: serde_json::json!({}),
            output: serde_json::json!({}),
            scopes: vec![],
            effect: ToolEffect::Write,
            idempotent: false,
            enqueues_job: None,
            timeout: None,
        });

        let tool = registry.get_tool("tool").unwrap();
        assert_eq!(tool.description, "Updated");
        assert!(matches!(tool.effect, ToolEffect::Write));
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
    ($registry:expr, $name:expr, $description:expr, $input:expr, $output:expr, $scopes:expr, $effect:expr, $idempotent:expr, $job:expr, $timeout:expr) => {
        $registry.register_tool($crate::ToolMetadata {
            name: $name.to_string(),
            description: $description.to_string(),
            input: $input,
            output: $output,
            scopes: $scopes,
            effect: $effect,
            idempotent: $idempotent,
            enqueues_job: Some($job.to_string()),
            timeout: Some($timeout),
        });
    };
}
