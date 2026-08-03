# Tasks

- [ ] Define `AuthContext` struct with user identity and claims
- [ ] Create `Authentication` trait for adapter pattern
- [ ] Implement `ApiKeyAuth` adapter (header-based)
- [ ] Implement `JwtAuth` adapter (token validation)
- [ ] Implement `SessionAuth` adapter (cookie-based)
- [ ] Create `Policy` trait for authorization
- [ ] Create `RequireAuth` middleware that extracts AuthContext
- [ ] Create `RequirePolicy` middleware for authorization
- [ ] Add auth error responses (401, 403)
- [ ] Add auth tests (middleware, adapters, policies)
- [ ] Update examples to use authentication
