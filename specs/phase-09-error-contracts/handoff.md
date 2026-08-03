# Handoff

## Error Codes
- NotFound (404), Validation (400), Authentication (401), Authorization (403)
- Conflict (409), RateLimited (429), Internal (500), External (502), Unavailable (503)

## Response Format
```json
{
  "error": {
    "code": "validation",
    "message": "Invalid email format",
    "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
    "details": null
  }
}
```

## Correlation ID Strategy
- Generated per-request (UUID v4)
- Returned in X-Request-Id response header
- Included in error response body
- Logged with tracing span

## Migration Guide
- Replace `AppError` responses with `ErrorResponse`
- Use `ErrorCode` instead of `ErrorKind`
- Correlation IDs are automatic via middleware
