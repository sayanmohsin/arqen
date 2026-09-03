# Configuration

Arqen applications are configured through environment variables and optional configuration files.

## Environment variables

| Variable                              | Description                                                        | Default                              |
| ------------------------------------- | ------------------------------------------------------------------ | ------------------------------------ |
| `ARQEN_HOST`                          | Bind address for the HTTP server                                   | `127.0.0.1`                          |
| `ARQEN_PORT`                          | Port for the HTTP server                                           | `8888`                               |
| `ARQEN_STORAGE_MODE`                  | Storage mode: `memory`, `native`, `persistent`, `http`, or `cloud` | `memory`                             |
| `ARQEN_PERSISTENT_PATH`               | Native durable thingd storage path                                 | unset; required for `persistent`     |
| `ARQEN_THINGD_URL`                    | thingd HTTP service URL                                            | unset; required for `http`           |
| `ARQEN_THINGD_MAX_CONCURRENCY`        | Maximum active HTTP Thingd requests                                | `16`                                 |
| `ARQEN_THINGD_REQUEST_TIMEOUT`        | HTTP Thingd request timeout in seconds                             | `30`                                 |
| `ARQEN_THINGD_MAX_RETRIES`            | Maximum retries for safe/transient Thingd requests                 | `2`                                  |
| `ARQEN_THINGD_MAX_RETRY_DURATION`     | Maximum total retry duration in seconds                            | `30`                                 |
| `ARQEN_THINGD_MAX_QUERY_SCAN_OBJECTS` | Maximum objects an HTTP range query may scan                       | `100000`                             |
| `ARQEN_CLOUD_URL`                     | Future public thingd.cloud endpoint                                | unset; cloud mode is not implemented |
| `ARQEN_THINGD_AUTH_TOKEN`             | Server-side thingd/cloud bearer token                              | unset; never log or commit           |
| `ARQEN_THINGD_ENCRYPTION_KEY`         | 64-hex-character native Thingd encryption key                      | unset; server-side only              |
| `ARQEN_THINGD_SCHEMA_PATH`            | Versioned `.thingd` schema path                                    | unset                                |
| `ARQEN_THINGD_CACHE_ENABLED`          | Enable the allowlisted catalog read cache                          | `false`                              |
| `ARQEN_THINGD_CACHE_COLLECTIONS`      | Comma-separated collections permitted in the cache                 | unset; required when enabled         |
| `ARQEN_SYNC_ENABLED`                  | Enable opt-in Thingd source-to-replica sync                        | `false`                              |
| `ARQEN_SYNC_MODE`                     | Sync capability: `disabled`, `http`, or `native`                   | `disabled`                           |
| `ARQEN_SYNC_SOURCE_ID`                | Stable source instance identifier                                  | unset                                |
| `ARQEN_SYNC_TARGET_URL`               | Thingd replication target URL                                      | unset; required when enabled         |
| `ARQEN_SYNC_TARGET_AUTH_TOKEN`        | Target bearer credential                                           | unset; required by target policy     |
| `ARQEN_SYNC_COLLECTIONS`              | Comma-separated replication allowlist                              | empty                                |
| `ARQEN_SYNC_REPLICATE_ALL`            | Explicitly replicate all supported application collections         | `false`                              |
| `ARQEN_SYNC_POLL_INTERVAL`            | Sync polling interval in seconds                                   | `5`                                  |
| `ARQEN_SYNC_BATCH_SIZE`               | Maximum changes per replication page                               | `500`                                |
| `ARQEN_SYNC_SNAPSHOT_FALLBACK`        | Bootstrap stale replicas from a snapshot                           | `true`                               |
| `ARQEN_JWT_SECRET`                    | JWT secret, kept redacted in configuration output                  | unset                                |
| `ARQEN_API_KEY_HEADER`                | API-key request header                                             | `X-API-Key`                          |
| `ARQEN_LOG_LEVEL`                     | Log level                                                          | `info`                               |
| `ARQEN_LOG_FORMAT`                    | Log format (`pretty`, `json`, `compact`)                           | `pretty`                             |
| `ARQEN_SERVICE_NAME`                  | Stable service name included in structured request logs            | package name                         |
| `ARQEN_WORKER_ENABLED`                | Enable workers                                                     | implementation default               |
| `ARQEN_WORKER_QUEUES`                 | Comma-separated worker queues                                      | implementation default               |
| `ARQEN_WORKER_POLL_INTERVAL`          | Worker polling interval                                            | implementation default               |
| `ARQEN_WORKER_LEASE_SECONDS`          | Job lease duration                                                 | implementation default               |
| `ARQEN_WORKER_MAX_RETRIES`            | Maximum job retries                                                | implementation default               |
| `ARQEN_WORKER_CONCURRENCY`            | Worker concurrency                                                 | implementation default               |
| `ARQEN_HEALTH_CHECK_TIMEOUT`          | Dependency health-check timeout                                    | implementation default               |
| `ARQEN_HEALTH_STARTUP_DELAY`          | Startup delay before health checks                                 | implementation default               |
| `ARQEN_REQUEST_TIMEOUT`               | HTTP request timeout                                               | `30s`                                |
| `ARQEN_REQUEST_LOG_SAMPLE_RATE`       | Successful request log sample rate                                 | `0.01`                               |
| `ARQEN_SLOW_REQUEST_THRESHOLD_MS`     | Always-log request duration threshold                              | `250`                                |
| `ARQEN_COMPRESSION_THRESHOLD`         | Minimum response size for gzip/Brotli compression (bytes)          | `1024`                               |
| `ARQEN_COMPRESSION_ENABLED`           | Enable gzip/Brotli response compression                            | `true`                               |
| `ARQEN_MAX_BODY_SIZE`                 | Maximum request body size                                          | `1048576`                            |
| `ARQEN_SHUTDOWN_TIMEOUT`              | Graceful shutdown timeout                                          | `10s`                                |

