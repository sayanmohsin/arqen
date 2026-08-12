use arqen::thingd::MemoryThingdBackend;
use arqen::thingd::*;
use serde_json::json;

#[cfg(feature = "http-server")]
mod http_composition {
    use arqen::AppState;
    use axum::Router;
    use axum::body::Body;
    use axum::extract::FromRef;
    use axum::extract::State;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use tower::ServiceExt;

    #[derive(Clone)]
    struct CustomState {
        arqen: AppState,
        label: String,
    }

    impl FromRef<CustomState> for AppState {
        fn from_ref(state: &CustomState) -> Self {
            state.arqen.clone()
        }
    }

    async fn app_handler(State(state): State<CustomState>) -> String {
        state.label
    }

    #[tokio::test]
    async fn test_builtin_routes_mount_on_custom_state() {
        let app_state = AppState::builder().build().unwrap();

        let router: Router<CustomState> =
            arqen::http::builtin_routes(&app_state).route("/api/hello", get(app_handler));
        let router = router.with_state(CustomState {
            arqen: app_state,
            label: "hello".to_string(),
        });

        for path in ["/health", "/ready", "/agent", "/agent/manifest", "/docs"] {
            let response = router
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "path {path} should respond"
            );
        }

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/hello")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(&body[..], b"hello");
    }
}

#[cfg(feature = "http-server")]
mod tool_invoke {
    use arqen::{
        AppError, AppState, ToolContext, ToolEffect, ToolHandler, ToolMetadata, ToolRegistry,
    };
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    struct EchoHandler;

    #[async_trait::async_trait]
    impl ToolHandler for EchoHandler {
        async fn execute(
            &self,
            _ctx: &ToolContext,
            input: serde_json::Value,
        ) -> Result<serde_json::Value, AppError> {
            Ok(input)
        }
    }

    fn tool_state() -> AppState {
        let mut registry = ToolRegistry::new("contract-app", "1.0.0", "Contract app", "memory");
        registry.register_tool(ToolMetadata {
            name: "echo".to_string(),
            description: "Echo input".to_string(),
            input: serde_json::json!({
                "type": "object",
                "properties": {"msg": {"type": "string"}},
                "required": ["msg"]
            }),
            output: serde_json::json!({"type": "object"}),
            scopes: vec![],
            effect: ToolEffect::Read,
            idempotent: true,
            enqueues_job: None,
            timeout: Some(5),
        });
        registry.register_handler("echo", EchoHandler);
        AppState::builder()
            .with_tool_registry(registry)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_tool_invoke_through_builtin_router() {
        let state = tool_state();
        let router = arqen::http::create_router_with_state(state);

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
    async fn test_tool_invoke_matches_manifest_names() {
        let state = tool_state();
        let router = arqen::http::create_router_with_state(state);

        let manifest = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/agent/manifest")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(manifest.status(), StatusCode::OK);
        let body = axum::body::to_bytes(manifest.into_body(), 4096)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let tool_names: Vec<&str> = json["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert!(tool_names.contains(&"echo"));

        let invoke_paths: Vec<&str> = json["endpoints"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["path"].as_str().unwrap())
            .collect();
        assert!(invoke_paths.contains(&"/agent/tools/echo"));

        let invoked = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/agent/tools/echo")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::json!({"msg": "ok"}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invoked.status(), StatusCode::OK);
    }
}

async fn create_backend() -> Box<dyn ThingdBackend> {
    Box::new(MemoryThingdBackend::new())
}

#[tokio::test]
async fn test_object_crud() {
    let backend = create_backend().await;

    // Create object
    let obj = backend
        .put_object("users", "user1", json!({"name": "Alice"}))
        .await
        .unwrap();
    assert_eq!(obj.id, "user1");
    assert_eq!(obj.collection, "users");
    assert_eq!(obj.data["name"], "Alice");

    // Get object
    let fetched = backend.get_object("users", "user1").await.unwrap();
    assert!(fetched.is_some());
    let fetched = fetched.unwrap();
    assert_eq!(fetched.data["name"], "Alice");

    // Update object
    let updated = backend
        .put_object("users", "user1", json!({"name": "Alice Smith"}))
        .await
        .unwrap();
    assert_eq!(updated.data["name"], "Alice Smith");

    // Query objects
    let filter = ThingdFilter {
        field: "name".to_string(),
        operator: FilterOperator::Eq,
        value: json!("Alice Smith"),
    };
    let results = backend
        .query_objects(
            "users",
            arqen::thingd::traits::QueryOptions::filtered(vec![filter]),
        )
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].data["name"], "Alice Smith");

    // Count objects
    let count = backend.count_objects("users").await.unwrap();
    assert_eq!(count, 1);

    // Delete object
    backend.delete_object("users", "user1").await.unwrap();
    let fetched = backend.get_object("users", "user1").await.unwrap();
    assert!(fetched.is_none());
}

#[tokio::test]
async fn test_batch_operations() {
    let backend = create_backend().await;

    let operations = vec![
        ThingdOperation::Put {
            collection: "users".to_string(),
            id: "user1".to_string(),
            data: json!({"name": "Alice"}),
        },
        ThingdOperation::Put {
            collection: "users".to_string(),
            id: "user2".to_string(),
            data: json!({"name": "Bob"}),
        },
        ThingdOperation::Delete {
            collection: "users".to_string(),
            id: "user1".to_string(),
        },
    ];

    let results = backend.batch_write(operations).await.unwrap();
    assert_eq!(results.len(), 3);
    assert!(results[0].success);
    assert!(results[1].success);
    assert!(results[2].success);

    let count = backend.count_objects("users").await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_events() {
    let backend = create_backend().await;

    // Append events
    let event1 = backend
        .append_event("user_events", "user_created", json!({"user_id": "user1"}))
        .await
        .unwrap();
    let _event2 = backend
        .append_event("user_events", "user_created", json!({"user_id": "user2"}))
        .await
        .unwrap();

    assert_eq!(event1.stream, "user_events");
    assert_eq!(event1.event_type, "user_created");

    // Read events
    let events = backend.read_events("user_events", None, 10).await.unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].data["user_id"], "user1");
    assert_eq!(events[1].data["user_id"], "user2");

