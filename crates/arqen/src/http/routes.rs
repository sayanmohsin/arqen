use crate::core::error::CorrelationId;
use crate::state::AppState;
use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::{Value, json};

pub async fn health(
    State(_state): State<AppState>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> impl IntoResponse {
    let body = Json(json!({
        "status": "ok",
        "correlation_id": correlation_id.to_string()
    }));
    (StatusCode::OK, body)
}

pub async fn ready(
    State(state): State<AppState>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> impl IntoResponse {
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
