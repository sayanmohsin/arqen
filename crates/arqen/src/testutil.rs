//! Testing utilities module for Arqen.
//!
//! Provides TestApp, MockAuth, fixture helpers, and request builders for testing.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::response::Response;
use axum::Router;
use tower::ServiceExt;

use crate::agent::ToolRegistry;
use crate::auth::{AuthContext, AuthError, Authentication};
use crate::config::AppConfig;
use crate::state::AppState;
use crate::thingd::{MemoryThingdBackend, ThingdBackend};

/// Test application with memory adapters.
pub struct TestApp {
    state: AppState,
    router: Router,
}

impl TestApp {
    /// Create a new TestApp builder.
    pub fn builder() -> TestAppBuilder {
        TestAppBuilder::new()
    }

    /// Make an HTTP request to the test app.
    pub async fn request(&self, method: Method, path: &str) -> Response {
        self.router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("failed to make request")
    }

    /// Make a GET request.
    pub async fn get(&self, path: &str) -> Response {
        self.request(Method::GET, path).await
    }

    /// Make a POST request with JSON body.
    pub async fn post_json(&self, path: &str, body: serde_json::Value) -> Response {
        self.router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .expect("failed to make request")
    }

    /// Make a PUT request with JSON body.
    pub async fn put_json(&self, path: &str, body: serde_json::Value) -> Response {
        self.router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .expect("failed to make request")
    }

    /// Make a DELETE request.
    pub async fn delete(&self, path: &str) -> Response {
        self.request(Method::DELETE, path).await
    }

    /// Make a request with custom headers.
    pub async fn request_with_headers(
        &self,
        method: Method,
        path: &str,
        headers: Vec<(&str, &str)>,
    ) -> Response {
        let mut builder = Request::builder().method(method).uri(path);
        for (key, value) in headers {
            builder = builder.header(key, value);
        }
        self.router
            .clone()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .expect("failed to make request")
    }

    /// Get the app state.
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Get the storage backend.
    pub fn storage(&self) -> &Arc<dyn ThingdBackend> {
        &self.state.storage
    }

    /// Get the tool registry.
    pub fn registry(&self) -> &Arc<ToolRegistry> {
        &self.state.tool_registry
    }
}

/// Builder for TestApp.
pub struct TestAppBuilder {
    config: AppConfig,
    auth: Option<Arc<dyn Authentication>>,
    storage: Option<Arc<dyn ThingdBackend>>,
    registry: Option<ToolRegistry>,
}

