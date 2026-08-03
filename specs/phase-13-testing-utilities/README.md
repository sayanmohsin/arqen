# Phase 13: Testing Utilities

Objective: provide TestApp/TestState with memory adapters and mock auth for testing.

Dependencies: 08, 09, 10, 11, 12.

In scope: TestApp builder, memory adapters, mock auth, fixture helpers, and test assertions.

Out of scope: performance testing, load testing.

Deliverables: `testutil.rs` module, TestApp builder, memory adapters, mock auth, and tests.

Acceptance: TestApp spins up a test server with memory adapters; mock auth is configurable; fixtures provide test data; assertions are ergonomic.

Tests: TestApp construction, memory adapters, mock auth, fixture helpers.

Docs: update testing guide.

Handoff: record TestApp API, memory adapters, and fixture helpers.
