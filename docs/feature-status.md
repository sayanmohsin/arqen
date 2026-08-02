# Feature status

This table is deliberately conservative. “Documented” means the contract or
design exists; it does not mean every production path is complete.

| Area | Status | Notes |
|---|---|---|
| Rust workspace and Axum HTTP server | Available | Working crates and health, readiness, docs, and agent routes exist. |
| In-memory storage mode | Available | Intended for local development and tests. |
| Typed tools and manifests | Contract / partial | Types and examples exist; broader discovery parity is still evolving. |
| Durable jobs and workers | Contract / partial | Queue semantics and worker code exist; production hardening continues. |
| Native durable thingd | In progress | A key maturity gate for the project. |
| HTTP thingd adapter | Contract / partial | Public-contract parity and operational validation remain. |
| Cloud adapter | Future | Depends on a public thingd-cloud customer contract. |
| CLI `new`, `dev`, `start`, `check`, `doctor` | Available / early | Commands exist; some checks and watcher behavior are intentionally incomplete. |
| Node.js support | Future direction | Planned through HTTP, SDKs, templates, and manifests; no Node package is promised yet. |
| GitHub Pages docs | In progress | This site is the public documentation surface for Phase 12. |

Do not infer completion from a roadmap heading. Check the implementation,
tests, and phase acceptance evidence for the feature you need.
