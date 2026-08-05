# Handoff

## Validation API
- `Validated<T>` extractor for request validation
- `#[derive(Validate)]` for struct validation
- Built-in validators: required, min, max, regex, email, url

## Usage

```rust
async fn create_user(
    Validated(input): Validated<CreateUser>,
) -> Json<Value> { ... }
```

## Derive Macros

```rust
#[derive(Validate, Deserialize)]
struct CreateUser {
    #[validate(required, email)]
    email: Option<String>,
    
    #[validate(required, min_length = 3)]
    name: Option<String>,
}
```

## Migration Guide
- Replace manual validation in handlers with `Validated<T>` extractor
- Use derive macros for struct validation
- Validation errors follow error contract (Phase 09)
