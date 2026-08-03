# Phase 09: Error Contracts

Objective: establish stable, predictable error responses with correlation IDs and consistent mapping to HTTP status codes.

Dependencies: 08.

In scope: error codes, correlation IDs, redacted error responses, consistent IntoResponse, error context propagation.

Out of scope: auth errors (Phase 10), validation errors (Phase 11).

Deliverables: `error.rs` module with ErrorCode enum, correlation ID middleware, error response types, and tests.

Acceptance: all errors map to stable HTTP status codes; correlation IDs are included in responses; internal errors are redacted; error messages are consistent across the API.

Tests: error mapping, correlation ID propagation, redaction, response format.

Docs: update error handling guide.

Handoff: record error codes, response format, and correlation ID strategy.
