# Test Plan

## Unit Tests
- Config loading from env vars
- Config loading from file
- Config validation (missing required fields, invalid values)
- Secret redaction in Display/Debug
- AppState builder construction
- Storage adapter selection
- Feature flag combinations

## Integration Tests
- End-to-end config loading and AppState construction
- Config override (env > file > defaults)
- Secret handling in error messages

## Manual Verification
- Examples compile and run with AppConfig
- Secrets do not appear in logs
