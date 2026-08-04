use crate::core::error::CorrelationId;
use crate::health::HealthReport;
use crate::state::AppState;
use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
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
