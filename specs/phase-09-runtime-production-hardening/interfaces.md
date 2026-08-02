# Interfaces

- `GET /health`: `{ "status": "ok" }`.
- `GET /ready`: 200 only when dependencies are ready, otherwise 503.
- `X-Request-Id`: accept a safe value or generate and echo one.
- Config validates host, port, storage mode, log format, and required secrets.
