# Scope

Health endpoint is liveness-only (always 200 if process is running).
Readiness endpoint checks registered dependencies.
Dependency checks have configurable timeouts.
Dependencies can be in healthy, degraded, or unhealthy states.
Readiness returns 503 when any dependency is unhealthy.
Degraded state is reported but doesn't fail readiness.
