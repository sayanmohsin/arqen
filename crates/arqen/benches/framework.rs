use arqen::ThingdBackend;
use criterion::{Criterion, criterion_group, criterion_main};
use std::sync::Arc;

fn bench_routing_health(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("routing");
    group.bench_function("health_route", |b| {
        b.iter(|| {
            rt.block_on(async {
                let state = arqen::AppState::builder().build().unwrap();
                let router = arqen::create_router_with_state(state);
                use tower::ServiceExt;
                let response = router
                    .oneshot(
                        axum::http::Request::builder()
                            .uri("/health")
                            .body(axum::body::Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(response.status(), 200);
            });
        });
    });
    group.finish();
}

fn bench_manifest_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("manifest");
    group.bench_function("100_tools", |b| {
        b.iter(|| {
            let mut registry = arqen::ToolRegistry::new("bench", "0.1.0", "bench", "memory");
            for i in 0..100 {
                registry.register_tool(arqen::ToolMetadata {
                    name: format!("tool_{}", i),
                    description: format!("Benchmark tool {}", i),
                    input: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}}}),
                    output: serde_json::json!({"type": "object"}),
                    scopes: vec![],
                    effect: arqen::ToolEffect::Read,
                    idempotent: true,
                    enqueues_job: None,
                    timeout: None,
                });
            }
            let manifest = registry.generate_manifest();
            let _json = serde_json::to_value(&manifest).unwrap();
        });
    });
    group.finish();
}

fn bench_validation(c: &mut Criterion) {
    use arqen::{FieldError, Validate, ValidationErrors};

    struct BenchPayload {
        name: String,
        email: String,
        age: u32,
    }

    impl Validate for BenchPayload {
        fn validate(&self) -> Result<(), ValidationErrors> {
            let mut errors = Vec::new();
            if self.name.is_empty() {
                errors.push(FieldError::new("name", "required", "name is required"));
            }
            if !self.email.contains('@') {
                errors.push(FieldError::new("email", "email", "invalid email"));
            }
            if self.age > 150 {
                errors.push(FieldError::new("age", "max_value", "age too high"));
            }
            if errors.is_empty() {
                Ok(())
            } else {
                Err(ValidationErrors { errors })
            }
        }
    }

    let mut group = c.benchmark_group("validation");
    group.bench_function("3_fields", |b| {
        let payload = BenchPayload {
            name: "test".into(),
            email: "a@b.c".into(),
            age: 25,
        };
        b.iter(|| {
            let _ = payload.validate();
        });
    });
    group.finish();
}

fn bench_thingd_crud(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("thingd_memory");

    group.bench_function("put_object", |b| {
        let backend = arqen::MemoryThingdBackend::new();
        b.iter(|| {
            rt.block_on(async {
                backend
                    .put_object("bench", "item1", serde_json::json!({"data": "value"}))
                    .await
                    .unwrap();
            });
        });
    });

    group.bench_function("get_object", |b| {
        let backend = arqen::MemoryThingdBackend::new();
        rt.block_on(async {
            backend
                .put_object("bench", "item1", serde_json::json!({"data": "value"}))
                .await
                .unwrap();
        });
        b.iter(|| {
            rt.block_on(async {
                backend.get_object("bench", "item1").await.unwrap();
            });
        });
    });

    group.bench_function("query_objects", |b| {
        let backend = arqen::MemoryThingdBackend::new();
        rt.block_on(async {
            for i in 0..100 {
                backend
                    .put_object(
                        "bench",
                        &format!("item{}", i),
                        serde_json::json!({"idx": i, "name": format!("item{}", i)}),
                    )
                    .await
                    .unwrap();
            }
        });
        b.iter(|| {
            rt.block_on(async {
                use arqen::thingd::traits::QueryOptions;
                let _ = backend
                    .query_objects("bench", QueryOptions::default())
                    .await
                    .unwrap();
            });
        });
    });

    group.finish();
}

fn bench_jobs(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("jobs");
    group.bench_function("enqueue_dequeue", |b| {
        let backend = arqen::MemoryThingdBackend::new();
        b.iter(|| {
            rt.block_on(async {
                let _job = backend
                    .push_job("bench_queue", serde_json::json!({"task": "work"}), 3)
                    .await
                    .unwrap();
                let claimed = backend
                    .claim_job("bench_queue", "worker1", 30)
                    .await
                    .unwrap()
                    .unwrap();
                backend
                    .complete_job("bench_queue", &claimed.id)
                    .await
                    .unwrap();
            });
        });
    });
    group.finish();
}

fn bench_health(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("health");
    group.bench_function("10_checks", |b| {
        let mut registry = arqen::HealthRegistry::new();
        for _ in 0..10 {
            registry.register(Arc::new(arqen::health::AlwaysHealthy));
        }
        b.iter(|| {
            rt.block_on(async {
                let _ = registry.check_liveness().await;
            });
        });
    });
    group.finish();
}

criterion_group!(
    framework,
    bench_routing_health,
    bench_manifest_generation,
    bench_validation,
    bench_thingd_crud,
    bench_jobs,
    bench_health,
);
criterion_main!(framework);
