# Acceptance Criteria

1. AppConfig loads from env vars with `ARQEN_` prefix
2. Config files are optional (env vars override file values)
3. Secrets are wrapped in `Secret<T>` and redacted in Display/Debug
4. Config validation produces clear, actionable error messages
5. AppState is constructed via builder, not struct literals
6. Storage adapter is selected via config (memory/persistent/http)
7. All existing tests pass with new config system
8. Examples compile and run with AppConfig
