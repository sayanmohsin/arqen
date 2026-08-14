# Deployment

Deployment guidance for Arqen applications.

## Deployment modes

### Memory mode (development)

No external dependencies. Suitable for local development and CI.

```bash
arqen dev --storage memory
```

- In-memory thingd engine
- Process-local and disposable
- No persistence across restarts

### Native durable thingd

Native mode embeds Thingd's durable RocksDB engine (plus Tantivy search) in the
application process. It is intended for durable deployments with at least 4 GB
RAM (`e2-medium` or larger); the native store alone can consume hundreds of MB
and Linux OOM kills commonly appear only as exit code 137. For e2-micro and
other small VMs, use HTTP mode and run thingd separately. See the thingd memory
and native-store compatibility documentation for the store-specific
requirements. Existing Fjall directories must be migrated with
`thingd-migrate` before they can be opened by Thingd 0.83.1.

Use the `native` storage mode for local durable storage without a
separate thingd service.

```toml
[storage]
mode = "native"
persistent_path = "/var/lib/arqen/data"
```

```bash
ARQEN_STORAGE_MODE=native ARQEN_PERSISTENT_PATH=/var/lib/arqen/data arqen start
```

### HTTP sidecar

HTTP mode is the recommended production path for small VMs. The application
does not open the native store, and the remote thingd service owns its schema,
index lifecycle, memory budget, and backup format.

Connect to an external thingd service over HTTP:

```toml
[storage]
mode = "http"
http_url = "http://thingd:8080"
```

```bash
ARQEN_STORAGE_MODE=http ARQEN_THINGD_URL=http://thingd:8080 arqen start
```

Configure the Thingd HTTP service for asynchronous search indexing:

```env
THINGD_SEARCH_MODE=persistent-async
THINGD_SEARCH_COMMIT_INTERVAL_MS=250
THINGD_SEARCH_COMMIT_BATCH_SIZE=32
THINGD_SEARCH_QUEUE_MAX_KEYS=10000
```

Writes are durable before asynchronous search indexing catches up. Keep
`/ready` in the deployment probe and treat Thingd `503 Retry-After: 1` as
transient during recovery; Arqen retries bounded mutation and bootstrap work.

For a native deployment, stop Arqen before making a backup so the lock and all
store files are consistent:

```bash
arqen store export --data-dir /var/lib/arqen/data --output /var/backups/arqen-$(date +%Y%m%d).tar.gz
arqen store import --data-dir /var/lib/arqen/data --input /var/backups/arqen-20260811.tar.gz
```

The CLI includes lock metadata and excludes macOS `._*` and `.DS_Store`
artifacts. Restore ownership to the runtime user after transferring an
archive; the CLI does not assume a container UID.

### Cloud (future)

Cloud integration is optional. A hosted thingd-cloud adapter must use a
documented public customer API, not control-plane databases or private
modules.

Before using cloud storage for a multi-user application, validate the
application-hardening requirements: production configuration guardrails,
tenant/instance identity, scoped repositories, HTTP contract tests, request
idempotency, conditional writes, backups, and a separate worker role. See
[application-hardening.md](application-hardening.md).

## Docker deployment

Build a release binary and containerize it:

```dockerfile
FROM rust:1.96 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features cli

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/arqen /usr/local/bin/
EXPOSE 8888
CMD ["arqen", "start"]
```

Build and run:

```bash
docker build -t my-arqen-app .
docker run -p 8888:8888 \
  -e ARQEN_STORAGE_MODE=http \
  -e ARQEN_THINGD_URL=http://thingd:8080 \
  my-arqen-app
```

## Environment variables for production

For high-throughput services, use JSON logs with the default 1% successful
request sample, keep the 250ms slow-request threshold, and leave response
compression enabled for responses above 1KiB. Tune
`ARQEN_REQUEST_LOG_SAMPLE_RATE`, `ARQEN_SLOW_REQUEST_THRESHOLD_MS`, and
`ARQEN_COMPRESSION_THRESHOLD` only after reviewing benchmark and collector
capacity. HTTP Thingd clients default to 16 concurrent requests and workers
are bounded by `ARQEN_WORKER_CONCURRENCY`.

| Variable                  | Recommended value        |
| ------------------------- | ------------------------ |
| `ARQEN_HOST`              | `0.0.0.0`                |
| `ARQEN_PORT`              | `8888`                   |
| `ARQEN_STORAGE_MODE`      | `http` or `native`       |
| `ARQEN_THINGD_URL`        | thingd service URL       |
| `ARQEN_THINGD_AUTH_TOKEN` | server-side thingd token |
| `ARQEN_LOG_LEVEL`         | `warn` or `info`         |
| `ARQEN_LOG_FORMAT`        | `json`                   |
| `ARQEN_JWT_SECRET`        | secret value             |

## Health and readiness

Orchestrators use health endpoints for lifecycle decisions:

- `GET /health` - liveness probe (restart the process if failing)
- `GET /ready` - readiness probe (stop routing traffic if failing)

Kubernetes example:

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8888
  initialDelaySeconds: 5
  periodSeconds: 10
readinessProbe:
  httpGet:
    path: /ready
    port: 8888
  initialDelaySeconds: 3
  periodSeconds: 5
```

## Checklist

Every deployment should address:

- Release builds (`cargo build --release`)
- Environment variables documented above
- Secret management (env vars or secrets manager, never committed)
- Health and readiness checks
- Graceful shutdown (`ARQEN_SHUTDOWN_TIMEOUT`)
- Worker scaling (`ARQEN_WORKER_CONCURRENCY`)
- thingd connectivity and credentials
- Structured log collection (JSON format)
- Production configuration validation (`AppConfig::validate_production()`)
- Native memory and store compatibility requirements
- Durable storage and backup ownership
- Tenant/instance routing and isolation tests
- Conditional-write and idempotency behavior
- Cloud API contract/version compatibility
- Queue lag, dead-letter, and worker health monitoring
