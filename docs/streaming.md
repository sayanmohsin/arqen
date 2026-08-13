# Streaming large responses

Use `jsonl_response` when a response contains many independent records:

```rust
use futures_util::stream;
use arqen::http::jsonl_response;

let records = stream::iter(vec![
    Ok(serde_json::json!({"id": 1})),
    Ok(serde_json::json!({"id": 2})),
]);
let response = jsonl_response(records);
```

The response uses `application/x-ndjson` and serializes each record as it is
produced. Use it for exports and large reads where clients support JSONL; keep
ordinary JSON for small, stable API payloads.
