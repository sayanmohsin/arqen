//! Schema generation utilities for tool input/output definitions.
//!
//! # Unstable API
//!
//! The `SchemaGenerator` and `Schema` types are placeholders for future
//! integration with a JSON Schema generation library (e.g., `schemars`).
//! The current implementation provides basic helper methods but does not
//! generate real schemas from Rust types.
//!
//! Do not depend on these APIs in production code. They will change
//! significantly when real schema generation is implemented.

use serde_json::Value;

/// A placeholder schema generator.
///
/// See [module docs](self) for why this is unstable.
pub struct SchemaGenerator;

impl SchemaGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Build a JSON Schema object from explicit property definitions.
    ///
    /// `properties` is a list of `(name, type_name)` pairs where `type_name`
    /// is one of `"string"`, `"number"`, `"boolean"`, `"array"`, or
    /// anything else (treated as `"object"`).
    pub fn object_schema(properties: Vec<(&str, &str)>) -> Value {
        let props: serde_json::Map<String, Value> = properties
            .into_iter()
            .map(|(name, type_name)| {
                let type_str = match type_name {
                    "string" => "string",
                    "number" => "number",
                    "boolean" => "boolean",
                    "array" => "array",
                    _ => "object",
                };
                (name.to_string(), serde_json::json!({ "type": type_str }))
            })
            .collect();

        serde_json::json!({
            "type": "object",
            "properties": props,
        })
    }
}

impl Default for SchemaGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can describe their JSON Schema.
///
/// This is a placeholder. A real implementation would derive schemas
/// using `schemars::JsonSchema`.
pub trait Schema {
    fn schema() -> Value;
}
