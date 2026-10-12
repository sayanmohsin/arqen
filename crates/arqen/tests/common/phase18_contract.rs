use std::sync::Arc;

use arqen::core::ErrorKind;
use arqen::thingd::{
    PushJobOptions, QueryOptions, ScopedThingdBackend, SearchOptions, ThingdBackend,
    ThingdOperation,
};
use serde_json::json;

/// Shared isolation and mutation contract for every backend that can provide
/// the required operations. The caller supplies a fresh backend or a unique
/// namespace when running this against a shared Thingd server.
pub async fn run_phase18_contract(inner: Arc<dyn ThingdBackend>) {
    let suffix = uuid::Uuid::new_v4().to_string();
    let tenant = format!("tenant-{suffix}");
    let instance = format!("instance-{suffix}");
    let collection = format!("records-{suffix}");
    let stream = format!("activity-{suffix}");
    let queue = format!("work-{suffix}");
    let search_term = format!("needle-{suffix}");

    let owner = ScopedThingdBackend::user(Arc::clone(&inner), &tenant, &instance, "subject-a");
    let other_scopes = [
        ScopedThingdBackend::user(
            Arc::clone(&inner),
            format!("{tenant}-other"),
            &instance,
            "subject-a",
        ),
        ScopedThingdBackend::user(
            Arc::clone(&inner),
            &tenant,
            format!("{instance}-other"),
            "subject-a",
        ),
        ScopedThingdBackend::user(Arc::clone(&inner), &tenant, &instance, "subject-b"),
    ];

    owner
        .put_object(&collection, "same-id", json!({"text": search_term}))
        .await
        .unwrap();
    let owner_query = owner
        .query_objects(&collection, QueryOptions::default())
        .await
        .unwrap();
    assert_eq!(owner_query.len(), 1);
    assert_eq!(owner_query[0].id, "same-id");
    let owner_search = owner
        .search(
            &search_term,
            SearchOptions {
                limit: 10,
                offset: 0,
                filters: vec![],
            },
        )
        .await
        .unwrap();
    assert_eq!(owner_search.items.len(), 1);

    owner
        .put_object_if_version(&collection, "cas", json!({"version": 1}), 0)
        .await
        .unwrap();
    let initial = owner
        .get_object_versioned(&collection, "cas")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(initial.version, 1);
    owner
        .put_object_if_version(&collection, "cas", json!({"version": 2}), initial.version)
        .await
        .unwrap();
    let stale = owner
        .put_object_if_version(&collection, "cas", json!({"version": 3}), initial.version)
        .await
        .unwrap_err();
    assert_eq!(stale.kind, ErrorKind::Conflict);

    owner
        .append_event(&stream, "created", json!({"value": search_term}))
        .await
        .unwrap();
    let key = format!("phase18-{suffix}");
    let first_job = owner
        .push_job_with_options(
            &queue,
            json!({"value": "first"}),
            3,
            PushJobOptions {
                idempotency_key: Some(key.clone()),
                delay_ms: None,
            },
        )
        .await
        .unwrap();
    let replayed_job = owner
        .push_job_with_options(
            &queue,
            json!({"value": "replay"}),
            3,
            PushJobOptions {
                idempotency_key: Some(key),
                delay_ms: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(replayed_job.id, first_job.id);
    assert_eq!(replayed_job.payload, json!({"value": "first"}));

    let link = owner
        .create_link("source", "target", "related")
        .await
        .unwrap();
    owner
        .batch_write(vec![ThingdOperation::Put {
            collection: collection.clone(),
            id: "batch-only".to_string(),
            data: json!({"value": "private"}),
        }])
        .await
        .unwrap();

    for other in other_scopes {
        assert!(
            other
                .get_object(&collection, "same-id")
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            other
                .query_objects(&collection, QueryOptions::default())
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            other
                .search(
                    &search_term,
                    SearchOptions {
                        limit: 10,
                        offset: 0,
                        filters: vec![],
                    },
                )
                .await
                .unwrap()
                .items
                .is_empty()
        );
        assert!(
            other
                .read_events(&stream, None, 10)
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            other
                .claim_job(&queue, "phase18-worker", 30)
                .await
                .unwrap()
                .is_none()
        );
        assert!(other.get_links("source", None).await.unwrap().is_empty());
        assert!(
            other
                .get_object(&collection, "batch-only")
                .await
                .unwrap()
                .is_none()
        );
        let isolated_create = other
            .put_object_if_version(&collection, "cas", json!({"version": 99}), 0)
            .await
            .expect("version zero creates an object inside each independent scope");
        assert_eq!(isolated_create.data["version"], 99);
    }

    let claimed = owner
        .claim_job(&queue, "phase18-worker", 30)
        .await
        .unwrap()
        .expect("one job should exist after replay");
    assert_eq!(claimed.id, first_job.id);
    assert_eq!(claimed.payload, json!({"value": "first"}));
    assert!(
        owner
            .claim_job(&queue, "phase18-worker", 30)
            .await
            .unwrap()
            .is_none()
    );
    owner.delete_link(&link.id).await.unwrap();
}