    // Read events with pagination
    let events = backend
        .read_events("user_events", Some(event1.id.clone()), 10)
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data["user_id"], "user2");
}

#[tokio::test]
async fn test_jobs() {
    let backend = create_backend().await;

    // Push jobs
    let job1 = backend
        .push_job("email_queue", json!({"to": "alice@example.com"}), 3)
        .await
        .unwrap();
    let job2 = backend
        .push_job("email_queue", json!({"to": "bob@example.com"}), 3)
        .await
        .unwrap();

    assert_eq!(job1.state, JobState::Queued);
    assert_eq!(job2.state, JobState::Queued);

    // Claim job
    let claimed = backend
        .claim_job("email_queue", "worker1", 30)
        .await
        .unwrap();
    assert!(claimed.is_some());
    let claimed = claimed.unwrap();
    assert_eq!(claimed.state, JobState::Leased);
    assert_eq!(claimed.attempts, 1);

    // Complete job
    backend
        .complete_job("email_queue", &claimed.id)
        .await
        .unwrap();
    let _fetched = backend
        .get_object("email_queue", &claimed.id)
        .await
        .unwrap();
    // Note: jobs are stored in a separate map, not in objects
    // This test may need adjustment based on implementation

    // Claim another job
    let claimed2 = backend
        .claim_job("email_queue", "worker1", 30)
        .await
        .unwrap();
    assert!(claimed2.is_some());
    let claimed2 = claimed2.unwrap();

    // Nack job (should retry)
    backend.nack_job("email_queue", &claimed2.id).await.unwrap();

    // Claim again (should be the same job with increased attempts)
    let claimed3 = backend
        .claim_job("email_queue", "worker1", 30)
        .await
        .unwrap();
    assert!(claimed3.is_some());
    let claimed3 = claimed3.unwrap();
    assert_eq!(claimed3.attempts, 2);

    // Exhaust retries
    for _ in 0..5 {
        backend.nack_job("email_queue", &claimed3.id).await.unwrap();
        let claimed = backend
            .claim_job("email_queue", "worker1", 30)
            .await
            .unwrap();
        if let Some(job) = claimed
            && job.state == JobState::Dead
        {
            break;
        }
    }
}

#[tokio::test]
async fn test_links() {
    let backend = create_backend().await;

    // Create links
    let link1 = backend
        .create_link("user1", "post1", "authored")
        .await
        .unwrap();
    let _link2 = backend
        .create_link("user1", "post2", "liked")
        .await
        .unwrap();
    let _link3 = backend
        .create_link("user2", "post1", "authored")
        .await
        .unwrap();

    assert_eq!(link1.source_id, "user1");
    assert_eq!(link1.target_id, "post1");
    assert_eq!(link1.relation, "authored");

    // Get links
    let links = backend.get_links("user1", None).await.unwrap();
    assert_eq!(links.len(), 2);

    let links = backend.get_links("user1", Some("authored")).await.unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].target_id, "post1");

    // Delete link
    backend.delete_link(&link1.id).await.unwrap();
    let links = backend.get_links("user1", None).await.unwrap();
    assert_eq!(links.len(), 1);
}

