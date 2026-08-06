# Phase 17: Developer experience, performance, and agent onboarding

## Objective

Make Arqen easier to adopt, faster to operate, easier to maintain, and
straightforward for a new human or coding agent to understand from a clean
checkout.

## Public outcome

Arqen will have a stable, discoverable CLI; a coherent documentation system;
measured performance budgets; one reproducible formatting and linting contract;
and a public onboarding path that does not depend on private AI instruction
files.

## Status

Ready for implementation after Phases 08–16. This phase is planning only until
an agent begins implementation.

## Non-goals

- No NestJS-style hidden dependency injection container.
- No coupling to Watchloom domain types.
- No changes to thingd or thingd-cloud source.
- No claim that benchmarks alone make the framework production-ready.
- No tracked `AGENTS.md`, `.opencode`, or other private AI working documents.

## Workstreams

| Workstream | Result |
|---|---|
| CLI | Stable command taxonomy, help, errors, exit codes, config discovery, and smoke tests |
| Documentation | Task-oriented guides, API reference links, examples, versioned status, and search/navigation quality |
| Performance | Reproducible benchmarks, allocation/latency budgets, hot-path review, and regression gates |
| Code quality | Rustfmt, Clippy, Markdownlint, Prettier, dependency audit, and consistent CI commands |
| Standards | Public API, error, logging, security, testing, and compatibility conventions |
| Onboarding | A ten-minute clean-checkout path for people and agents using public files only |

The implementation must remain one published Cargo package: `arqen`.
