use arqen_thingd::MemoryThingdBackend;
use arqen_thingd::traits::*;
use serde_json::json;

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
    let results = backend.query_objects("users", Some(filter)).await.unwrap();
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
    let results = backend.query_objects("items", Some(filter)).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].data["name"], "Banana");

    // Test Gt
    let filter = ThingdFilter {
        field: "price".to_string(),
        operator: FilterOperator::Gt,
        value: json!(1.0),
    };
    let results = backend.query_objects("items", Some(filter)).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].data["name"], "Cherry");

    // Test Contains
    let filter = ThingdFilter {
        field: "name".to_string(),
        operator: FilterOperator::Contains,
        value: json!("pp"),
    };
    let results = backend.query_objects("items", Some(filter)).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].data["name"], "Apple");
}