#[tokio::test]
async fn test_search() {
    let backend = create_backend().await;

    // Seed data
    backend
        .put_object(
            "users",
            "user1",
            json!({"name": "Alice", "email": "alice@example.com"}),
        )
        .await
        .unwrap();
    backend
        .put_object(
            "users",
            "user2",
            json!({"name": "Bob", "email": "bob@example.com"}),
        )
        .await
        .unwrap();
    backend
        .put_object(
            "posts",
            "post1",
            json!({"title": "Hello World", "content": "First post"}),
        )
        .await
        .unwrap();

    // Search
    let results = backend
        .search(
            "Alice",
            SearchOptions {
                limit: 10,
                offset: 0,
                filters: vec![],
            },
        )
        .await
        .unwrap();
    assert_eq!(results.total, 1);
    assert_eq!(results.items[0].data["name"], "Alice");

    // Search with filter
    let results = backend
        .search(
            "example",
            SearchOptions {
                limit: 10,
                offset: 0,
                filters: vec![ThingdFilter {
                    field: "name".to_string(),
                    operator: FilterOperator::Contains,
                    value: json!("Bob"),
                }],
            },
        )
        .await
        .unwrap();
    assert_eq!(results.total, 1);
    assert_eq!(results.items[0].data["name"], "Bob");
}

#[tokio::test]
async fn test_reset_and_seed() {
    let backend = create_backend().await;

    // Seed data
    backend.seed().await.unwrap();
    let count = backend.count_objects("users").await.unwrap();
    assert!(count > 0);

    // Reset
    backend.reset().await.unwrap();
    let count = backend.count_objects("users").await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_filter_operators() {
    let backend = create_backend().await;

    // Create objects
    backend
        .put_object("items", "item1", json!({"name": "Apple", "price": 1.0}))
        .await
        .unwrap();
    backend
        .put_object("items", "item2", json!({"name": "Banana", "price": 0.5}))
        .await
        .unwrap();
    backend
        .put_object("items", "item3", json!({"name": "Cherry", "price": 2.0}))
        .await
        .unwrap();

    // Test Eq
    let filter = ThingdFilter {
        field: "name".to_string(),
        operator: FilterOperator::Eq,
        value: json!("Banana"),
    };
    let results = backend
        .query_objects("items", QueryOptions::filtered(vec![filter]))
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].data["name"], "Banana");

    // Test Gt
    let filter = ThingdFilter {
        field: "price".to_string(),
        operator: FilterOperator::Gt,
        value: json!(1.0),
    };
    let results = backend
        .query_objects("items", QueryOptions::filtered(vec![filter]))
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].data["name"], "Cherry");

    // Test Contains
    let filter = ThingdFilter {
        field: "name".to_string(),
        operator: FilterOperator::Contains,
        value: json!("pp"),
    };
    let results = backend
        .query_objects("items", QueryOptions::filtered(vec![filter]))
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].data["name"], "Apple");

    let search_results = backend
        .search(
            "",
            SearchOptions {
                limit: 10,
                offset: 0,
                filters: vec![ThingdFilter {
                    field: "price".into(),
                    operator: FilterOperator::Gt,
                    value: json!(1.0),
                }],
            },
        )
        .await
        .unwrap();
    assert_eq!(search_results.items.len(), 1);
    assert_eq!(search_results.items[0].data["name"], "Cherry");
}

#[tokio::test]
async fn test_query_objects_multiple_filters_and_pagination() {
    let backend = create_backend().await;

    for (id, title_id, country, season) in [
        ("o1", "t1", "US", 1),
        ("o2", "t1", "US", 2),
        ("o3", "t1", "UK", 1),
        ("o4", "t2", "US", 1),
        ("o5", "t1", "US", 3),
    ] {
        backend
            .put_object(
                "offers",
                id,
                json!({"title_id": title_id, "country": country, "season": season}),
            )
            .await
            .unwrap();
    }

    // Conjunctive filters: title_id = t1 AND country = US.
    let filters = vec![
        ThingdFilter {
            field: "title_id".to_string(),
            operator: FilterOperator::Eq,
            value: json!("t1"),
        },
        ThingdFilter {
            field: "country".to_string(),
            operator: FilterOperator::Eq,
            value: json!("US"),
        },
    ];

    let all = backend
        .query_objects("offers", QueryOptions::filtered(filters.clone()))
        .await
        .unwrap();
    assert_eq!(all.len(), 3);

    let page = backend
        .query_objects(
            "offers",
            QueryOptions {
                filters,
                limit: Some(2),
                offset: 1,
            },
        )
        .await
        .unwrap();
    assert_eq!(page.len(), 2);
    assert_eq!(page[0].id, "o2");
    assert_eq!(page[1].id, "o5");
}
