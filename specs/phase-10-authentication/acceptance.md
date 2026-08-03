# Acceptance Criteria

1. AuthContext is extracted from requests via middleware
2. API keys are validated via configurable header
3. JWT tokens are validated with configurable secret
4. Sessions are validated via cookie or header
5. Authorization is policy-based (role, permission, custom)
6. Unauthenticated requests receive 401
7. Unauthorized requests receive 403
8. All auth adapters are pluggable via trait
9. Tests verify middleware, adapters, and policies
