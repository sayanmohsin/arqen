use crate::http::RuntimeInfo;
use axum::{Json, extract::Extension, http::StatusCode, response::IntoResponse};
use serde_json::{Value, json};

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn ready(Extension(runtime): Extension<RuntimeInfo>) -> impl IntoResponse {
    let status = if runtime.thingd_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    let body = Json(json!({
        "status": if runtime.thingd_ready { "ready" } else { "not_ready" },
        "storage_mode": runtime.storage_mode,
        "checks": { "thingd": if runtime.thingd_ready { "ok" } else { "unavailable" } }
    }));
    (status, body)
}

pub async fn agent(Extension(runtime): Extension<RuntimeInfo>) -> Json<Value> {
    let manifest = runtime.registry.generate_manifest();
    Json(json!({
        "name": manifest.name,
        "version": manifest.version,
        "description": manifest.description,
        "storage_mode": manifest.storage_mode,
        "tools_count": manifest.tools.len(),
        "jobs_count": manifest.jobs.len(),
        "endpoints_count": manifest.endpoints.len()
    }))
}

pub async fn agent_manifest(Extension(runtime): Extension<RuntimeInfo>) -> Json<Value> {
    let manifest = runtime.registry.generate_manifest();
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

    Json(json!({
        "name": manifest.name,
        "version": manifest.version,
        "description": manifest.description,
        "storage_mode": manifest.storage_mode,
        "tools": tools,
        "jobs": jobs,
        "endpoints": endpoints
    }))
}

pub async fn docs() -> axum::response::Html<String> {
    axum::response::Html(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Arqen API Documentation</title>
    <style>
        body { font-family: sans-serif; margin: 40px; }
        h1 { color: #333; }
        .endpoint { margin: 20px 0; padding: 10px; border: 1px solid #ddd; border-radius: 5px; }
        .method { font-weight: bold; color: #007bff; }
        .path { font-family: monospace; }
    </style>
</head>
<body>
    <h1>Arqen API Documentation</h1>
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
</html>"#
            .to_string(),
    )
}
