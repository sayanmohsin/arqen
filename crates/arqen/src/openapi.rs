//! OpenAPI documentation module for Arqen.
//!
//! Provides OpenAPI 3.0 spec generation with Swagger UI support.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// OpenAPI 3.0 specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: OpenApiInfo,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub paths: HashMap<String, PathItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiInfo {
    pub title: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub operation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub responses: HashMap<String, Response>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, MediaType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    pub schema: Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    #[serde(rename = "type")]
    pub schema_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Schema>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    pub description: String,
    pub content: HashMap<String, MediaType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub schema: Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub schemas: HashMap<String, Schema>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub security_schemes: HashMap<String, SecurityScheme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SecurityScheme {
    #[serde(rename = "http")]
    Http {
        scheme: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        bearer_format: Option<String>,
    },
    #[serde(rename = "apiKey")]
    ApiKey {
        #[serde(rename = "in")]
        location: String,
        name: String,
    },
}

/// Builder for OpenAPI specifications.
pub struct OpenApiGenerator {
    title: String,
    version: String,
    description: Option<String>,
    license: Option<License>,
    paths: HashMap<String, PathItem>,
    tags: Vec<Tag>,
    components: Components,
    security: Vec<HashMap<String, Vec<String>>>,
}

impl OpenApiGenerator {
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            version: version.into(),
            description: None,
            license: None,
            paths: HashMap::new(),
            tags: Vec::new(),
            components: Components {
                schemas: HashMap::new(),
                security_schemes: HashMap::new(),
            },
            security: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_license(mut self, name: impl Into<String>, url: Option<String>) -> Self {
        self.license = Some(License {
            name: name.into(),
            url,
        });
        self
    }

    pub fn with_bearer_auth(mut self) -> Self {
        self.components.security_schemes.insert(
            "bearerAuth".to_string(),
            SecurityScheme::Http {
                scheme: "bearer".to_string(),
                bearer_format: Some("JWT".to_string()),
            },
        );
        self.security
            .push([("bearerAuth".to_string(), vec![])].into());
        self
    }

    pub fn with_api_key_auth(mut self, header_name: impl Into<String>) -> Self {
        self.components.security_schemes.insert(
            "apiKeyAuth".to_string(),
            SecurityScheme::ApiKey {
                location: "header".to_string(),
                name: header_name.into(),
            },
        );
        self.security
            .push([("apiKeyAuth".to_string(), vec![])].into());
        self
    }

    pub fn add_tag(mut self, name: impl Into<String>, description: Option<String>) -> Self {
        self.tags.push(Tag {
            name: name.into(),
            description,
        });
        self
    }

    pub fn add_get(
        mut self,
        path: impl Into<String>,
        operation_id: impl Into<String>,
        summary: Option<String>,
        tags: Vec<String>,
    ) -> Self {
        let path_str = path.into();
        let item = self.paths.entry(path_str).or_insert_with(|| PathItem {
            get: None,
            post: None,
            put: None,
            delete: None,
            patch: None,
        });
        item.get = Some(Operation {
            operation_id: operation_id.into(),
            summary,
            description: None,
            tags,
            responses: default_responses(),
            security: None,
            request_body: None,
            parameters: Vec::new(),
        });
        self
    }

    pub fn add_post(
        mut self,
        path: impl Into<String>,
        operation_id: impl Into<String>,
        summary: Option<String>,
        tags: Vec<String>,
        request_body: Option<RequestBody>,
    ) -> Self {
        let path_str = path.into();
        let item = self.paths.entry(path_str).or_insert_with(|| PathItem {
            get: None,
            post: None,
            put: None,
            delete: None,
            patch: None,
        });
        item.post = Some(Operation {
            operation_id: operation_id.into(),
            summary,
            description: None,
            tags,
            responses: default_responses(),
            security: None,
            request_body,
            parameters: Vec::new(),
        });
        self
    }

    pub fn add_schema(mut self, name: impl Into<String>, schema: Schema) -> Self {
        self.components.schemas.insert(name.into(), schema);
        self
    }