impl TestAppBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
            auth: None,
            storage: None,
            registry: None,
        }
    }

    /// Set the config.
    pub fn with_config(mut self, config: AppConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the auth adapter.
    pub fn with_auth(mut self, auth: Arc<dyn Authentication>) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Set the storage backend.
    pub fn with_storage(mut self, storage: Arc<dyn ThingdBackend>) -> Self {
        self.storage = Some(storage);
        self
    }

    /// Set the tool registry.
    pub fn with_registry(mut self, registry: ToolRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Build the TestApp.
    pub fn build(self) -> TestApp {
        let storage = self.storage.unwrap_or_else(|| Arc::new(MemoryThingdBackend::new()));

        let registry = self.registry.unwrap_or_else(|| {
            ToolRegistry::new(
                &format!("{}-test", env!("CARGO_PKG_NAME")),
                env!("CARGO_PKG_VERSION"),
                "Test agent",
                "memory",
            )
        });

        let state = AppState::builder()
            .with_config(self.config)
            .with_storage(storage)
            .with_tool_registry(registry)
            .build()
            .expect("failed to build AppState");

        let router = crate::http::create_router();

        TestApp { state, router }
    }
}

impl Default for TestAppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock authentication adapter for testing.
pub struct MockAuth {
    behavior: MockAuthBehavior,
}

/// Behavior for MockAuth.
pub enum MockAuthBehavior {
    /// Always succeed with the given context.
    AlwaysSuccess(AuthContext),
    /// Always fail with the given error.
    AlwaysFail(AuthError),
}

impl MockAuth {
    /// Create a MockAuth that always succeeds.
    pub fn always_success(subject: impl Into<String>) -> Self {
        Self {
            behavior: MockAuthBehavior::AlwaysSuccess(AuthContext::new(subject, "mock")),
        }
    }

    /// Create a MockAuth that always fails.
    pub fn always_fail(error: AuthError) -> Self {
        Self {
            behavior: MockAuthBehavior::AlwaysFail(error),
        }
    }

    /// Create a MockAuth that returns missing credentials error.
    pub fn always_missing() -> Self {
        Self::always_fail(AuthError::Missing)
    }

    /// Create a MockAuth that returns invalid credentials error.
    pub fn always_invalid() -> Self {
        Self::always_fail(AuthError::Invalid)
    }
}

#[axum::async_trait]
impl Authentication for MockAuth {
    async fn authenticate(&self, _headers: &axum::http::HeaderMap) -> Result<AuthContext, AuthError> {
        match &self.behavior {
            MockAuthBehavior::AlwaysSuccess(ctx) => Ok(ctx.clone()),
            MockAuthBehavior::AlwaysFail(err) => Err(err.clone()),
        }
    }
}

/// Fixture helper for creating test data.
pub struct Fixtures {
    storage: Arc<dyn ThingdBackend>,
}

impl Fixtures {
    /// Create a new Fixtures instance.
    pub fn new(storage: Arc<dyn ThingdBackend>) -> Self {
        Self { storage }
    }

    /// Create a test object.
    pub async fn create_object(
        &self,
        kind: &str,
        id: &str,
        data: serde_json::Value,
    ) -> Result<crate::thingd::ThingdObject, crate::core::AppError> {
        self.storage.put_object(kind, id, data).await
    }

    /// Get a test object.
    pub async fn get_object(
        &self,
        kind: &str,
        id: &str,
    ) -> Result<Option<serde_json::Value>, crate::core::AppError> {
        let obj = self.storage.get_object(kind, id).await?;
        Ok(obj.map(|o| o.data))
    }

    /// Delete a test object.
    pub async fn delete_object(
        &self,
        kind: &str,
        id: &str,
    ) -> Result<(), crate::core::AppError> {
        self.storage.delete_object(kind, id).await
    }

    /// Create multiple test objects.
    pub async fn create_objects(
        &self,
        kind: &str,
        count: usize,
    ) -> Result<Vec<crate::thingd::ThingdObject>, crate::core::AppError> {
        let mut objects = Vec::new();
        for i in 0..count {
            let id = format!("{}-{}", kind, i);
            let data = serde_json::json!({"index": i});
            let obj = self.storage.put_object(kind, &id, data).await?;
            objects.push(obj);
        }
        Ok(objects)
    }
}

/// Response body reader.
pub async fn read_body(response: Response) -> serde_json::Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("failed to read body");
    serde_json::from_slice(&body).expect("failed to parse JSON")
}

/// Assert that a response has the expected status code.
#[macro_export]
macro_rules! assert_response {
    ($response:expr, $status:expr) => {
        assert_eq!($response.status(), $status, "expected status {}", $status);
    };
    ($response:expr, $status:expr, $body:expr) => {
        assert_eq!($response.status(), $status, "expected status {}", $status);
        let body = axum::body::to_bytes($response.into_body(), usize::MAX)
            .await
            .expect("failed to read body");
        let body: serde_json::Value = serde_json::from_slice(&body).expect("failed to parse JSON");
        assert_eq!(body, $body);
    };
}

/// Assert that an error response has the expected error code.
#[macro_export]
macro_rules! assert_error {
    ($response:expr, $code:expr) => {
        assert_eq!($response.status(), $crate::core::error::ErrorCode::$code.status_code());
        let body = axum::body::to_bytes($response.into_body(), usize::MAX)
            .await
            .expect("failed to read body");
        let body: serde_json::Value = serde_json::from_slice(&body).expect("failed to parse JSON");
        assert_eq!(body["error"]["code"], stringify!($code));
    };
}

