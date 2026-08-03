# Handoff

## Auth Adapters
- `ApiKeyAuth` - header-based API key validation
- `JwtAuth` - JWT token validation with configurable secret
- `SessionAuth` - cookie-based session validation

## Policy Traits
- `Policy::check(&self, context, resource) -> Result<(), AuthError>`
- Built-in: `RequireRole`, `RequirePermission`

## Usage
```rust
let auth = RequireAuth::new(Arc::new(JwtAuth::new(secret)));
let policy = RequirePolicy::new(Arc::new(RequireRole("admin".into())));

let router = Router::new()
    .route("/protected", get(handler))
    .layer(auth)
    .layer(policy);
```

## Migration Guide
- Add `AuthContext` to handler extractors
- Use `RequireAuth` middleware for protected routes
- Implement custom policies via `Policy` trait
