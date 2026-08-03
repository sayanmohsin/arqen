# Tasks

- [ ] Define `ErrorCode` enum with stable codes (NotFound, Validation, Auth, Internal, etc.)
- [ ] Create `ErrorResponse` struct with code, message, correlation_id, details
- [ ] Implement `From<AppError>` for `ErrorResponse` with redaction
- [ ] Add correlation ID middleware (X-Request-Id header)
- [ ] Implement `ErrorContext` for propagating context through call stack
- [ ] Add error response tests (format, redaction, correlation ID)
- [ ] Update all handlers to use new error types
- [ ] Document error codes and response format
