use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use tracing::info;
use uuid::Uuid;

pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map_or_else(|| Uuid::new_v4().to_string(), ToOwned::to_owned);

    let mut response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status().as_u16();

    info!(
        method = %method,
        uri = %uri,
        request_id = %request_id,
        status = status,
        duration_ms = duration.as_millis() as u64,
        "Request completed"
    );

    let header = HeaderValue::try_from(&request_id).expect("UUID is a valid header value");
    response.headers_mut().insert("x-request-id", header);
    response
}
