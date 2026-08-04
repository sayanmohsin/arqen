use crate::agent::{ToolContext, ToolOutcome};
use crate::auth::AuthContext;
use crate::core::error::CorrelationId;
use crate::core::{AppError, ErrorKind};
use crate::health::HealthReport;
use crate::state::AppState;
use axum::{
    Json,
    extract::{Extension, FromRequest, Path, Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

pub async fn health(
    State(state): State<AppState>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> impl IntoResponse {
    if let Some(registry) = &state.health_registry {
        let report = registry.check_liveness().await;
        let status = StatusCode::from_u16(report.status.to_http_status()).unwrap_or(StatusCode::OK);
        let body = Json(json!({
            "status": format_health_status(&report),
            "checks": format_checks(&report),
            "correlation_id": correlation_id.to_string()
        }));
        (status, body)
    } else {
        let body = Json(json!({
            "status": "ok",
            "correlation_id": correlation_id.to_string()
        }));
        (StatusCode::OK, body)
    }
}

pub async fn ready(
    State(state): State<AppState>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> impl IntoResponse {
    if let Some(registry) = &state.health_registry {
        let report = registry.check_readiness().await;
        let status = StatusCode::from_u16(report.status.to_http_status()).unwrap_or(StatusCode::OK);
        let body = Json(json!({
            "status": format_health_status(&report),
            "storage_mode": state.storage_mode,
            "checks": format_checks(&report),
            "correlation_id": correlation_id.to_string()
        }));
        (status, body)
    } else {
        let status = if state.thingd_ready {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        };
        let body = Json(json!({
            "status": if state.thingd_ready { "ready" } else { "not_ready" },
            "storage_mode": state.storage_mode,
            "checks": { "thingd": if state.thingd_ready { "ok" } else { "unavailable" } },
            "correlation_id": correlation_id.to_string()
        }));
        (status, body)
    }
}

pub async fn agent(
    State(state): State<AppState>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> impl IntoResponse {
    let manifest = state.tool_registry.generate_manifest();
    let body = Json(json!({
        "name": manifest.name,
        "version": manifest.version,
        "description": manifest.description,
        "storage_mode": manifest.storage_mode,
        "tools_count": manifest.tools.len(),
        "jobs_count": manifest.jobs.len(),
        "endpoints_count": manifest.endpoints.len(),
        "correlation_id": correlation_id.to_string()
    }));
    (StatusCode::OK, body)
}

pub async fn agent_manifest(
    State(state): State<AppState>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> impl IntoResponse {
    let manifest = state.tool_registry.generate_manifest();
    let tools: Vec<Value> = manifest
        .tools
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "input": t.input,
                "output": t.output,
                "scopes": t.scopes,
                "effect": format!("{:?}", t.effect),
                "idempotent": t.idempotent,
                "enqueues_job": t.enqueues_job,
                "timeout": t.timeout
            })
        })
        .collect();

    let jobs: Vec<Value> = manifest
        .jobs
        .iter()
        .map(|j| {
            json!({
                "name": j.name,
                "queue": j.queue,
                "description": j.description,
                "max_retries": j.max_retries,
                "timeout": j.timeout
            })
        })
        .collect();

    let endpoints: Vec<Value> = manifest
        .endpoints
        .iter()
        .map(|e| {
            json!({
                "path": e.path,
                "method": e.method,
                "description": e.description,
                "authenticated": e.authenticated
            })
        })
        .collect();

    let body = Json(json!({
        "name": manifest.name,
        "version": manifest.version,
        "description": manifest.description,
        "storage_mode": manifest.storage_mode,
        "tools": tools,
        "jobs": jobs,
        "endpoints": endpoints,
        "correlation_id": correlation_id.to_string()
    }));
    (StatusCode::OK, body)
}

/// Execute a tool by name.
///
/// `POST /agent/tools/:name` with a JSON body matching the tool's input schema.
/// Scopes are derived from the request's auth context when one is present (e.g.
/// when `auth_middleware` is layered in front); anonymous callers can only run
/// tools that declare no required scopes. Output is returned inline, or a 202
/// with the job ID when the tool enqueues a job.
pub async fn tool_invoke(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Extension(correlation_id): Extension<CorrelationId>,
    request: Request,
) -> Response {
    let ctx = request
        .extensions()
        .get::<AuthContext>()
        .map(ToolContext::from_auth_context)
        .unwrap_or_else(ToolContext::anonymous);

    let input = match Json::<Value>::from_request(request, &state).await {
        Ok(Json(value)) => value,
        Err(rejection) => {
            let error = AppError::new(
                ErrorKind::Validation,
                format!("invalid JSON body: {rejection}"),
            );
            return error.into_response();
        }
    };

    match state
        .tool_registry
        .execute(&name, input, &ctx, Some(state.storage.as_ref()))
        .await
    {
        Ok(ToolOutcome::Output(output)) => (
            StatusCode::OK,
            Json(json!({
                "tool": name,
                "output": output,
                "correlation_id": correlation_id.to_string()
            })),
        )
            .into_response(),
        Ok(ToolOutcome::Enqueued { queue, job_id }) => (
            StatusCode::ACCEPTED,
            Json(json!({
                "tool": name,
                "status": "enqueued",
                "queue": queue,
                "job_id": job_id,
                "correlation_id": correlation_id.to_string()
            })),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}

fn format_health_status(report: &HealthReport) -> &'static str {
    match &report.status {
        crate::health::HealthStatus::Healthy => "ok",
        crate::health::HealthStatus::Degraded { .. } => "degraded",
        crate::health::HealthStatus::Unhealthy { .. } => "not_ready",
    }
}

fn format_checks(report: &HealthReport) -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();
    for check in &report.checks {
        let status_str = match &check.status {
            crate::health::HealthStatus::Healthy => "healthy",
            crate::health::HealthStatus::Degraded { .. } => "degraded",
            crate::health::HealthStatus::Unhealthy { .. } => "unhealthy",
        };
        let value = match &check.status {
            crate::health::HealthStatus::Healthy => {
                json!({ "status": status_str, "duration_ms": check.duration_ms })
            }
            crate::health::HealthStatus::Degraded { reason } => {
                json!({ "status": status_str, "reason": reason, "duration_ms": check.duration_ms })
            }
            crate::health::HealthStatus::Unhealthy { reason } => {
                json!({ "status": status_str, "reason": reason, "duration_ms": check.duration_ms })
            }
        };
        map.insert(check.name.clone(), value);
    }
    map
}

pub async fn docs(
    State(_state): State<AppState>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> axum::response::Html<String> {
    axum::response::Html(
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Arqen API Documentation</title>
    <style>
        body {{ font-family: sans-serif; margin: 40px; }}
        h1 {{ color: #333; }}
        .endpoint {{ margin: 20px 0; padding: 10px; border: 1px solid #ddd; border-radius: 5px; }}
        .method {{ font-weight: bold; color: #007bff; }}
        .path {{ font-family: monospace; }}
        .correlation-id {{ font-family: monospace; color: #666; font-size: 0.9em; }}
    </style>
</head>
<body>
    <h1>Arqen API Documentation</h1>
    <p class="correlation-id">Request ID: {}</p>
    <div class="endpoint">
        <span class="method">GET</span> <span class="path">/health</span>
        <p>Liveness check</p>
    </div>
    <div class="endpoint">
        <span class="method">GET</span> <span class="path">/ready</span>
        <p>Readiness check</p>
    </div>
    <div class="endpoint">
        <span class="method">GET</span> <span class="path">/agent</span>
        <p>Agent description</p>
    </div>
    <div class="endpoint">
        <span class="method">GET</span> <span class="path">/agent/manifest</span>
        <p>Agent manifest with tool definitions</p>
    </div>
    <div class="endpoint">
        <span class="method">POST</span> <span class="path">/agent/tools/{{name}}</span>
        <p>Invoke an agent tool by name</p>
    </div>
    <div class="endpoint">
        <span class="method">GET</span> <span class="path">/docs</span>
        <p>This documentation page</p>
    </div>
</body>
</html>"#,
            correlation_id
        )
        .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::{AlwaysHealthy, AlwaysUnhealthy, HealthRegistry};
    use crate::http::correlation_id_middleware;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::middleware;
    use axum::routing::get;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn test_router(state: AppState) -> Router {
        Router::new()
            .route("/health", get(health))
            .route("/ready", get(ready))
            .layer(middleware::from_fn(correlation_id_middleware))
            .with_state(state)
    }

    fn tool_state() -> AppState {
        let mut registry = crate::ToolRegistry::default();
        registry.register_tool(crate::ToolMetadata {
            name: "echo".to_string(),
            description: "Echo input".to_string(),
            input: serde_json::json!({
                "type": "object",
                "properties": {"msg": {"type": "string"}},
                "required": ["msg"]
            }),
            output: serde_json::json!({"type": "object"}),
            scopes: vec![],
            effect: crate::ToolEffect::Read,
            idempotent: true,
            enqueues_job: None,
            timeout: Some(10),
        });
        registry.register_handler("echo", TestEchoHandler);
        registry.register_tool(crate::ToolMetadata {
            name: "secret".to_string(),
            description: "Requires scope".to_string(),
            input: serde_json::json!({"type": "object"}),
            output: serde_json::json!({"type": "object"}),
            scopes: vec!["read:secret".to_string()],
            effect: crate::ToolEffect::Read,
            idempotent: true,
            enqueues_job: None,
            timeout: None,
        });
        registry.register_handler("secret", TestEchoHandler);
        AppState::builder()
            .with_tool_registry(registry)
            .build()
            .unwrap()
    }

    #[derive(Clone)]
    struct TestEchoHandler;

    #[async_trait::async_trait]
    impl crate::agent::ToolHandler for TestEchoHandler {
        async fn execute(
            &self,
            _ctx: &ToolContext,
            input: serde_json::Value,
        ) -> Result<serde_json::Value, crate::core::AppError> {
            Ok(input)
        }
    }

    fn tool_router(state: AppState) -> Router {
        Router::new()
            .route("/agent/tools/:name", axum::routing::post(tool_invoke))
            .layer(middleware::from_fn(correlation_id_middleware))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_tool_invoke_returns_output() {
        let state = tool_state();
        let router = tool_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/agent/tools/echo")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::json!({"msg": "hi"}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["tool"], "echo");
        assert_eq!(json["output"]["msg"], "hi");
    }

    #[tokio::test]
    async fn test_tool_invoke_unknown_tool_returns_not_found() {
        let state = tool_state();
        let router = tool_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/agent/tools/nope")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::json!({}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_tool_invoke_invalid_input_returns_validation_error() {
        let state = tool_state();
        let router = tool_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/agent/tools/echo")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::json!({}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "validation");
    }

    #[tokio::test]
    async fn test_tool_invoke_scope_enforced() {
        let state = tool_state();
        let router = tool_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/agent/tools/secret")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::json!({}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_tool_invoke_with_auth_context_scope() {
        let state = tool_state();
        let router = tool_router(state);

        let mut request = Request::builder()
            .method("POST")
            .uri("/agent/tools/secret")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::json!({}).to_string()))
            .unwrap();
        request.extensions_mut().insert(
            AuthContext::new("user-1", "test")
                .with_claim("scopes", serde_json::json!(["read:secret"])),
        );

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_without_registry() {
        let state = AppState::builder().build().unwrap();
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_with_registry_healthy() {
        let mut registry = HealthRegistry::new();
        registry.register(Arc::new(AlwaysHealthy));
        let state = AppState::builder()
            .with_health_registry(registry)
            .build()
            .unwrap();
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_with_registry_unhealthy() {
        let mut registry = HealthRegistry::new();
        registry.register(Arc::new(AlwaysUnhealthy::new("down")));
        let state = AppState::builder()
            .with_health_registry(registry)
            .build()
            .unwrap();
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_ready_without_registry() {
        let state = AppState::builder().build().unwrap();
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_ready_with_registry_healthy() {
        let mut registry = HealthRegistry::new();
        registry.register(Arc::new(AlwaysHealthy));
        let state = AppState::builder()
            .with_health_registry(registry)
            .build()
            .unwrap();
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_ready_with_registry_unhealthy() {
        let mut registry = HealthRegistry::new();
        registry.register(Arc::new(AlwaysUnhealthy::new("down")));
        let state = AppState::builder()
            .with_health_registry(registry)
            .build()
            .unwrap();
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
