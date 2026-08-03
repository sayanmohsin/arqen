# Acceptance Criteria

1. TestApp spins up a test server with memory adapters
2. Mock auth is configurable (success, failure, custom)
3. Fixtures provide test data (users, objects, events, jobs)
4. Assertions are ergonomic (assert_response!, assert_error!)
5. TestApp is async and works with tokio::test
6. No real HTTP calls are made
7. Tests verify TestApp, MockAuth, and fixtures
