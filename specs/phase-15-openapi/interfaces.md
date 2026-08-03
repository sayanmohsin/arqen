# Interfaces

## OpenApiGenerator

```rust
pub struct OpenApiGenerator {
    title: String,
    version: String,
    description: Option<String>,
}

impl OpenApiGenerator {
    pub fn new(title: &str, version: &str) -> Self { ... }
    pub fn with_description(mut self, desc: &str) -> Self { ... }
    pub fn generate(&self, router: &Router) -> OpenApi { ... }
}
```

## OpenApi

```rust
pub struct OpenApi {
    pub openapi: String,
    pub info: Info,
    pub paths: Paths,
    pub components: Option<Components>,
}

pub struct Info {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}
```
