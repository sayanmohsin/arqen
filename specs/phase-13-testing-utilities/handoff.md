# Handoff

## TestApp API
- `TestApp::builder()` - construct test app
- `TestApp::request(method, path)` - make HTTP request
- `TestApp::state()` - access AppState

## MockAuth
- `MockAuth::always_success(context)` - always authenticate
- `MockAuth::always_fail(error)` - always fail
- `MockAuth::custom(fn)` - custom behavior

## Fixtures
- `Fixtures::new(storage)` - create fixture helper
- `Fixtures::create_user(name, email)` - create test user
- `Fixtures::create_event(stream, event_type)` - create test event
- `Fixtures::create_job(queue, payload)` - create test job

## Assertions
- `assert_response!(response, status)` - assert HTTP status
- `assert_response!(response, status, body)` - assert status and body
- `assert_error!(response, code)` - assert error code

## Usage

```rust
#[tokio::test]
async fn test_create_user() {
    let app = TestApp::builder().build();
    let fixtures = Fixtures::new(app.state().storage.clone());
    
    let response = app.request(Method::POST, "/users")
        .json(&json!({"name": "Alice", "email": "alice@example.com"}))
        .send()
        .await;
    
    assert_response!(response, StatusCode::CREATED);
}
```
