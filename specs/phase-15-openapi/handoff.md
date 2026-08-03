# Handoff

## OpenAPI API
- `OpenApiGenerator::new(title, version)` - create generator
- `OpenApiGenerator::generate(router)` - generate OpenAPI spec

## Usage
```rust
let generator = OpenApiGenerator::new("My API", "1.0.0")
    .with_description("My API description");

let openapi = generator.generate(&router);

let router = Router::new()
    .route("/openapi.json", get(|| async { Json(openapi) }))
    .route("/docs", get(swagger_ui));
```

## Schema Generation
- Uses serde for JSON Schema generation
- Derive macros generate schemas automatically
- Custom schemas via `#[schema(...)]` attribute

## Migration Guide
- Add `OpenApiGenerator` to AppState
- Expose `/openapi.json` endpoint
- Add Swagger UI via `/docs` endpoint
