# Phase 11: CLI packaging and release

Objective: make Arqen pleasant to start like Express while keeping generated
projects, hot reload, Docker, checks, and release artifacts real. Outcome:
`new`, `generate`, `dev`, `start`, `check`, and `doctor` work from a clean
checkout.

Dependencies: 02, 05, 08, 09. In scope: inline project scaffolding,
cargo-watch workflow,
startup banner, env loading, Docker/Compose smoke tests, checks, and release
docs. Out of scope: Watchloom UI and private cloud automation.

Acceptance: generated projects compile without unavailable registry assumptions;
dev has a real documented reload loop; check fails on broken projects; Docker
starts with a subcommand and `/health` returns 200; release metadata is validated.

Tests: generated project, CLI help, Docker build/run, health smoke, and package dry run.
Docs: update getting-started, commands, hot-reload, Docker, and deployment docs.
Handoff: record exact commands, outputs, image tags, and release blockers.
