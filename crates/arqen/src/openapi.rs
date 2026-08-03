//! OpenAPI documentation module for Arqen.
//!
//! Provides static OpenAPI spec generation.

use serde::{Deserialize, Serialize};

/// OpenAPI 3.0 specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: OpenApiInfo,
    pub paths: Vec<OpenApiPath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<OpenApiComponents>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiInfo {
    pub title: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiPath {
    pub path: String,
    pub method: String,
    pub operation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub responses: Vec<OpenApiResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiResponse {
    pub status: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiComponents {
    pub schemas: Vec<OpenApiSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSchema {
    pub name: String,
    pub schema: serde_json::Value,
}

/// Builder for OpenAPI specifications.
pub struct OpenApiGenerator {
    title: String,
    version: String,
    description: Option<String>,
    paths: Vec<OpenApiPath>,
}

impl OpenApiGenerator {
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            version: version.into(),
            description: None,
            paths: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn add_path(
        mut self,
        path: impl Into<String>,
        method: impl Into<String>,
        operation_id: impl Into<String>,
        summary: Option<String>,
        tags: Vec<String>,
    ) -> Self {
        self.paths.push(OpenApiPath {
            path: path.into(),
            method: method.into(),
            operation_id: operation_id.into(),
            summary,
            description: None,
            tags,
            responses: vec![
                OpenApiResponse {
                    status: "200".to_string(),
                    description: "Success".to_string(),
                },
                OpenApiResponse {
                    status: "401".to_string(),
                    description: "Unauthorized".to_string(),
                },
                OpenApiResponse {
                    status: "500".to_string(),
                    description: "Internal Server Error".to_string(),
                },
            ],
        });
        self
    }

    pub fn build(self) -> OpenApiSpec {
        OpenApiSpec {
            openapi: "3.0.3".to_string(),
            info: OpenApiInfo {
                title: self.title,
                version: self.version,
                description: self.description,
            },
            paths: self.paths,
            components: None,
        }
    }
}

/// Generate the default Arqen OpenAPI spec.
pub fn default_spec() -> OpenApiSpec {
    OpenApiGenerator::new("Arqen API", env!("CARGO_PKG_VERSION"))
        .with_description("Backend infrastructure for agent-ready applications")
        .add_path("/health", "GET", "health", Some("Liveness check".to_string()), vec!["health".to_string()])
        .add_path("/ready", "GET", "readiness", Some("Readiness check".to_string()), vec!["health".to_string()])
        .add_path("/agent", "GET", "agent", Some("Agent description".to_string()), vec!["agent".to_string()])
        .add_path("/agent/manifest", "GET", "agent_manifest", Some("Agent manifest".to_string()), vec!["agent".to_string()])
        .add_path("/docs", "GET", "docs", Some("API documentation".to_string()), vec!["docs".to_string()])
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_new() {
        let generator = OpenApiGenerator::new("Test API", "1.0.0");
        let spec = generator.build();
        assert_eq!(spec.openapi, "3.0.3");
        assert_eq!(spec.info.title, "Test API");
        assert_eq!(spec.info.version, "1.0.0");
    }

    #[test]
    fn test_generator_with_description() {
        let generator = OpenApiGenerator::new("Test API", "1.0.0")
            .with_description("A test API");
        let spec = generator.build();
        assert_eq!(spec.info.description, Some("A test API".to_string()));
    }

    #[test]
    fn test_generator_add_path() {
        let generator = OpenApiGenerator::new("Test API", "1.0.0")
            .add_path("/users", "GET", "list_users", Some("List users".to_string()), vec!["users".to_string()]);
        let spec = generator.build();
        assert_eq!(spec.paths.len(), 1);
        assert_eq!(spec.paths[0].path, "/users");
        assert_eq!(spec.paths[0].method, "GET");
    }

    #[test]
    fn test_default_spec() {
        let spec = default_spec();
        assert_eq!(spec.openapi, "3.0.3");
        assert_eq!(spec.info.title, "Arqen API");
        assert!(!spec.paths.is_empty());
    }

    #[test]
    fn test_spec_serialization() {
        let spec = default_spec();
        let json = serde_json::to_string_pretty(&spec).unwrap();
        assert!(json.contains("3.0.3"));
        assert!(json.contains("Arqen API"));
    }
}
