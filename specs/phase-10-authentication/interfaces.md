# Interfaces

## AuthContext

```rust
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub subject: String,
    pub claims: HashMap<String, serde_json::Value>,
    pub adapter: String,
}
```

## Authentication Trait

```rust
#[async_trait]
pub trait Authentication: Send + Sync {
    async fn authenticate(&self, request: &HeaderMap) -> Result<AuthContext, AuthError>;
}
```

## Policy Trait

```rust
pub trait Policy: Send + Sync {
    fn check(&self, context: &AuthContext, resource: &str) -> Result<(), AuthError>;
}
```

## Auth Error

```rust
pub enum AuthError {
    Missing,
    Invalid,
    Expired,
    Unauthorized(String),
}
```

## RequireAuth Middleware

```rust
pub struct RequireAuth {
    adapter: Arc<dyn Authentication>,
}

impl<S> Layer<S> for RequireAuth {
    type Service = RequireAuthService<S>;
    fn layer(&self, inner: S) -> Self::Service { ... }
}
```
