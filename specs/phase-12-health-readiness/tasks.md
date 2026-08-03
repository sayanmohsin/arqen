# Tasks

- [ ] Define `HealthCheck` trait for dependency checks
- [ ] Create `HealthRegistry` for registering dependencies
- [ ] Implement `HealthStatus` enum (Healthy, Degraded, Unhealthy)
- [ ] Add configurable timeouts for dependency checks
- [ ] Implement `/health` endpoint (liveness only)
- [ ] Implement `/ready` endpoint (readiness with dependency checks)
- [ ] Add dependency check implementations (storage, memory, disk)
- [ ] Add health tests (registry, checks, timeouts, degraded states)
- [ ] Update examples to use health registry
