#![cfg(feature = "http-client")]

use arqen::thingd::{
    FilterOperator, HttpThingdBackend, PushJobOptions, QueryOptions, SearchOptions, ThingdBackend,
    ThingdFilter, ThingdOperation,
};
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::Response;
use axum::routing::any;
use std::sync::Arc;

#[path = "common/phase18_contract.rs"]
mod phase18_contract;

/// Run against a direct Thingd REST server with `ARQEN_THINGD_TEST_URL` set.
/// If the server requires bearer auth, set its `THINGD_AUTH_TOKEN` as
/// `ARQEN_THINGD_AUTH_TOKEN`; Cloud project keys target the separate MCP API.
fn thingd_test_backend(base_url: &str) -> HttpThingdBackend {
    let backend = HttpThingdBackend::new(base_url);
    match std::env::var("ARQEN_THINGD_AUTH_TOKEN") {
        Ok(token) => backend.with_auth(&token),
        Err(_) => backend,
    }
}

#[tokio::test]
#[ignore = "requires a running thingd-server"]
async fn thingd_server_public_rest_contract() {
    let base_url = std::env::var("ARQEN_THINGD_TEST_URL")
        .expect("ARQEN_THINGD_TEST_URL must point at thingd-server");
    let backend = thingd_test_backend(&base_url);
    let collection = format!("arqen_contract_{}", uuid::Uuid::new_v4());

    let object = backend
        .put_object(&collection, "one", serde_json::json!({"name":"one"}))
        .await
        .unwrap();
    assert_eq!(object.id, "one");
    assert!(
        backend
            .get_object(&collection, "one")
            .await
            .unwrap()
            .is_some()
    );

    let batch = backend
        .batch_write(vec![
            ThingdOperation::Put {
                collection: collection.clone(),
                id: "two".into(),
                data: serde_json::json!({"name":"two"}),
            },
            ThingdOperation::Put {
                collection: collection.clone(),
                id: "three".into(),
                data: serde_json::json!({"name":"three"}),
            },
        ])
        .await
        .unwrap();
    assert_eq!(batch.len(), 2);
    assert!(batch.iter().all(|result| result.success));

    let queried = backend
        .query_objects(&collection, QueryOptions::default())
        .await
        .unwrap();
    assert!(queried.len() >= 3);
    backend
        .put_object(
            &collection,
            "expired",
            serde_json::json!({"expiresAt":"2026-08-10T12:00:00Z"}),
        )
        .await
        .unwrap();
    backend
        .put_object(
            &collection,
            "valid",
            serde_json::json!({"expiresAt":"2026-08-12T12:00:00Z"}),
        )
        .await
        .unwrap();
    let expired = backend
        .query_objects(
            &collection,
            QueryOptions::filtered(vec![ThingdFilter {
                field: "expiresAt".into(),
                operator: FilterOperator::Lt,
                value: serde_json::json!("2026-08-11T12:00:00+00:00"),
            }]),
        )
        .await
        .unwrap();
    assert!(expired.iter().any(|object| object.id == "expired"));
    assert!(!expired.iter().any(|object| object.id == "valid"));
    let searched = backend
        .search(
            "two",
            SearchOptions {
                limit: 10,
                offset: 0,
                filters: vec![],
            },
        )
        .await
        .unwrap();
    assert!(searched.items.iter().any(|item| item.id == "two"));
    let filtered_search = backend
        .search(
            "two",
            SearchOptions {
                limit: 10,
                offset: 0,
                filters: vec![ThingdFilter {
                    field: "name".into(),
                    operator: FilterOperator::Eq,
                    value: serde_json::json!("not-two"),
                }],
            },
        )
        .await
        .unwrap();
    assert!(filtered_search.items.is_empty());

    let event = backend
        .append_event(
            &format!("{collection}:events"),
            "contract.created",
            serde_json::json!({"id":"one"}),
        )
        .await
        .unwrap();
    assert_eq!(event.event_type, "contract.created");
    assert!(
        !backend
            .read_events(&format!("{collection}:events"), None, 10)
            .await
            .unwrap()
            .is_empty()
    );

    let job = backend
        .push_job(
            &format!("{collection}:jobs"),
            serde_json::json!({"id":"one"}),
            2,
        )
        .await
        .unwrap();
    let claimed = backend
        .claim_job(&job.queue, "arqen-contract-worker", 30)
        .await
        .unwrap()
        .expect("job should be claimable");
    backend.complete_job(&job.queue, &claimed.id).await.unwrap();

    let link = backend.create_link("one", "two", "related").await.unwrap();
    assert!(
        backend
            .get_links("one", Some("related"))
            .await
            .unwrap()
            .iter()
            .any(|item| item.id == link.id)
    );
    backend.delete_link(&link.id).await.unwrap();

    // Do not call the global reset endpoint here: this test may run against a
    // shared development server. The collection name is unique per run and
    // the link is deleted explicitly above.
}

