# Interfaces

## TestApp

```rust
pub struct TestApp {
    state: AppState,
    router: Router,
}

impl TestApp {
    pub fn builder() -> TestAppBuilder { ... }
    pub async fn request(&self, method: Method, path: &str) -> TestResponse { ... }
    pub fn state(&self) -> &AppState { ... }
}

pub struct TestAppBuilder {
    config: TestConfig,
    auth: Option<Arc<dyn Authentication>>,
    storage: Option<Arc<dyn ThingdBackend>>,
}

impl TestAppBuilder {
    pub fn new() -> Self { ... }
    pub fn with_auth(mut self, auth: Arc<dyn Authentication>) -> Self { ... }
    pub fn with_storage(mut self, storage: Arc<dyn ThingdBackend>) -> Self { ... }
    pub fn build(self) -> TestApp { ... }
}
```

## MockAuth

```rust
pub struct MockAuth {
    behavior: MockAuthBehavior,
}

pub enum MockAuthBehavior {
    AlwaysSuccess(AuthContext),
    AlwaysFail(AuthError),
    Custom(Box<dyn Fn(&HeaderMap) -> Result<AuthContext, AuthError> + Send + Sync>),
}
```

## Fixtures

```rust
pub struct Fixtures {
    storage: Arc<dyn ThingdBackend>,
}

impl Fixtures {
    pub fn new(storage: Arc<dyn ThingdBackend>) -> Self { ... }
    pub async fn create_user(&self, name: &str, email: &str) -> ThingdObject { ... }
    pub async fn create_event(&self, stream: &str, event_type: &str) -> ThingdEvent { ... }
    pub async fn create_job(&self, queue: &str, payload: Value) -> ThingdJob { ... }
}
```

## Test Assertions

```rust
macro_rules! assert_response {
    ($response:expr, $status:expr) => { ... };
    ($response:expr, $status:expr, $body:expr) => { ... };
}

macro_rules! assert_error {
    ($response:expr, $code:expr) => { ... };
}
```
