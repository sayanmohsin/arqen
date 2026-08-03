# Test Plan

## Unit Tests
- ErrorCode to HTTP status mapping
- ErrorResponse JSON format
- Secret redaction in error messages
- Correlation ID generation and propagation

## Integration Tests
- Error responses include correlation ID
- Internal errors are redacted
- Error format is consistent across handlers

## Manual Verification
- Error responses match documented format
- Correlation IDs appear in logs
