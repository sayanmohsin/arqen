# Phase 16: Module Composition

Objective: provide a module system for composing AuthModule, StorageModule, and AgentModule with explicit state and layers.

Dependencies: 08, 09, 10, 11, 12, 13, 14, 15.

In scope: module trait, module builders, state composition, layer composition, and module registration.

Out of scope: dependency injection container, automatic wiring.

Deliverables: `module.rs` module, module trait, module builders, and tests.

Acceptance: modules compose explicit state and layers; modules are registered via builder; modules provide routes and middleware; modules are testable in isolation.

Tests: module composition, state wiring, layer composition, module registration.

Docs: update module composition guide.

Handoff: record module API, builders, and composition patterns.
