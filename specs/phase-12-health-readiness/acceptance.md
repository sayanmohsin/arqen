# Acceptance Criteria

1. `/health` returns 200 if process is running (liveness only)
2. `/ready` returns 200 when all dependencies are healthy
3. `/ready` returns 503 when any dependency is unhealthy
4. Dependency checks have configurable timeouts
5. Dependencies report healthy, degraded, or unhealthy states
6. Degraded state doesn't fail readiness
7. Tests verify all health scenarios
