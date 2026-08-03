# Test Plan

## Unit Tests
- AuthContext extraction from request
- API key validation
- JWT token validation
- Session validation
- Policy evaluation

## Integration Tests
- Unauthenticated request returns 401
- Unauthorized request returns 403
- AuthContext is available in handlers
- Multiple auth adapters work together

## Manual Verification
- Auth middleware works with all adapters
- Error responses follow error contract (Phase 09)
