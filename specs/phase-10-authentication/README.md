# Phase 10: Authentication

Objective: provide pluggable authentication with API keys, JWT/session adapters, and policy traits for authorization.

Dependencies: 08, 09.

In scope: AuthContext, authentication middleware, API key adapter, JWT adapter, session adapter, policy traits, and authorization middleware.

Out of scope: OAuth2 flows, identity provider implementation, user management.

Deliverables: `auth.rs` module, authentication middleware, adapter implementations, policy traits, and tests.

Acceptance: requests are authenticated via configurable adapters; AuthContext is available in handlers; authorization is policy-based; unauthenticated requests receive 401; unauthorized requests receive 403.

Tests: authentication middleware, adapter implementations, policy evaluation, error responses.

Docs: update authentication guide.

Handoff: record auth adapters, policy traits, and integration examples.
