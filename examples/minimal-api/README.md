# Minimal API Example

A minimal Arqen application demonstrating basic HTTP server setup.

## Getting started

```bash
cargo run
```

The server will start on http://127.0.0.1:3000

## Endpoints

- GET /health - Liveness check
- GET /ready - Readiness check
- GET /agent - Agent description
- GET /agent/manifest - Agent manifest
- GET /docs - API documentation

## Development

```bash
cargo run
```

The server will start with development logging (pretty format).