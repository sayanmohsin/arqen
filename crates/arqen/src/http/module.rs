//! HTTP module support for Arqen.
//!
//! Provides a trait for modules that expose HTTP routes via Axum.
//! This is feature-gated on `http-server`.

use axum::Router;

use crate::module::Module;
use crate::state::AppState;

/// A module that exposes HTTP routes.
///
/// This trait extends [`Module`] with the ability to return an Axum [`Router`].
/// It is separate from the core `Module` trait to keep the module system
/// framework-neutral.
///
/// # Example
///
/// ```rust,ignore
/// use arqen::http::HttpModule;
/// use arqen::module::Module;
/// use arqen::state::AppState;
/// use axum::{Router, routing::get, Json};
///
/// struct UsersModule;
///
/// async fn list_users() -> Json<serde_json::Value> {
///     Json(serde_json::json!({"users": []}))
/// }
///
/// impl Module for UsersModule {
///     fn name(&self) -> &str { "users" }
/// }
///
/// impl HttpModule for UsersModule {
///     fn router(&self) -> Router<AppState> {
///         Router::new()
///             .route("/users", get(list_users))
///     }
/// }
/// ```
pub trait HttpModule: Module {
    /// Return the Axum router for this module's HTTP routes.
    fn router(&self) -> Router<AppState>;
}

/// Merge multiple HTTP module routers into a base router.
///
/// Each module's router is merged sequentially. This is a convenience
/// function for explicit route composition.
///
/// # Example
///
/// ```rust,ignore
/// use arqen::http::{merge_module_routes, create_router_with_state};
///
/// let router = create_router_with_state(state.clone());
/// let router = merge_module_routes(router, &[Box::new(UsersModule), Box::new(JobsModule)]);
/// ```
pub fn merge_module_routes(
    base: Router<AppState>,
    modules: &[Box<dyn HttpModule>],
) -> Router<AppState> {
    let mut router = base;
    for m in modules {
        router = router.merge(m.router());
    }
    router
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::routing::get;

    struct TestModule;

    #[async_trait]
    impl crate::module::Module for TestModule {
        fn name(&self) -> &str {
            "test"
        }
    }

    impl HttpModule for TestModule {
        fn router(&self) -> Router<AppState> {
            Router::new()
        }
    }

    struct UsersModule;

    #[async_trait]
    impl crate::module::Module for UsersModule {
        fn name(&self) -> &str {
            "users"
        }
    }

    impl HttpModule for UsersModule {
        fn router(&self) -> Router<AppState> {
            async fn list_users() -> &'static str {
                "ok"
            }
            Router::new().route("/users", get(list_users))
        }
    }

    struct JobsModule;

    #[async_trait]
    impl crate::module::Module for JobsModule {
        fn name(&self) -> &str {
            "jobs"
        }
    }

    impl HttpModule for JobsModule {
        fn router(&self) -> Router<AppState> {
            async fn list_jobs() -> &'static str {
                "ok"
            }
            Router::new().route("/jobs", get(list_jobs))
        }
    }

    #[test]
    fn test_http_module_router() {
        let module = TestModule;
        let _router = module.router();
    }

    #[test]
    fn test_merge_module_routes_empty_list() {
        let base = Router::new();
        let modules: Vec<Box<dyn HttpModule>> = vec![];
        let _router = merge_module_routes(base, &modules);
    }

    #[test]
    fn test_merge_module_routes_single() {
        let base = Router::new();
        let modules: Vec<Box<dyn HttpModule>> = vec![Box::new(TestModule)];
        let _router = merge_module_routes(base, &modules);
    }

    #[test]
    fn test_merge_module_routes_multiple() {
        let base = Router::new();
        let modules: Vec<Box<dyn HttpModule>> = vec![Box::new(UsersModule), Box::new(JobsModule)];
        let _router = merge_module_routes(base, &modules);
    }

    #[test]
    fn test_merge_module_routes_with_base_routes() {
        async fn health() -> &'static str {
            "ok"
        }
        let base = Router::new().route("/health", get(health));
        let modules: Vec<Box<dyn HttpModule>> = vec![Box::new(UsersModule)];
        let _router = merge_module_routes(base, &modules);
    }
}
