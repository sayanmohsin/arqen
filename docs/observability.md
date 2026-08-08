# Observability

Arqen starts with structured tracing and application-level metrics. Configure
pretty logs for local work and JSON logs for collection systems. Attach a
correlation identifier to requests and include context such as `job_id`,
`worker_id`, queue, status, and duration.

The request metrics report tracks totals, status counts, uptime, error rate,
and duration percentiles including p50, p95, p99, and maximum. Job metrics
track processed, completed, failed, and average duration values.

These are framework primitives, not a hosted observability product. Export
records to your logging and metrics platform, define retention and redaction
rules, and set service-level alerts in the deployment environment. OpenTelemetry
and Prometheus exporters are not bundled in the current release.

At minimum, alert on readiness failures, elevated 5xx/error rate, p95 and p99
latency, storage dependency failures, queue lag, retry growth, and dead-letter
growth. Preserve correlation IDs across HTTP requests, repository calls, and
jobs so one user action can be followed through the system.

For operational checks, combine `/health` for process liveness with `/ready`
for required dependency readiness. See [deployment](deployment.md) and
[security](security.md) for production handling.
