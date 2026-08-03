# Tasks

- [ ] Create `config.rs` module with `AppConfig` struct
- [ ] Implement env var loading with `ARQEN_` prefix
- [ ] Implement optional file loading (TOML/YAML)
- [ ] Add config validation with clear error messages
- [ ] Create `Secret<T>` wrapper that redacts in Display/Debug
- [ ] Create `AppState` struct with builder pattern
- [ ] Wire storage adapter selection (memory/persistent/http)
- [ ] Add `AppState::builder()` with validation
- [ ] Update existing code to use AppConfig instead of hardcoded values
- [ ] Add config tests (loading, validation, redaction, defaults)
- [ ] Update examples to use AppConfig
