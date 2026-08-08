# Logging and observability

Use `tracing` and `tracing-subscriber` throughout the generated application.

Development logs should be readable. Production logs should be structured JSON. Every request and job should carry a request or correlation ID.

Minimum fields for every request or job:

- timestamp;
- level and target;
- request ID;
- route or job name;
- status or outcome;
- duration;
- error category when applicable.
- tenant/instance ID when applicable;
- authenticated subject when applicable;
- storage mode and backend latency for storage operations;
- job ID, queue, worker ID, and attempt for jobs.

Secrets, bearer tokens, provider credentials, and raw request bodies must never be logged by default.

Built-in middleware covers request logging, correlation IDs, request timeouts,
body limits, permissive development CORS, and health/readiness endpoints. The
default CORS policy should be replaced with an explicit origin policy before a
production deployment.

## Recommended techniques

Use spans around application work and fields instead of interpolated log
strings:

```rust
use tracing::{info_span, Instrument};

async fn load_profile(user_id: &str) -> Result<Profile, AppError> {
    async move {
        tracing::info!(operation = "profile.load", "loading profile");
        // repository call
        # todo!()
    }
    .instrument(info_span!("profile", subject = %user_id))
    .await
}
```

Log lifecycle events at `info`, expected validation/auth failures at `warn`,
and unexpected failures at `error`. Do not log full request bodies, tokens,
passwords, API keys, provider responses, or arbitrary user-controlled JSON.
Use stable identifiers and hashes when an event needs correlation without
revealing payload data. Keep high-volume successful request logs at `info` or
sample them in the deployment collector; always retain errors and slow
requests.

`EnvFilter` takes precedence over the configured level when the standard
`RUST_LOG` variable is set. This is a tracing ecosystem variable, not an
Axum-specific variable:

```bash
RUST_LOG=arqen=debug,my_app=info arqen dev
ARQEN_LOG_FORMAT=json arqen start
```

**Current status:** Request logging includes method, URI, status, duration,
request ID, subject, tenant ID, and instance ID when a `RequestContext` is
present. Structured JSON logging is available through the logging
configuration. Arqen provides in-process request metrics and percentiles; it
does not currently ship an OpenTelemetry exporter or Prometheus endpoint.
