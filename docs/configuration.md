# Configuration

Arqen applications are configured through environment variables and optional configuration files.

## Environment variables

| Variable                     | Description                                          | Default                          |
| ---------------------------- | ---------------------------------------------------- | -------------------------------- |
| `ARQEN_HOST`                 | Bind address for the HTTP server                     | `127.0.0.1`                      |
| `ARQEN_PORT`                 | Port for the HTTP server                             | `8888`                           |
| `ARQEN_STORAGE_MODE`         | thingd storage mode (`memory`, `persistent`, `http`) | `memory`                         |
| `ARQEN_PERSISTENT_PATH`      | Native durable thingd storage path                   | unset; required for `persistent` |
| `ARQEN_THINGD_URL`           | thingd HTTP service URL                              | unset; required for `http`       |
| `ARQEN_JWT_SECRET`           | JWT secret, kept redacted in configuration output    | unset                            |
| `ARQEN_API_KEY_HEADER`       | API-key request header                               | `X-API-Key`                      |
| `ARQEN_LOG_LEVEL`            | Log level                                            | `info`                           |
| `ARQEN_LOG_FORMAT`           | Log format (`pretty`, `json`, `compact`)             | `pretty`                         |
| `ARQEN_WORKER_ENABLED`       | Enable workers                                       | implementation default           |
| `ARQEN_WORKER_QUEUES`        | Comma-separated worker queues                        | implementation default           |
| `ARQEN_WORKER_POLL_INTERVAL` | Worker polling interval                              | implementation default           |
| `ARQEN_WORKER_LEASE_SECONDS` | Job lease duration                                   | implementation default           |
| `ARQEN_WORKER_MAX_RETRIES`   | Maximum job retries                                  | implementation default           |
| `ARQEN_WORKER_CONCURRENCY`   | Worker concurrency                                   | implementation default           |
| `ARQEN_HEALTH_CHECK_TIMEOUT` | Dependency health-check timeout                      | implementation default           |
| `ARQEN_HEALTH_STARTUP_DELAY` | Startup delay before health checks                   | implementation default           |
| `ARQEN_REQUEST_TIMEOUT`      | HTTP request timeout                                 | `30s`                            |
| `ARQEN_MAX_BODY_SIZE`        | Maximum request body size                            | `1048576`                        |
| `ARQEN_SHUTDOWN_TIMEOUT`     | Graceful shutdown timeout                            | `10s`                            |

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

[logging]
level = "info"
format = "pretty"  # or "json"

[storage]
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
- Requires `ARQEN_THINGD_URL` or `storage.http_url` configuration
- Suitable for production deployments

## Precedence chain

Configuration is loaded in order of precedence (highest wins):

1. **CLI flags** (`--host`, `--port`, `--log`, `--storage`, `--file`)
2. **Environment variables** (`ARQEN_*`)
3. **Config file** (`arqen.toml`)
4. **Defaults**

A value set at a higher layer overrides the same value at a lower layer.
For example, `ARQEN_PORT=9000` overrides `port = 8888` in `arqen.toml`,
which overrides the compiled default of `8888`.

## Startup banner

When an Arqen application starts, it prints a banner with essential information:

```text
Arqen v0.4.0
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
- Storage mode (memory or http)

Development mode (`arqen dev`) uses pretty logging. Production mode
(`arqen start`) uses JSON logging. `arqen dev` does not include an
integrated file watcher.
