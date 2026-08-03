# Handoff

## Config Schema
- See `interfaces.md` for full type definitions
- Env vars: `ARQEN_HOST`, `ARQEN_PORT`, `ARQEN_STORAGE_MODE`, `ARQEN_LOG_LEVEL`, `ARQEN_LOG_FORMAT`
- Secrets: `ARQEN_JWT_SECRET`, `ARQEN_THINGD_TOKEN`

## Public APIs
- `AppConfig::from_env()` - load from environment
- `AppConfig::from_file(path)` - load from file
- `AppState::builder()` - construct app state

## Migration Guide
- Replace `RuntimeInfo::new(storage, registry)` with `AppState::builder().with_config(config).build()`
- Replace `create_router_with_runtime(runtime)` with `create_router(app_state)`
- Use `Secret<String>` for tokens and keys

## Limitations
- File config supports TOML only (YAML in future)
- No hot-reload (restart required)
- No config encryption (use env vars for secrets)
