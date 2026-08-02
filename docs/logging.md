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

Built-in middleware covers request logging. Timeouts, body limits, panic capture, CORS configuration, and health endpoints are planned.

**Current status:** Basic request logging middleware logs method, URI, status, and duration. Structured JSON logging is available via `--log json`.
