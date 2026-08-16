# Standards

Coding and documentation standards for the Arqen project.

## API design

- Use descriptive names: `register_tool`, `check_readiness`, `create_router`.
- Document all public items with `///` doc comments.
- Gate unstable or advanced features behind Cargo feature flags.
- Keep the public API surface small; prefer internal modules for
  implementation details.
- Use `#[serde(rename_all = "snake_case")]` for JSON field names.

## Errors

- Use `AppError` with `ErrorKind` for structured error handling.
- Attach `ErrorContext` with the operation name, affected entity, and
  reason when helpful.
- Never expose internal system errors to API responses; map them to
  appropriate `ErrorKind` variants.
- Use `From` impls to convert framework errors (`reqwest::Error`,
  `std::io::Error`) into `AppError`.

```rust
use arqen::core::{AppError, ErrorKind};

fn do_work() -> Result<(), AppError> {
    Err(AppError::new(ErrorKind::Internal, "something went wrong"))
}
```

## Logging

- Use `tracing` macros: `info!`, `warn!`, `error!`, `debug!`, `trace!`.
- Use structured fields, not format strings in the message:

```rust
tracing::info!(user_id = %user.id, action = "login", "user logged in");
```

- Never log secrets, JWT tokens, API keys, or passwords.
- Use `redacted` fields for sensitive data that needs to appear in logs:

```rust
tracing::info!(api_key = %redacted(&key), "key validated");
```

- Prefer `json` format in production, `pretty` in development.

## Secrets

- Wrap sensitive values in `Secret<T>` from `arqen::config`.
- `Secret<T>` implements `Display` and `Debug` as `[REDACTED]`.
- Use constant-time comparison for API keys (via the `subtle` crate).
- Never commit secrets to version control.
- Store secrets in environment variables or a secrets manager, not in
  config files checked into git.

## Tests

- **Unit tests**: in the same module, behind `#[cfg(test)] mod tests`.
- **Integration tests**: in the `tests/` directory at the crate root.
- **Contract tests**: verify public API stability against the published
  interface.
- Test both success and error paths.
- Use `tokio::test` for async tests.
- Use `HealthRegistry` and `ModuleBuilder` test helpers for health and
  module composition tests.

## Compatibility

- Follow semver: minor bumps for new features, patch for bug fixes,
  major for breaking changes.
- Feature flags must be additive: enabling a feature should never
  break existing code.
- Maintain adapter parity between in-memory and durable backends.
- Document any deviation from the public thingd HTTP contract.
- Keep native Thingd dependencies behind an explicit feature and publish the
  tested native version separately from the HTTP API version.

## Changelog

Update `CHANGELOG.md` for every user-facing change:

- New features
- Bug fixes
- Breaking changes
- Deprecations

Use [Conventional Commits](https://www.conventionalcommits.org/) format
for commit messages and changelog entries.

## Performance

- Benchmark before and after performance changes.
- Record evidence: throughput, latency percentiles, memory usage.
- Use `criterion` or similar for micro-benchmarks.
- Use `tracing` spans for production performance visibility.
- Do not optimize without measurement; prefer clear code first.

## Review checklists

### API changes

- [ ] Public API documented
- [ ] Feature-gated if unstable
- [ ] Semver impact noted
- [ ] CHANGELOG updated
- [ ] Tests added for new public items

### Performance changes

- [ ] Baseline benchmark recorded
- [ ] After benchmark recorded
- [ ] No regression in default path
- [ ] Evidence linked in commit or PR

### Documentation changes

- [ ] Accurate and up-to-date
- [ ] Code examples compile and run
- [ ] Links resolve correctly
- [ ] No claims about unfinished features
