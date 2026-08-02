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
    Json(json!({
        "name": "arqen-app",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "An Arqen application",
        "storage_mode": runtime.storage_mode
    }))
}

pub async fn agent_manifest(Extension(runtime): Extension<RuntimeInfo>) -> Json<Value> {
    Json(json!({
        "name": "arqen-app",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "An Arqen application",
        "storage_mode": runtime.storage_mode,
        "tools": [],
        "jobs": [],
        "endpoints": [
            {
                "path": "/health",
                "method": "GET",
                "description": "Liveness check",
                "authenticated": false
            },
            {
                "path": "/ready",
                "method": "GET",
                "description": "Readiness check",
                "authenticated": false
            },
            {
                "path": "/agent",
                "method": "GET",
                "description": "Agent description",
                "authenticated": false
            },
            {
                "path": "/agent/manifest",
                "method": "GET",
                "description": "Agent manifest",
                "authenticated": false
            },
            {
                "path": "/docs",
                "method": "GET",
                "description": "API documentation",
                "authenticated": false
            }
        ]
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
