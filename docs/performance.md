# Performance

Arqen provides reproducible Criterion benchmarks for core framework operations.

## Running benchmarks

```bash
cargo bench --workspace --all-features --bench framework -- --noplot
```

Reports are written to `target/criterion/` with HTML reports and estimates.

## Benchmark methodology

### Environment

- **Recorded**: 2026-10-01, Thingd 0.91.1; Arqen checkout based on `e5b11c9` with the upgrade changes
- **OS/architecture**: macOS 27.0.1, arm64
- **Rust**: `rustc 1.98.1 (48a229cea 2026-09-01)`
- **Feature flags**: `--all-features`
- **Storage mode**: memory, native, persistent, or cache (as named by each benchmark)
- **Sample size**: 100 iterations (Criterion default)
- **Warm-up**: 3 seconds per benchmark
- **Measurement time**: 5 seconds per benchmark

### Workloads

| Workload                                    | Description                                        | Fixture                                   |
| ------------------------------------------- | -------------------------------------------------- | ----------------------------------------- |
| `routing/health_route`                      | End-to-end GET /health through Arqen               | Health registry with AlwaysHealthy checks |
| `manifest/100_tools`                        | Generate manifest with 100 tools + JSON serialize  | 100 ToolMetadata entries                  |
| `validation/3_fields`                       | Validate a struct with 3 fields (extensible)       | BenchPayload struct                       |
| `thingd_memory/put_object`                  | Insert object into MemoryThingdBackend             | Single object                             |
| `thingd_memory/get_object`                  | Fetch object from MemoryThingdBackend              | Pre-populated store                       |
| `thingd_memory/query_objects`               | Query all objects from a collection                | 100 objects                               |
| `thingd_native/put_object`                  | Async adapter over native thingd                   | In-memory native engine                   |
| `thingd_native/get_object`                  | Async adapter over native thingd                   | Pre-populated native engine               |
| `thingd_persistent/rocksdb_put_object`      | Durable write through native adapter               | Thingd RocksDB engine, search disabled    |
| `thingd_persistent/thingdb_put_object`      | Durable write through native adapter               | ThingDB engine, search disabled           |
| `thingd_cache/hit`                          | Read-through cache hit                             | Memory source and cache                   |
| `jobs/enqueue_dequeue`                      | Push + claim + complete a job                      | Memory backend                            |
| `health/10_checks`                          | Run liveness check with 10 dependencies            | 10 AlwaysHealthy checks                   |
| `performance/request_metrics_record`        | Hot-path request counter and bounded sample update | RequestMetrics instance                   |
| `performance/thingd_memory_batch_write_100` | Batch write of 100 objects                         | Memory backend                            |

### Estimates and percentiles

Criterion’s `estimates.json` records statistical estimates for the median and
mean. The table below records the median from the latest local run; it is not a
production latency guarantee. This harness does not publish p95/p99 samples,
so p95 budgets below require a separate distribution or service-level
benchmark. Raw Criterion data is available in
`target/criterion/<group>/<id>/new/estimates.json` after a local run.

### Latest local run

Measured on the environment above with the command shown in [Running benchmarks](#running-benchmarks).
Values are median estimates from Criterion.

| Workload                                    |     Median |
| ------------------------------------------- | ---------: |
| `routing/health_route`                      |  80.125 µs |
| `manifest/100_tools`                        | 231.622 µs |
| `validation/3_fields`                       |   2.899 ns |
| `thingd_memory/put_object`                  | 530.323 ns |
| `thingd_memory/get_object`                  | 274.600 ns |
| `thingd_memory/query_objects`               |  21.962 µs |
| `thingd_native/put_object`                  |   7.096 µs |
| `thingd_native/get_object`                  |   5.212 µs |
| `thingd_persistent/rocksdb_put_object`      |  48.772 µs |
| `thingd_persistent/thingdb_put_object`      |   5.284 ms |
| `thingd_cache/hit`                          | 466.173 ns |
| `jobs/enqueue_dequeue`                      |   4.027 µs |
| `health/10_checks`                          |  10.815 µs |
| `performance/request_metrics_record`        | 105.156 ns |
| `performance/thingd_memory_batch_write_100` |  68.962 µs |

Against the available local Criterion baseline, manifest generation regressed
3.1%, memory reads regressed 3.1%, and native in-memory writes regressed 8.2%.
Validation improved 19.9%, health checks improved 5.4%, and batch writes
improved 3.0%. Routing, memory writes and queries, native reads, cache hits,
jobs, and request metrics were unchanged or within Criterion's noise threshold.
The persistent workloads have no prior baseline: RocksDB measured 48.8 µs and
ThingDB 5.284 ms per durable write with default synchronous persistence and
search disabled. This local single-record result is diagnostic, not a
production comparison; measure representative datasets and hardware before
choosing an engine.

## Performance budgets

| Workload                      | Target    | Notes                      |
| ----------------------------- | --------- | -------------------------- |
| In-memory health route        | p95 < 1ms | Benchmark environment only |
| In-memory manifest generation | p95 < 2ms | 100 tools                  |
| In-memory object CRUD         | p95 < 2ms | Single object operations   |
| Job enqueue/dequeue           | p95 < 2ms | Memory backend             |

The 0.10 release gate additionally compares representative before/after p95
results for routing, metrics, Thingd reads/writes, batch writes, and job
processing. The target is at least 30% lower p95 for a measured bottleneck;
where the baseline is already below the noise floor, the requirement is no
regression plus bounded resource usage.

These are benchmark-harness targets, not production guarantees. The native
benchmarks use the in-memory native engine to isolate adapter overhead; run a
separate persistent-path benchmark before choosing disk settings. HTTP latency
must be measured against the deployed thingd service because network distance,
TLS, pooling, and server load dominate the result.

Thingd 0.91.1 exposes both RocksDB and its experimental Rust-native ThingDB
backend behind `PersistentBackend`. Arqen keeps RocksDB as its default and
benchmarks durable writes against both engines with search indexing disabled to
focus on storage write cost. These local results do not substitute for
application-specific durability, recovery, or production disk tests.

## HTTP response performance

Arqen enables Brotli/gzip compression for responses above the configured
threshold. Use `ARQEN_COMPRESSION_ENABLED` and
`ARQEN_COMPRESSION_THRESHOLD` to tune it after measuring CPU and network
cost. Responses that opt into `HttpCachePolicy` can use ETags and return `304`
without transferring the body.

For large result sets, use the `jsonl_response` helper. It serializes one
record at a time as `application/x-ndjson` instead of materializing the whole
response. HTTP range filters are bounded by
`ARQEN_THINGD_MAX_QUERY_SCAN_OBJECTS`; exceeding that bound is an explicit
error rather than a partial response.

## Limitations

- HTTP sidecar latency is not included in this harness; set up a service-level
  benchmark for the target deployment.
- Use `ARQEN_THINGD_MAX_CONCURRENCY` to bound sidecar pressure and benchmark
  it against the Thingd server's own worker and connection limits.
- Allocation counts are not measured in this phase (would require an instrumented
  allocator).
- Network I/O, disk I/O, and external service latency are not represented.
- Criterion provides statistical estimates; exact percentiles may vary between runs.

## Adding new benchmarks

Add a new function to `crates/arqen/benches/framework.rs` and register it in
the `criterion_group!` macro. Follow the existing pattern:

1. Create a group with `c.benchmark_group("name")`
2. Add benchmarks with `group.bench_function("id", |b| { ... })`
3. Call `group.finish()`

To change sample size or warm-up, use `group.sample_size(n)` or
`group.warm_up_time(Duration)`.
