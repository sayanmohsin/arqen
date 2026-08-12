#![cfg(feature = "http-client")]

use arqen::thingd::{
    FilterOperator, HttpThingdBackend, QueryOptions, SearchOptions, ThingdBackend, ThingdFilter,
    ThingdOperation,
};

/// Run against a real thingd-server with:
/// `ARQEN_THINGD_TEST_URL=http://127.0.0.1:8757 cargo test -p arqen --test http_thingd_contract -- --ignored`
#[tokio::test]
#[ignore = "requires a running thingd-server"]
async fn thingd_server_public_rest_contract() {
    let base_url = std::env::var("ARQEN_THINGD_TEST_URL")
        .expect("ARQEN_THINGD_TEST_URL must point at thingd-server");
    let backend = HttpThingdBackend::new(&base_url);
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
