# Production runbook

Use this checklist when promoting an Arqen application beyond local
development. Arqen provides runtime primitives; the application owner remains
responsible for backups, identity configuration, dependency contracts, and
incident response.

## Preflight

```bash
cargo fmt --all -- --check
cargo check --workspace --all-features
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
arqen check
arqen doctor
```

Review the generated OpenAPI document, agent manifest, health checks, and
storage adapter contract before deployment.

## Required configuration

Production should use structured logs, explicit authentication, and a
non-memory storage mode. Applications should call
`AppConfig::validate_production()` during startup.

```bash
export ARQEN_HOST=0.0.0.0
export ARQEN_PORT=8888
export ARQEN_STORAGE_MODE=http
export ARQEN_THINGD_URL=https://thingd.internal.example
export ARQEN_THINGD_AUTH_TOKEN='loaded-from-secret-manager'
export ARQEN_LOG_FORMAT=json
export ARQEN_LOG_LEVEL=info
export ARQEN_JWT_SECRET='loaded-from-secret-manager'
```

For an embedded durable instance, use `native` and set
`ARQEN_PERSISTENT_PATH`. Never use memory mode for data that must survive a
restart.

## Start and verify

```bash
arqen start
curl --fail-with-body -i http://127.0.0.1:8888/health
curl --fail-with-body -i http://127.0.0.1:8888/ready
curl --fail-with-body -s http://127.0.0.1:8888/agent/manifest | jq .
curl -sI http://127.0.0.1:8888/health | grep -Ei '^(server|x-powered-by):'
```

`/health` indicates process liveness. `/ready` indicates required dependency
readiness. Do not route traffic solely from a successful process start.

Arqen-managed responses identify the framework with `Server: Arqen` and
`X-Powered-By: Arqen`. These headers are intentional; a reverse proxy may
remove or replace them.

## Logging rules

Every request and job should retain a correlation/request ID. Include route,
status, duration, storage mode, backend latency, tenant/instance, subject,
job ID, queue, worker ID, and attempt where applicable.

Never log authorization headers, bearer tokens, API keys, JWTs, passwords,
secret configuration values, raw request bodies, or unrestricted provider
payloads. Log stable resource identifiers and safe error categories instead.
Send JSON logs to the collector, retain error and slow-request events, and
sample only high-volume successful events.

## Validation rules

- Validate configuration before binding the server.
- Validate request bodies with `Validated<T>`.
- Enforce body, pagination, collection, and batch limits.
- Return stable field paths and codes without echoing secrets.
- Test malformed JSON, missing fields, boundary values, nested errors, and
  cross-field failures.
- Test tenant and subject isolation at repository and job boundaries.

## Incident checks

```bash
# Inspect the last deployment's local configuration shape without printing secrets
arqen --json check

# Check dependency and tool availability
arqen doctor

# Inspect service behavior
curl -i http://127.0.0.1:8888/health
curl -i http://127.0.0.1:8888/ready
```

Check readiness failures, 5xx rate, p95/p99 latency, storage errors, queue
lag, retry growth, and dead-letter growth. Preserve correlation IDs when
opening an incident.

## Shutdown and rollback

Use the configured graceful shutdown timeout and allow workers to finish or
release leases. For native storage, stop the process before copying or
restoring the data directory. For HTTP/cloud storage, follow the thingd
backup, restore, and compatibility procedure; Arqen does not own cloud sync
or replication semantics.
