# Acceptance Criteria

1. All errors map to stable HTTP status codes via ErrorCode
2. Correlation IDs are included in all error responses
3. Internal error details are redacted (no stack traces, DB errors)
4. Error responses follow consistent JSON format
5. Correlation IDs are generated per-request and propagated
6. Error codes are stable across versions
7. All handlers use new error types
8. Tests verify format, redaction, and correlation ID propagation
