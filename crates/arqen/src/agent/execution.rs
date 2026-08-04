//! Tool execution boundary.
//!
//! Tools are described by [`ToolMetadata`](crate::ToolMetadata) and executed
//! through a registered [`ToolHandler`]. The [`ToolContext`] carries the
//! caller's identity and granted scopes so that scope checks and schema
//! validation happen before any handler runs.

use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;

use crate::auth::AuthContext;
use crate::core::{AppError, ErrorKind};

/// The caller context passed to a tool handler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContext {
    /// Authenticated subject (user ID, API key ID, etc.).
    pub subject: String,
    /// Scopes granted to the caller.
    pub scopes: Vec<String>,
}

impl ToolContext {
    /// Create a context for an unauthenticated caller.
    pub fn anonymous() -> Self {
        Self {
            subject: "anonymous".to_string(),
            scopes: Vec::new(),
        }
    }

    /// Create a context for an explicit subject and set of scopes.
    pub fn new(subject: impl Into<String>, scopes: Vec<String>) -> Self {
        Self {
            subject: subject.into(),
            scopes,
        }
    }

    /// Derive a context from an [`AuthContext`].
    ///
    /// Scopes are read from the `scopes` claim (an array of strings).
    pub fn from_auth_context(ctx: &AuthContext) -> Self {
        let scopes = ctx
            .get_claim("scopes")
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        Self {
            subject: ctx.subject.clone(),
            scopes,
        }
    }
}

/// The result of executing a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolOutcome {
    /// The tool produced a value inline.
    Output(serde_json::Value),
    /// The tool enqueued a background job instead of executing inline.
    Enqueued { queue: String, job_id: String },
}

/// Handler for a single agent tool.
///
/// Implement this trait and register the handler with
/// [`ToolRegistry::register_handler`](crate::ToolRegistry::register_handler).
/// Input validation and scope enforcement run before the handler is invoked.
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Execute the tool with validated input.
    ///
    /// Return the tool output or an [`AppError`]. Errors are redacted before
    /// being returned over HTTP.
    async fn execute(
        &self,
        ctx: &ToolContext,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, AppError>;
}

/// Validate a value against a hand-written JSON Schema.
///
/// Supports the subset used by Arqen tool schemas: `type`, `required`, and
/// nested `properties`/`items`. Unknown schema keywords are ignored.
pub fn validate_against_schema(
    value: &serde_json::Value,
    schema: &serde_json::Value,
) -> Result<(), AppError> {
    if !schema.is_object() {
        return Ok(());
    }

    if let Some(expected_type) = schema.get("type").and_then(serde_json::Value::as_str) {
        let valid = match expected_type {
            "object" => value.is_object(),
            "string" => value.is_string(),
            "number" => value.is_number(),
            "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
            "boolean" => value.is_boolean(),
            "array" => value.is_array(),
            "null" => value.is_null(),
            _ => true,
        };
        if !valid {
            return Err(AppError::new(
                ErrorKind::Validation,
                format!("expected value of type {expected_type}"),
            ));
        }
    }

    if let Some(object) = value.as_object() {
        if let Some(required) = schema.get("required").and_then(serde_json::Value::as_array) {
            for required_field in required {
                if let Some(field) = required_field.as_str()
                    && !object.contains_key(field)
                {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        format!("missing required field: {field}"),
                    ));
                }
            }
        }
        if let Some(properties) = schema
            .get("properties")
            .and_then(serde_json::Value::as_object)
        {
            for (field, field_schema) in properties {
                if let Some(field_value) = object.get(field) {
                    validate_against_schema(field_value, field_schema)?;
                }
            }
        }
    }

    if let Some(array) = value.as_array()
        && let Some(items_schema) = schema.get("items")
    {
        for item in array {
            validate_against_schema(item, items_schema)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_against_schema_accepts_valid_object() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "id": {"type": "string"},
                "count": {"type": "integer"}
            },
            "required": ["id"]
        });
        let value = serde_json::json!({"id": "abc", "count": 3});
        assert!(validate_against_schema(&value, &schema).is_ok());
    }

    #[test]
    fn test_validate_against_schema_rejects_missing_required() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {"id": {"type": "string"}},
            "required": ["id"]
        });
        let value = serde_json::json!({});
        let err = validate_against_schema(&value, &schema).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
    }

    #[test]
    fn test_validate_against_schema_rejects_wrong_type() {
        let schema =
            serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}}});
        let value = serde_json::json!({"id": 42});
        let err = validate_against_schema(&value, &schema).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
    }

    #[test]
    fn test_validate_against_schema_ignores_non_object_schema() {
        assert!(validate_against_schema(&serde_json::json!(1), &serde_json::json!(true)).is_ok());
    }

    #[test]
    fn test_tool_context_from_auth_context_reads_scopes_claim() {
        let ctx = AuthContext::new("user-1", "test")
            .with_claim("scopes", serde_json::json!(["read:users", "write:users"]));
        let tool_ctx = ToolContext::from_auth_context(&ctx);
        assert_eq!(tool_ctx.subject, "user-1");
        assert_eq!(
            tool_ctx.scopes,
            vec!["read:users".to_string(), "write:users".to_string()]
        );
    }

    #[test]
    fn test_tool_context_anonymous() {
        let ctx = ToolContext::anonymous();
        assert_eq!(ctx.subject, "anonymous");
        assert!(ctx.scopes.is_empty());
    }
}
