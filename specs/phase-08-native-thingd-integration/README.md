# Phase 08: native thingd integration

Objective: make the public `thingd` Rust crate the source of truth for embedded
Arqen storage. Outcome: memory and persistent modes share native thingd semantics.

Dependencies: 01 and 03. Sidecar HTTP remains Phase 10.

In scope: native engine handle, object/event/queue/link/search mappings,
engine selection, locking policy, and parity tests. Out of scope: cloud
internals, Watchloom models, and changes inside thingd.

Deliverables: native memory/persistent wiring, compatibility boundary, persistence
tests, and migration notes for the legacy copied backend.

Acceptance: no production path uses copied storage semantics; both native
engines pass the same contract suite; unsupported operations fail explicitly.

Tests: unit conversions, memory/native integration, reopen persistence,
concurrency, and legacy contract regression.

Docs: update `docs/thingd-integration.md` and `docs/adapter-contract.md`.

Handoff: record dependency version, public APIs, tests, limitations, and Phase
10 impact in `handoff.md`.
