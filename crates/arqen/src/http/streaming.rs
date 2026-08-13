//! Explicit newline-delimited JSON responses for large result sets.

use axum::body::Body;
use axum::http::{HeaderValue, header};
use axum::response::Response;
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use serde::Serialize;

/// Build an `application/x-ndjson` response from an async record stream.
pub fn jsonl_response<S, T>(stream: S) -> Response
where
    S: Stream<Item = Result<T, serde_json::Error>> + Send + 'static,
    T: Serialize,
{
    let body_stream = stream.map(|item| match item {
        Ok(value) => serde_json::to_vec(&value)
            .map(|mut bytes| {
                bytes.push(b'\n');
                Bytes::from(bytes)
            })
            .map_err(std::io::Error::other),
        Err(error) => Err(std::io::Error::other(error)),
    });
    let mut response = Response::new(Body::from_stream(body_stream));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/x-ndjson"),
    );
    response
}
