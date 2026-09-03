//! Application-owned request hooks adapted into the internal HTTP pipeline.

use async_trait::async_trait;
use axum::response::IntoResponse;

use crate::context::RequestContext;
use crate::core::{AppError, ErrorKind};

/// Read-only information supplied to an application middleware hook.
#[derive(Debug, Clone)]
pub struct MiddlewareContext {
    method: String,
    path: String,
    request_id: String,
    request: RequestContext,
    status: Option<u16>,
}

impl MiddlewareContext {
    /// HTTP method for the current request.
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Request path for the current request.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Correlation/request identifier for the current request.
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Authentication and tenancy context, if available.
    pub fn request_context(&self) -> &RequestContext {
        &self.request
    }

    /// Response status. This is `None` in the pre-request callback.
    pub fn status(&self) -> Option<u16> {
        self.status
    }
}

/// Arqen-owned request middleware hook.
///
/// Hooks run in registration order before a handler and in reverse
/// registration order after the handler. Returning an error from `before`
/// stops the request and produces Arqen's standard error response. Returning
/// an error from `after` replaces the response with that error response.
#[async_trait]
pub trait MiddlewareHook: Send + Sync {
    /// Stable name used in diagnostics.
    fn name(&self) -> &str;

    /// Run before the application handler.
    async fn before(
        &self,
        _context: &MiddlewareContext,
        _state: &crate::state::AppState,
    ) -> Result<(), AppError> {
        Ok(())
    }

    /// Run after the application handler has produced a response.
    async fn after(
        &self,
        _context: &MiddlewareContext,
        _state: &crate::state::AppState,
    ) -> Result<(), AppError> {
        Ok(())
    }
}

pub(crate) async fn run_middleware_hooks(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let request_context = crate::context::from_extensions(request.extensions());
    let context = MiddlewareContext {
        method: request.method().to_string(),
        path: request.uri().path().to_string(),
        request_id: request_context.correlation_id.clone(),
        request: request_context,
        status: None,
    };

    for hook in state.middleware_hooks.iter() {
        if let Err(error) = hook.before(&context, &state).await {
            tracing::warn!(hook = hook.name(), error = %error, "request hook rejected request");
            return error.into_response();
        }
    }

    let mut response = next.run(request).await;
    let response_context = MiddlewareContext {
        status: Some(response.status().as_u16()),
        ..context
    };
    for hook in state.middleware_hooks.iter().rev() {
        if let Err(error) = hook.after(&response_context, &state).await {
            tracing::warn!(hook = hook.name(), error = %error, "response hook failed");
            response = error.into_response();
            break;
        }
    }
    response
}

/// Construct an authorization error for a hook that rejects a request.
pub fn reject(message: impl Into<String>) -> AppError {
    AppError::new(ErrorKind::Authorization, message)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    use super::*;

    struct RecordingHook {
        name: &'static str,
        events: Arc<Mutex<Vec<String>>>,
        reject_path: bool,
    }

    #[async_trait]
    impl MiddlewareHook for RecordingHook {
        fn name(&self) -> &str {
            self.name
        }

        async fn before(
            &self,
            context: &MiddlewareContext,
            _state: &crate::state::AppState,
        ) -> Result<(), AppError> {
            self.events
                .lock()
                .unwrap()
                .push(format!("before:{}", self.name));
            if self.reject_path && context.path() == "/blocked" {
                return Err(reject("blocked by test hook"));
            }
            Ok(())
        }

        async fn after(
            &self,
            _context: &MiddlewareContext,
            _state: &crate::state::AppState,
        ) -> Result<(), AppError> {
            self.events
                .lock()
                .unwrap()
                .push(format!("after:{}", self.name));
            Ok(())
        }
    }

    #[tokio::test]
    async fn hooks_run_in_order_and_can_reject_early() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let state = crate::state::AppState::builder()
            .with_middleware_hook(RecordingHook {
                name: "first",
                events: events.clone(),
                reject_path: false,
            })
            .with_middleware_hook(RecordingHook {
                name: "second",
                events: events.clone(),
                reject_path: false,
            })
            .build()
            .unwrap();
        let response = crate::http::create_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success());
        assert_eq!(
            *events.lock().unwrap(),
            vec![
                "before:first",
                "before:second",
                "after:second",
                "after:first"
            ]
        );

        let state = crate::state::AppState::builder()
            .with_middleware_hook(RecordingHook {
                name: "rejector",
                events: events.clone(),
                reject_path: true,
            })
            .build()
            .unwrap();
        let response = crate::http::create_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/blocked")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::FORBIDDEN);
        assert_eq!(events.lock().unwrap().last().unwrap(), "before:rejector");
    }
}