/// Assert that JSON contains expected fields.
#[macro_export]
macro_rules! assert_json_contains {
    ($json:expr, { $($key:expr => $value:expr),* $(,)? }) => {
        $(
            assert_eq!($json[$key], $value, "expected {} to be {:?}", $key, $value);
        )*
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_testapp_builder_default() {
        let app = TestApp::builder().build();
        let response = app.get("/health").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_testapp_get() {
        let app = TestApp::builder().build();
        let response = app.get("/health").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_testapp_post_json() {
        let app = TestApp::builder().build();
        let response = app.post_json("/agent", json!({})).await;
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_testapp_put_json() {
        let app = TestApp::builder().build();
        let response = app.put_json("/agent", json!({})).await;
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_testapp_delete() {
        let app = TestApp::builder().build();
        let response = app.delete("/agent").await;
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_testapp_state() {
        let app = TestApp::builder().build();
        assert_eq!(app.state().config.server.port, 3000);
    }

    #[tokio::test]
    async fn test_testapp_storage() {
        let app = TestApp::builder().build();
        let result = app.storage().count_objects("test").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_testapp_registry() {
        let app = TestApp::builder().build();
        let manifest = app.registry().generate_manifest();
        assert!(manifest.name.contains("test"));
    }

    #[tokio::test]
    async fn test_mock_auth_always_success() {
        let auth = MockAuth::always_success("user-123");
        let ctx = auth.authenticate(&axum::http::HeaderMap::new()).await.unwrap();
        assert_eq!(ctx.subject, "user-123");
        assert_eq!(ctx.adapter, "mock");
    }

    #[tokio::test]
    async fn test_mock_auth_always_fail() {
        let auth = MockAuth::always_fail(AuthError::Invalid);
        let err = auth.authenticate(&axum::http::HeaderMap::new()).await.unwrap_err();
        assert_eq!(err, AuthError::Invalid);
    }

    #[tokio::test]
    async fn test_mock_auth_always_missing() {
        let auth = MockAuth::always_missing();
        let err = auth.authenticate(&axum::http::HeaderMap::new()).await.unwrap_err();
        assert_eq!(err, AuthError::Missing);
    }

    #[tokio::test]
    async fn test_mock_auth_always_invalid() {
        let auth = MockAuth::always_invalid();
        let err = auth.authenticate(&axum::http::HeaderMap::new()).await.unwrap_err();
        assert_eq!(err, AuthError::Invalid);
    }

    #[tokio::test]
    async fn test_fixtures_create_object() {
        let storage = Arc::new(MemoryThingdBackend::new());
        let fixtures = Fixtures::new(storage);
        let obj = fixtures.create_object("user", "user-1", json!({"name": "Alice"})).await.unwrap();
        assert_eq!(obj.id, "user-1");
    }

    #[tokio::test]
    async fn test_fixtures_get_object() {
        let storage = Arc::new(MemoryThingdBackend::new());
        let fixtures = Fixtures::new(storage);
        let obj = fixtures.create_object("user", "user-1", json!({"name": "Alice"})).await.unwrap();
        let data = fixtures.get_object("user", &obj.id).await.unwrap();
        assert!(data.is_some());
        assert_eq!(data.unwrap()["name"], "Alice");
    }

    #[tokio::test]
    async fn test_fixtures_get_nonexistent() {
        let storage = Arc::new(MemoryThingdBackend::new());
        let fixtures = Fixtures::new(storage);
        let obj = fixtures.get_object("user", "nonexistent").await.unwrap();
        assert!(obj.is_none());
    }

    #[tokio::test]
    async fn test_fixtures_create_multiple() {
        let storage = Arc::new(MemoryThingdBackend::new());
        let fixtures = Fixtures::new(storage);
        let objects = fixtures.create_objects("item", 5).await.unwrap();
        assert_eq!(objects.len(), 5);
    }

    #[tokio::test]
    async fn test_read_body() {
        let app = TestApp::builder().build();
        let response = app.get("/health").await;
        let body = read_body(response).await;
        assert!(body.is_object() || body.is_string() || body.is_null());
    }
}
