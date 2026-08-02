# Configuration

Arqen applications are configured through environment variables and optional configuration files.

## Environment variables

| Variable | Description | Default |
|---|---|---|
| `ARQEN_HOST` | Bind address for the HTTP server | `127.0.0.1` |
| `ARQEN_PORT` | Port for the HTTP server | `3000` |
| `ARQEN_LOG` | Log level or output format (`debug`, `info`, `warn`, `error`, `json`) | `info` |
| `ARQEN_STORAGE_MODE` | thingd storage mode (`memory`, `http`) | `memory` |

## Configuration file

Arqen supports an optional `arqen.toml` configuration file in the project root. Environment variables take precedence over file configuration.

Example `arqen.toml`:

```toml
[server]
host = "127.0.0.1"
port = 3000

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
║  Arqen v0.1.0                               ║
║  API:        http://127.0.0.1:3000           ║
║  Health:     http://127.0.0.1:3000/health    ║
║  Docs:       http://127.0.0.1:3000/docs      ║
║  Agent:      http://127.0.0.1:3000/agent     ║
║  Storage:    memory                          ║
║  Workers:    enabled                         ║
║  Hot reload: enabled (dev mode)              ║
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
- Hot reload state (enabled/disabled, dev mode only)

## Precedence order

1. Command-line flags (e.g., `--host`, `--port`)
2. Environment variables (e.g., `ARQEN_HOST`)
3. Configuration file (`arqen.toml`)
4. Defaults