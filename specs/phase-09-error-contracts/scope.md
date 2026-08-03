# Scope

Error responses follow a consistent JSON format with stable error codes.
Correlation IDs are generated per-request and included in all responses.
Internal error details (stack traces, database errors) are redacted.
Error context is propagated through the call stack without losing information.
Error codes are stable across versions (additive, never breaking).
