# Tasks

## 1. Establish the baseline

- [x] Record current CLI commands, flags, generated files, exit behavior, and
  known limitations.
- [x] Inventory public docs, broken/stale claims, examples, and API links.
- [x] Capture baseline build, test, package, and benchmark results.

## 2. Harden the CLI

- [x] Introduce shared command-output and error helpers.
- [x] Normalize help text, version output, colors, verbosity, JSON output, and
  exit codes.
- [x] Implement and test configuration discovery and `--file` consistently.
- [x] Add command-level integration tests for success, invalid input, existing
  files, signals, and missing dependencies.
- [x] Update generated project README and all CLI docs from actual output.

## 3. Improve documentation

- [x] Rewrite the start path for a new user and a new coding agent.
- [x] Add troubleshooting and migration pages for one-crate releases.
- [x] Add runnable examples for modules, tools, jobs, auth, validation,
  health, OpenAPI, testing, and thingd modes.
- [x] Link docs to source/tests without private AI files.
- [x] Add a docs drift checklist to the release process.

## 4. Add performance evidence

- [x] Add a benchmark crate or package under the Arqen repository without
  making it a published library package.
- [x] Benchmark the workloads defined in `interfaces.md`.
- [x] Add a repeatable benchmark command and artifact/report format.
- [x] Investigate regressions before changing hot paths; add focused tests for
  every optimization.

## 5. Standardize quality tooling

- [x] Pin Prettier and add `format`, `format:check`, `lint`, and `build` docs
  scripts.
- [x] Add CI checks for Rust, docs formatting, Markdownlint, links, package
  dry-run, examples, and benchmark compilation.
- [x] Add editor-independent instructions to `CONTRIBUTING.md`.
- [x] Ensure generated files and local dependencies are ignored.

## 6. Define coding standards

- [x] Document public API naming, feature gating, errors, logging, secrets,
  tests, compatibility, and changelog rules.
- [x] Add review checklists for API changes, performance changes, and docs.
- [x] Keep the feature-status matrix and phase status evidence current.

## 7. Validate and hand off

- [x] Run the documented Rust, CLI, example, docs, formatting, and lint checks.
- [x] Verify no Watchloom, thingd, or thingd-cloud source changed.
- [x] Verify no private AI documents are tracked.
- [x] Record benchmark results, known limits, and follow-up issues in
  `handoff.md`.