    pub fn build(self) -> OpenApiSpec {
        OpenApiSpec {
            openapi: "3.0.3".to_string(),
            info: OpenApiInfo {
                title: self.title,
                version: self.version,
                description: self.description,
                license: self.license,
            },
            paths: self.paths,
            components: if self.components.schemas.is_empty()
                && self.components.security_schemes.is_empty()
            {
                None
            } else {
                Some(self.components)
            },
            security: if self.security.is_empty() {
                None
            } else {
                Some(self.security)
            },
            tags: if self.tags.is_empty() {
                None
            } else {
                Some(self.tags)
            },
        }
    }
}

fn default_responses() -> HashMap<String, Response> {
    let mut responses = HashMap::new();
    responses.insert(
        "200".to_string(),
        Response {
            description: "Success".to_string(),
            content: None,
        },
    );
    responses.insert(
        "400".to_string(),
        Response {
            description: "Bad Request".to_string(),
            content: None,
        },
    );
    responses.insert(
        "401".to_string(),
        Response {
            description: "Unauthorized".to_string(),
            content: None,
        },
    );
    responses.insert(
        "500".to_string(),
        Response {
            description: "Internal Server Error".to_string(),
            content: None,
        },
    );
    responses
}

/// Generate the default Arqen OpenAPI spec.
pub fn default_spec() -> OpenApiSpec {
    OpenApiGenerator::new("Arqen API", env!("CARGO_PKG_VERSION"))
        .with_description("Backend infrastructure for agent-ready applications")
        .add_tag("health", Some("Health checks".to_string()))
        .add_tag("agent", Some("Agent operations".to_string()))
        .add_tag("docs", Some("Documentation".to_string()))
        .add_get("/health", "health_check", Some("Liveness check".to_string()), vec!["health".to_string()])
        .add_get("/ready", "readiness_check", Some("Readiness check".to_string()), vec!["health".to_string()])
        .add_get("/agent", "agent_info", Some("Agent description".to_string()), vec!["agent".to_string()])
        .add_get("/agent/manifest", "agent_manifest", Some("Agent manifest".to_string()), vec!["agent".to_string()])
        .add_get("/docs", "api_docs", Some("API documentation".to_string()), vec!["docs".to_string()])
        .build()
}

/// Swagger UI HTML page.
pub fn swagger_ui_html(spec_url: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Arqen API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5.10.5/swagger-ui.css">
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.10.5/swagger-ui-bundle.js"></script>
    <script>
        SwaggerUIBundle({{
            url: "{}",
            dom_id: '#swagger-ui',
            presets: [
                SwaggerUIBundle.presets.apis,
                SwaggerUIBundle.SwaggerUIStandalonePreset
            ],
            layout: "BaseLayout"
        }});
    </script>
</body>
</html>"#,
        spec_url
    )
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
        let spec = OpenApiGenerator::new("Test API", "1.0.0")
            .with_description("A test API")
            .build();
        assert_eq!(spec.info.description, Some("A test API".to_string()));
    }

    #[test]
    fn test_generator_with_license() {
        let spec = OpenApiGenerator::new("Test API", "1.0.0")
            .with_license("MIT", Some("https://opensource.org/licenses/MIT".to_string()))
            .build();
        assert!(spec.info.license.is_some());
        assert_eq!(spec.info.license.unwrap().name, "MIT");
    }

    #[test]
    fn test_generator_with_bearer_auth() {
        let spec = OpenApiGenerator::new("Test API", "1.0.0")
            .with_bearer_auth()
            .build();
        assert!(spec.security.is_some());
        assert!(spec.components.is_some());
    }

    #[test]
    fn test_generator_add_get() {
        let spec = OpenApiGenerator::new("Test API", "1.0.0")
            .add_get("/users", "list_users", Some("List users".to_string()), vec!["users".to_string()])
            .build();
        assert!(spec.paths.contains_key("/users"));
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

    #[test]
    fn test_swagger_ui_html() {
        let html = swagger_ui_html("/api-docs/openapi.json");
        assert!(html.contains("swagger-ui"));
        assert!(html.contains("/api-docs/openapi.json"));
    }
}
