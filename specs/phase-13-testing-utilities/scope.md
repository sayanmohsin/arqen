# Scope

TestApp provides a test server with memory adapters.
Mock auth is configurable (always succeeds, always fails, custom).
Fixtures provide test data (users, objects, events, jobs).
Assertions are ergonomic (assert_response!, assert_error!, etc.).
TestApp is async and can be used with tokio::test.
No real HTTP calls are made (all in-memory).
