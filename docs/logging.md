# Logging and observability

Use `tracing` and `tracing-subscriber` throughout the generated application.

Development logs should be readable. Production logs should be structured JSON. Every request and job should carry a request or correlation ID.

Minimum fields:

- timestamp;
- level and target;
- request ID;
- route or job name;
- status or outcome;
- duration;
- error category when applicable.

Secrets, bearer tokens, provider credentials, and raw request bodies must never be logged by default.

Built-in middleware covers request logging, correlation IDs, request timeouts,
body limits, permissive development CORS, and health/readiness endpoints. The
default CORS policy should be replaced with an explicit origin policy before a
production deployment.

**Current status:** Request logging logs method, URI, status, duration, and
correlation ID. Structured JSON logging is available through the logging
configuration. Arqen provides in-process request metrics and percentiles; it
does not currently ship an OpenTelemetry exporter or Prometheus endpoint.