`ARQEN_CONFIG_FILE` selects the configuration file for generated applications
and for `arqen check`. The explicit `--file` CLI flag remains the preferred
way to select a file from the Arqen CLI.

`RUST_LOG` overrides `ARQEN_LOG_LEVEL` when present. Local development uses
compact logs by default; choose `pretty` for diagnosis. Production should use
JSON logs and a conservative application filter such as
`service=info,arqen=info`; change the environment and restart the service to
apply a new filter.

## Configuration file

Arqen supports an optional `arqen.toml` configuration file. Use the
`--file` flag to specify a custom path (default: `arqen.toml` in the
current directory).

### Config file discovery

1. If `--file <path>` is passed, load from that path.
2. Otherwise, look for `arqen.toml` in the current working directory.
3. If the file does not exist, proceed with defaults and env vars.
4. If the file exists but cannot be parsed, exit with a configuration error.

### Example arqen.toml

```toml
[server]
host = "127.0.0.1"
port = 8888
compression_enabled = true
compression_threshold = 1024

[logging]
level = "info"
format = "pretty"  # or "json"

[storage]
mode = "memory"
# persistent_path = "/var/lib/my-app/data"  # required for native/persistent
# http_url = "http://localhost:8080"        # required for http mode
# auth_token = "server-side-secret"         # prefer ARQEN_THINGD_AUTH_TOKEN
# encryption_key = ""                        # prefer ARQEN_THINGD_ENCRYPTION_KEY
# schema_path = "schema.thingd"
# cache_enabled = false
# cache_collections = ["catalog_titles", "catalog_genres"]

[sync]
enabled = false
# mode = "http" # disabled, http, or native; native requires storage.mode = "native"
# source_id = "local-instance"
# target_url = "https://thingd-replica.internal"
# collections = ["watchloom_titles"]
# replicate_all = false
# snapshot_fallback = true
```

## Storage modes

### Memory mode (default)

- In-memory thingd engine
- Process-local and disposable
- No external dependencies
- Suitable for development, testing, and prototypes

### HTTP mode

- Connects to a thingd service via HTTP
- Requires `ARQEN_THINGD_URL` or `storage.http_url` configuration
- Suitable for production deployments
- The optional cache is catalog-only and requires an explicit collection
  allowlist. It must not include user- or tenant-scoped collections.

### Native mode

- Embedded persistent thingd with no separate HTTP service
- Requires `persistent_path`
- `persistent` is retained as a compatibility alias for `native`
- The path must be writable and backed up by the deployment owner
- Production validation requires `ARQEN_THINGD_SCHEMA_PATH` for native mode;
  HTTP mode skips this requirement because the remote thingd service owns the
  schema.

### Cloud mode

`cloud` is reserved for a future versioned public thingd.cloud adapter. It
fails explicitly today; Arqen never silently falls back to memory.

## Precedence chain

Configuration is loaded in order of precedence (highest wins):

1. **CLI flags** (`--host`, `--port`, `--log`, `--storage` on `dev`/`start`;
   `--file` on `dev`, `start`, and `up`)
2. **Environment variables** (`ARQEN_*`)
3. **Config file** (`arqen.toml`)
4. **Defaults**

A value set at a higher layer overrides the same value at a lower layer.
For example, `ARQEN_PORT=9000` overrides `port = 8888` in `arqen.toml`,
which overrides the compiled default of `8888`.

## Startup banner

When an Arqen application starts, it prints a banner with essential information:

```text
Arqen v<current-version>
API:    http://127.0.0.1:8888
Health: http://127.0.0.1:8888/health
Docs:   http://127.0.0.1:8888/docs
Agent:  http://127.0.0.1:8888/agent
Storage: memory
```

The banner includes:

- Application version
- Bound API URL
- Health endpoint URL
- Docs endpoint URL
- Agent endpoint URL
- Storage mode (memory, native, persistent, or http)

Development mode (`arqen dev`) uses compact logging by default. Production
mode (`arqen start`) uses JSON logging. Use `arqen dev --watch` for optional
automatic reload. Call `AppConfig::validate_production()` from a
production bootstrap to reject memory storage, disabled authentication, and
pretty logs.
