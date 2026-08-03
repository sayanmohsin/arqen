# Scope

Authentication is pluggable via adapter pattern.
API keys are validated via header (configurable header name).
JWT tokens are validated with configurable secret/algorithms.
Sessions are validated via cookie or header.
AuthContext is extracted from requests and available in handlers.
Authorization is policy-based (role, permission, custom).
Unauthenticated requests receive 401; unauthorized requests receive 403.
