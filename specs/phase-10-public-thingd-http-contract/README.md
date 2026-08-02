# Phase 10: public thingd HTTP contract

Objective: make sidecar mode depend only on thingd's documented public REST
API. Outcome: authenticated, versioned, contract-tested remote storage.

Dependencies: 03 and a pinned public thingd REST contract. In scope: `/v1`
routes, envelopes, bearer auth, tenant propagation, errors, retries, timeouts,
and compatibility tests. Out of scope: private thingd-cloud modules and direct
database access.

Acceptance: objects, events, queues, search, and links match public REST;
credentials stay server-side; non-2xx responses map to typed errors; native and
HTTP modes pass equivalent scenarios.

Tests: local public thingd server/fixture only; include 401, 404, 409, 429,
5xx, timeout, malformed JSON, and unavailable-server cases.

Docs: update `docs/thingd-integration.md` and `docs/adapter-contract.md`.
Handoff: record server version, route matrix, unsupported features, and tests.
