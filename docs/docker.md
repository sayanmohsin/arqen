# Docker

Generated applications should include a multi-stage `Dockerfile`, `.dockerignore`, `docker-compose.yml`, and `.env.example`.

The image should compile a release Rust binary in a builder stage and run only the application binary in the runtime stage. The deployment documentation must explain the application port, `/health`, `/ready`, graceful shutdown, storage configuration, worker processes, and secret injection.

Local Compose should optionally run Arqen with a thingd sidecar so the same application can move from memory mode to HTTP-backed durable mode.
# Build context

Arqen's embedded native thingd dependency is a sibling crate during workspace
development. Build from the `/ancatag` parent directory:

```bash
docker build -f arqen/Dockerfile -t arqen:local .
docker compose -f arqen/docker-compose.yml up --build
```

The Dockerfile copies only Arqen and the public `thingd` crate into the build
context. It does not depend on private thingd-cloud modules.