/// The target must be a Thingd 0.92.3 server. All records are isolated by
/// unique tenant, instance, collection, stream, and queue identifiers; this
/// contract never calls a global reset endpoint.
#[tokio::test]
#[ignore = "requires an isolated Thingd 0.92.3 server"]
async fn thingd_0923_server_passes_phase18_adapter_contract() {
    let base_url = std::env::var("ARQEN_THINGD_TEST_URL")
        .expect("ARQEN_THINGD_TEST_URL must point at Thingd 0.92.3");
    let backend: Arc<dyn ThingdBackend> = Arc::new(thingd_test_backend(&base_url));
    phase18_contract::run_phase18_contract(backend).await;
}

/// Simulates a connection-level lost success: the proxy lets the first keyed
/// push commit upstream, then returns 503 so the adapter retries. The queue is
/// unique to this run and is never globally reset.
#[tokio::test]
#[ignore = "requires a reachable Thingd 0.92.3 server"]
async fn thingd_0923_http_retry_replay_does_not_duplicate_jobs() {
    let upstream = std::env::var("ARQEN_THINGD_TEST_URL")
        .expect("ARQEN_THINGD_TEST_URL must point at Thingd 0.92.3")
        .trim_end_matches('/')
        .to_string();
    let attempts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let keys = Arc::new(std::sync::Mutex::new(Vec::new()));
    let attempts_for_proxy = Arc::clone(&attempts);
    let keys_for_proxy = Arc::clone(&keys);
    let client = reqwest::Client::new();
    let app = Router::new().route(
        "/{*path}",
        any(move |request: Request| {
            let upstream = upstream.clone();
            let client = client.clone();
            let attempts = Arc::clone(&attempts_for_proxy);
            let keys = Arc::clone(&keys_for_proxy);
            async move {
                let path_and_query = request
                    .uri()
                    .path_and_query()
                    .map(|value| value.as_str().to_string())
                    .unwrap_or_else(|| request.uri().path().to_string());
                let url = format!("{upstream}{path_and_query}");
                let method = request.method().clone();
                let headers = request.headers().clone();
                let body = to_bytes(request.into_body(), usize::MAX).await.unwrap();
                if method == reqwest::Method::POST && path_and_query.ends_with("/push") {
                    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
                    keys.lock().unwrap().push(
                        parsed["idempotencyKey"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                    );
                }
                let mut outbound = client.request(method.clone(), url).body(body);
                for (name, value) in &headers {
                    if name != axum::http::header::HOST {
                        outbound = outbound.header(name, value);
                    }
                }
                let upstream_response = outbound.send().await.unwrap();
                let status = upstream_response.status();
                let response_body = upstream_response.bytes().await.unwrap();
                if method == reqwest::Method::POST
                    && path_and_query.ends_with("/push")
                    && status.is_success()
                    && attempts.fetch_add(1, std::sync::atomic::Ordering::Relaxed) == 0
                {
                    return Response::builder()
                        .status(503)
                        .header("Retry-After", "0")
                        .body(Body::from(
                            r#"{"error":{"detail":"simulated lost response"}}"#,
                        ))
                        .unwrap();
                }
                Response::builder()
                    .status(status)
                    .body(Body::from(response_body))
                    .unwrap()
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let proxy = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let queue = format!("arqen_phase18_{}", uuid::Uuid::new_v4());
    let key = format!("phase18-{}", uuid::Uuid::new_v4());
    let backend = thingd_test_backend(&format!("http://{address}"));
    let job = backend
        .push_job_with_options(
            &queue,
            serde_json::json!({"run":"retry-replay"}),
            3,
            PushJobOptions {
                idempotency_key: Some(key.clone()),
                delay_ms: None,
            },
        )
        .await
        .unwrap();

    {
        let sent_keys = keys.lock().unwrap();
        assert_eq!(attempts.load(std::sync::atomic::Ordering::Relaxed), 2);
        assert_eq!(sent_keys.as_slice(), &[key.clone(), key]);
    }
    let claimed = backend
        .claim_job(&queue, "phase18-retry-worker", 30)
        .await
        .unwrap()
        .expect("the successful replay should leave one claimable job");
    assert_eq!(claimed.id, job.id);
    assert!(
        backend
            .claim_job(&queue, "phase18-retry-worker", 30)
            .await
            .unwrap()
            .is_none()
    );
    proxy.abort();
}

#[tokio::test]
#[ignore = "requires a reachable Thingd 0.92.3 server"]
async fn thingd_0923_http_adapter_honors_queue_delay() {
    let base_url = std::env::var("ARQEN_THINGD_TEST_URL")
        .expect("ARQEN_THINGD_TEST_URL must point at Thingd 0.92.3");
    let queue = format!("arqen_phase18_delay_{}", uuid::Uuid::new_v4());
    let backend = thingd_test_backend(&base_url);
    backend
        .push_job_with_options(
            &queue,
            serde_json::json!({"run":"delay"}),
            3,
            PushJobOptions {
                idempotency_key: Some(format!("delay-{}", uuid::Uuid::new_v4())),
                delay_ms: Some(500),
            },
        )
        .await
        .unwrap();
    assert!(
        backend
            .claim_job(&queue, "phase18-delay-worker", 30)
            .await
            .unwrap()
            .is_none()
    );
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    assert!(
        backend
            .claim_job(&queue, "phase18-delay-worker", 30)
            .await
            .unwrap()
            .is_some()
    );
}
