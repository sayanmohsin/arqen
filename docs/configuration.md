# Configuration

Arqen applications are configured through environment variables and optional configuration files.

## Environment variables

| Variable | Description | Default |
|---|---|---|
| `ARQEN_HOST` | Bind address for the HTTP server | `127.0.0.1` |
| `ARQEN_PORT` | Port for the HTTP server | `8888` |
| `ARQEN_STORAGE_MODE` | thingd storage mode (`memory`, `persistent`, `http`) | `memory` |
| `ARQEN_PERSISTENT_PATH` | Native durable thingd storage path | unset; required for `persistent` |
| `ARQEN_THINGD_URL` | thingd HTTP service URL | unset; required for `http` |
| `ARQEN_JWT_SECRET` | JWT secret, kept redacted in configuration output | unset |
| `ARQEN_API_KEY_HEADER` | API-key request header | `X-API-Key` |
| `ARQEN_LOG_LEVEL` | Log level | `info` |
| `ARQEN_LOG_FORMAT` | Log format (`pretty`, `json`, `compact`) | `pretty` |
| `ARQEN_WORKER_ENABLED` | Enable workers | implementation default |
| `ARQEN_WORKER_QUEUES` | Comma-separated worker queues | implementation default |
| `ARQEN_WORKER_POLL_INTERVAL` | Worker polling interval | implementation default |
| `ARQEN_WORKER_LEASE_SECONDS` | Job lease duration | implementation default |
| `ARQEN_WORKER_MAX_RETRIES` | Maximum job retries | implementation default |
| `ARQEN_WORKER_CONCURRENCY` | Worker concurrency | implementation default |
| `ARQEN_HEALTH_CHECK_TIMEOUT` | Dependency health-check timeout | implementation default |
| `ARQEN_HEALTH_STARTUP_DELAY` | Startup delay before health checks | implementation default |
| `ARQEN_REQUEST_TIMEOUT` | HTTP request timeout | `30s` |
| `ARQEN_MAX_BODY_SIZE` | Maximum request body size | `1048576` |
| `ARQEN_SHUTDOWN_TIMEOUT` | Graceful shutdown timeout | `10s` |

## Configuration file

Arqen supports an optional `arqen.toml` configuration file in the project root. Environment variables take precedence over file configuration.

Example `arqen.toml`:

```toml
[server]
host = "127.0.0.1"
port = 8888

[logging]
level = "info"
format = "pretty"  # or "json"

[thingd]
mode = "memory"
# url = "http://localhost:8080"  # required for http mode
```

## Storage modes

### Memory mode (default)

- In-memory thingd engine
- Process-local and disposable
- No external dependencies
- Suitable for development, testing, and prototypes

### HTTP mode

- Connects to a thingd service via HTTP
- Requires `ARQEN_THINGD_URL` or `thingd.url` configuration
- Suitable for production deployments

## Startup banner

When an Arqen application starts, it prints a banner with essential information:

```text
╔══════════════════════════════════════════════╗
║  Arqen v0.3.0                               ║
║  API:        http://127.0.0.1:8888           ║
║  Health:     http://127.0.0.1:8888/health    ║
║  Docs:       http://127.0.0.1:8888/docs      ║
║  Agent:      http://127.0.0.1:8888/agent     ║
║  Storage:    memory                          ║
║  Workers:    enabled                         ║
║  Hot reload: external cargo-watch (dev only) ║
╚══════════════════════════════════════════════╝
```

The banner includes:

- Application name and version
- Bound API URL
- Health endpoint URL
- Docs endpoint URL
- Agent endpoint URL
- Storage mode (memory or http)
- Worker state (enabled/disabled)
- Development watcher note; `arqen dev` does not include an integrated watcher.

## Precedence order

1. Command-line flags (e.g., `--host`, `--port`)
2. Environment variables (e.g., `ARQEN_HOST`)
3. Configuration file (`arqen.toml`)
4. Defaults
