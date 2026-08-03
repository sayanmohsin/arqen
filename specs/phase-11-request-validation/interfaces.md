# Interfaces

## Validate Trait

```rust
pub trait Validate {
    fn validate(&self) -> Result<(), ValidationErrors>;
}
```

## Validated Extractor

```rust
pub struct Validated<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for Validated<T>
where
    T: Validate + DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ValidationRejection;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> { ... }
}
```

## Derive Macro

```rust
#[derive(Validate, Deserialize)]
struct CreateUser {
    #[validate(required, email)]
    email: Option<String>,
    
    #[validate(required, min_length = 3, max_length = 50)]
    name: Option<String>,
}
```

## ValidationRejection

```rust
pub struct ValidationRejection {
    pub errors: Vec<FieldError>,
}

pub struct FieldError {
    pub field: String,
    pub code: String,
    pub message: String,
}
```
