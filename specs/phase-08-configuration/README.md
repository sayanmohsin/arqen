# Phase 08: Configuration & App State

Objective: provide typed, validated configuration with env/file loading and a composable AppState that wires all dependencies explicitly.

Dependencies: 01, 02, 03.

In scope: AppConfig struct, env/file loading, validation, secret redaction, AppState builder, storage adapter selection, and feature flag awareness.

Out of scope: auth providers, request validation, OpenAPI, observability.

Deliverables: `config.rs` module, `AppState` builder, config tests, migration from hardcoded values.

Acceptance: all config loads from env/files with defaults; secrets never appear in logs or error messages; AppState composes all adapters; config validation produces clear error messages.

Tests: config loading, validation, secret redaction, AppState construction, feature flag combinations.

Docs: update configuration guide and examples.

Handoff: record config schema, env vars, AppState API, and migration guide.
