# Phase 11: Request Validation

Objective: provide typed extractors and derive-based validation for request data.

Dependencies: 08, 09.

In scope: validation extractors, derive macros, rejection types, error formatting, and integration with error contracts.

Out of scope: auth validation (Phase 10), business logic validation.

Deliverables: `validation.rs` module, derive macros, rejection types, and tests.

Acceptance: requests are validated at the extractor level; validation errors follow error contract format; derive macros generate validation code; rejections are typed and informative.

Tests: validation extractors, derive macros, rejection types, error formatting.

Docs: update validation guide.

Handoff: record validation API, derive macros, and rejection types.
