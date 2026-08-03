# Test Plan

## Unit Tests
- HealthCheck trait implementation
- HealthRegistry registration and execution
- HealthStatus enum behavior
- Timeout handling

## Integration Tests
- `/health` returns 200 (liveness)
- `/ready` returns 200 when healthy
- `/ready` returns 503 when unhealthy
- Timeout causes unhealthy state

## Manual Verification
- Health endpoints work with real dependencies
- Degraded state is reported correctly
