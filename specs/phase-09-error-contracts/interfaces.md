# Interfaces

## ErrorCode

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCode {
    NotFound,
    Validation,
    Authentication,
    Authorization,
    Conflict,
    RateLimited,
    Internal,
    External,
    Unavailable,
}
```

## ErrorResponse

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    pub correlation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}
```

## CorrelationId

```rust
pub struct CorrelationId(pub String);

impl CorrelationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}
```

## ErrorContext

```rust
pub struct ErrorContext {
    pub correlation_id: CorrelationId,
    pub path: String,
    pub method: String,
}
```
