use crate::agent::execution::{ToolContext, ToolHandler, ToolOutcome, validate_against_schema};
use crate::agent::{AgentManifest, EndpointMetadata, JobMetadata, ToolMetadata};
use crate::core::{AppError, ErrorKind};
use crate::thingd::ThingdBackend;
use std::collections::HashMap;
use std::sync::Arc;

/// Default maximum retries when a tool with no handler enqueues a job.
const DEFAULT_TOOL_JOB_MAX_RETRIES: u32 = 3;
/// Default tool execution timeout in seconds when none is declared.
const DEFAULT_TOOL_TIMEOUT_SECS: u32 = 30;

/// Registry for agent tools, jobs, and endpoints.
///
/// Use [`register_tool!`](crate::register_tool) macro for ergonomic tool
/// registration and [`ToolRegistry::register_handler`] to bind an executable
/// handler to a tool.
pub struct ToolRegistry {
    tools: HashMap<String, ToolMetadata>,
    handlers: HashMap<String, Arc<dyn ToolHandler>>,
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
            handlers: HashMap::new(),
            jobs: Vec::new(),
            endpoints: Vec::new(),
            app_name: app_name.to_string(),
            app_version: app_version.to_string(),
            app_description: app_description.to_string(),
            storage_mode: storage_mode.to_string(),
        }
    }

    pub fn register_tool(&mut self, tool: ToolMetadata) {
        assert!(
            !self.tools.contains_key(&tool.name),
            "duplicate agent tool registration: {}",
            tool.name
        );
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Register an executable handler for a tool.
    ///
    /// The handler is looked up by `name` when the tool is executed. Register
    /// the tool's metadata first (e.g. via [`register_tool!`](crate::register_tool))
    /// so scope and schema checks can run before the handler is invoked.
    pub fn register_handler(
        &mut self,
        name: impl Into<String>,
        handler: impl ToolHandler + 'static,
    ) {
        let name = name.into();
        assert!(
            !self.handlers.contains_key(&name),
            "duplicate agent tool handler registration: {name}"
        );
        self.handlers.insert(name, Arc::new(handler));
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
        let mut tools: Vec<_> = self.tools.values().collect();
        tools.sort_by(|left, right| left.name.cmp(&right.name));
        tools
    }

    /// Execute a tool by name.
    ///
    /// Scope enforcement and input-schema validation run before the handler is
    /// invoked. The handler runs with the tool's declared timeout. If the tool
    /// declares an `enqueues_job` queue and no handler is registered, the job
    /// is pushed to `backend` instead of executing inline.
    ///
    /// Returns [`ToolOutcome::Output`] for inline execution or
    /// [`ToolOutcome::Enqueued`] when a job was pushed.
    pub async fn execute(
        &self,
        name: &str,
        input: serde_json::Value,
        ctx: &ToolContext,
        backend: Option<&dyn ThingdBackend>,
    ) -> Result<ToolOutcome, AppError> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| AppError::new(ErrorKind::NotFound, format!("tool not found: {name}")))?;

        if !tool.scopes.iter().all(|scope| ctx.scopes.contains(scope)) {
            return Err(AppError::new(
                ErrorKind::Authorization,
                format!("missing required scopes for tool: {name}"),
            ));
        }

        validate_against_schema(&input, &tool.input)?;

        if tool.enqueues_job.is_some() && !self.handlers.contains_key(name) {
            let queue = tool.enqueues_job.as_deref().unwrap_or_default();
            let Some(backend) = backend else {
                return Err(AppError::new(
                    ErrorKind::NotImpl,
                    format!("tool {name} enqueues a job but no handler or backend is configured"),
                ));
            };
            let job = backend
                .push_job(queue, input, DEFAULT_TOOL_JOB_MAX_RETRIES)
                .await?;
            return Ok(ToolOutcome::Enqueued {
                queue: queue.to_string(),
                job_id: job.id,
            });
        }

        let handler = self.handlers.get(name).ok_or_else(|| {
            AppError::new(
                ErrorKind::NotImpl,
                format!("tool has no handler registered: {name}"),
            )
        })?;

        let timeout_secs = tool.timeout.unwrap_or(DEFAULT_TOOL_TIMEOUT_SECS);
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs as u64),
            handler.execute(ctx, input),
        )
        .await
        .map_err(|_| {
            AppError::new(
                ErrorKind::Timeout,
                format!("tool {name} timed out after {timeout_secs}s"),
            )
        })??;

        Ok(ToolOutcome::Output(output))
    }

    pub fn generate_manifest(&self) -> AgentManifest {
        let mut endpoints = self.endpoints.clone();
        for tool in self.list_tools() {
            endpoints.push(EndpointMetadata {
                path: format!("/agent/tools/{}", tool.name),
                method: "POST".to_string(),
                description: format!("Invoke the {} tool", tool.name),
                authenticated: !tool.scopes.is_empty(),
            });
        }
        endpoints.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then_with(|| left.method.cmp(&right.method))
                .then_with(|| left.description.cmp(&right.description))
        });
        let tools = self.list_tools().into_iter().cloned().collect();
        AgentManifest {
            name: self.app_name.clone(),
            version: self.app_version.clone(),
            description: self.app_description.clone(),
            storage_mode: self.storage_mode.clone(),
            tools,
            jobs: {
                let mut jobs = self.jobs.clone();
                jobs.sort_by(|left, right| left.name.cmp(&right.name));
                jobs
            },
            endpoints,
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

impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistry")
            .field("tools", &self.tools)
            .field("handlers", &self.handlers.len())
            .field("jobs", &self.jobs.len())
            .field("endpoints", &self.endpoints.len())
            .field("app_name", &self.app_name)
            .field("app_version", &self.app_version)
            .field("app_description", &self.app_description)
            .field("storage_mode", &self.storage_mode)
            .finish()
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
    #[should_panic(expected = "duplicate agent tool registration")]
    fn test_register_tool_rejects_duplicate() {
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
    }

    struct EchoHandler;

    #[async_trait::async_trait]
    impl ToolHandler for EchoHandler {
        async fn execute(
            &self,
            _ctx: &ToolContext,
            input: serde_json::Value,
        ) -> Result<serde_json::Value, AppError> {
            Ok(input)
        }
    }

    struct ErrorHandler;

    #[async_trait::async_trait]
    impl ToolHandler for ErrorHandler {
        async fn execute(
            &self,
            _ctx: &ToolContext,
            _input: serde_json::Value,
        ) -> Result<serde_json::Value, AppError> {
            Err(AppError::new(ErrorKind::Internal, "handler blew up"))
        }
    }

    struct SlowHandler;

    #[async_trait::async_trait]
    impl ToolHandler for SlowHandler {
        async fn execute(
            &self,
            _ctx: &ToolContext,
            _input: serde_json::Value,
        ) -> Result<serde_json::Value, AppError> {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            Ok(serde_json::json!({}))
        }
    }

    fn echo_registry() -> ToolRegistry {
        let mut registry = ToolRegistry::default();
        registry.register_tool(ToolMetadata {
            name: "echo".to_string(),
            description: "Echo input".to_string(),
            input: serde_json::json!({"type": "object", "properties": {"msg": {"type": "string"}}, "required": ["msg"]}),
            output: serde_json::json!({"type": "object"}),
            scopes: vec![],
            effect: ToolEffect::Read,
            idempotent: true,
            enqueues_job: None,
            timeout: Some(10),
        });
        registry.register_handler("echo", EchoHandler);
        registry
    }

    #[tokio::test]
    async fn test_execute_tool_by_name() {
        let registry = echo_registry();
        let ctx = ToolContext::anonymous();
        let outcome = registry
            .execute("echo", serde_json::json!({"msg": "hi"}), &ctx, None)
            .await
            .unwrap();
        match outcome {
            ToolOutcome::Output(value) => assert_eq!(value["msg"], "hi"),
            _ => panic!("expected inline output"),
        }
    }

    #[tokio::test]
    async fn test_execute_unknown_tool_returns_not_found() {
        let registry = echo_registry();
        let ctx = ToolContext::anonymous();
        let err = registry
            .execute("nope", serde_json::json!({}), &ctx, None)
            .await
            .unwrap_err();
        assert_eq!(err.kind, ErrorKind::NotFound);
    }

    #[tokio::test]
    async fn test_execute_missing_handler_returns_not_impl() {
        let mut registry = ToolRegistry::default();
        registry.register_tool(sample_tool("no_handler"));
        let ctx = ToolContext::anonymous();
        let err = registry
            .execute("no_handler", serde_json::json!({}), &ctx, None)
            .await
            .unwrap_err();
        assert_eq!(err.kind, ErrorKind::NotImpl);
    }

    #[tokio::test]
    async fn test_execute_enforces_scopes() {
        let mut registry = ToolRegistry::default();
        registry.register_tool(ToolMetadata {
            name: "protected".to_string(),
            description: "Requires scope".to_string(),
            input: serde_json::json!({"type": "object"}),
            output: serde_json::json!({"type": "object"}),
            scopes: vec!["read:secret".to_string()],
            effect: ToolEffect::Read,
            idempotent: true,
            enqueues_job: None,
            timeout: None,
        });
        registry.register_handler("protected", EchoHandler);

        let err = registry
            .execute(
                "protected",
                serde_json::json!({}),
                &ToolContext::anonymous(),
                None,
            )
            .await
            .unwrap_err();
        assert_eq!(err.kind, ErrorKind::Authorization);

        let ctx = ToolContext::new("user-1", vec!["read:secret".to_string()]);
        assert!(
            registry
                .execute("protected", serde_json::json!({}), &ctx, None)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_execute_validates_input_schema() {
        let registry = echo_registry();
        let ctx = ToolContext::anonymous();
        let err = registry
            .execute("echo", serde_json::json!({"nope": 1}), &ctx, None)
            .await
            .unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
    }

    #[tokio::test]
    async fn test_execute_propagates_handler_error() {
        let mut registry = ToolRegistry::default();
        registry.register_tool(sample_tool("boom"));
        registry.register_handler("boom", ErrorHandler);
        let ctx = ToolContext::anonymous();
        let err = registry
            .execute("boom", serde_json::json!({}), &ctx, None)
            .await
            .unwrap_err();
        assert_eq!(err.kind, ErrorKind::Internal);
    }

    #[tokio::test]
    async fn test_execute_honors_timeout() {
        let mut registry = ToolRegistry::default();
        registry.register_tool(ToolMetadata {
            name: "slow".to_string(),
            description: "Slow tool".to_string(),
            input: serde_json::json!({"type": "object"}),
            output: serde_json::json!({"type": "object"}),
            scopes: vec![],
            effect: ToolEffect::Read,
            idempotent: true,
            enqueues_job: None,
            timeout: Some(1),
        });
        registry.register_handler("slow", SlowHandler);
        let ctx = ToolContext::anonymous();
        let err = registry
            .execute("slow", serde_json::json!({}), &ctx, None)
            .await
            .unwrap_err();
        assert_eq!(err.kind, ErrorKind::Timeout);
    }

    #[tokio::test]
    async fn test_execute_enqueues_job_when_no_handler() {
        let mut registry = ToolRegistry::default();
        registry.register_tool(ToolMetadata {
            name: "send_email".to_string(),
            description: "Enqueues a job".to_string(),
            input: serde_json::json!({"type": "object"}),
            output: serde_json::json!({"type": "object"}),
            scopes: vec![],
            effect: ToolEffect::Write,
            idempotent: false,
            enqueues_job: Some("email_queue".to_string()),
            timeout: None,
        });
        let backend = crate::MemoryThingdBackend::new();
        let ctx = ToolContext::anonymous();
        let outcome = registry
            .execute(
                "send_email",
                serde_json::json!({"to": "a@b.c"}),
                &ctx,
                Some(&backend),
            )
            .await
            .unwrap();
        match outcome {
            ToolOutcome::Enqueued { queue, job_id } => {
                assert_eq!(queue, "email_queue");
                assert!(!job_id.is_empty());
            }
            _ => panic!("expected enqueued outcome"),
        }
        let claimed = backend
            .claim_job("email_queue", "worker-1", 30)
            .await
            .unwrap();
        assert!(claimed.is_some());
    }

    #[tokio::test]
    async fn test_execute_enqueued_without_backend_errors() {
        let mut registry = ToolRegistry::default();
        registry.register_tool(ToolMetadata {
            name: "send_email".to_string(),
            description: "Enqueues a job".to_string(),
            input: serde_json::json!({"type": "object"}),
            output: serde_json::json!({"type": "object"}),
            scopes: vec![],
            effect: ToolEffect::Write,
            idempotent: false,
            enqueues_job: Some("email_queue".to_string()),
            timeout: None,
        });
        let ctx = ToolContext::anonymous();
        let err = registry
            .execute("send_email", serde_json::json!({}), &ctx, None)
            .await
            .unwrap_err();
        assert_eq!(err.kind, ErrorKind::NotImpl);
    }

    #[test]
    fn test_generate_manifest_advertises_invoke_endpoints() {
        let mut registry = ToolRegistry::default();
        registry.register_tool(sample_tool("read"));
        registry.register_tool(sample_tool("write"));

        let manifest = registry.generate_manifest();
        let invoke_paths: Vec<&str> = manifest
            .endpoints
            .iter()
            .filter(|e| e.method == "POST")
            .map(|e| e.path.as_str())
            .collect();
        assert!(invoke_paths.contains(&"/agent/tools/read"));
        assert!(invoke_paths.contains(&"/agent/tools/write"));
    }
}
