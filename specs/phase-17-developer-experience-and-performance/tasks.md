# Tasks

## 1. Establish the baseline

- [ ] Record current CLI commands, flags, generated files, exit behavior, and
  known limitations.
- [ ] Inventory public docs, broken/stale claims, examples, and API links.
- [ ] Capture baseline build, test, package, and benchmark results.

## 2. Harden the CLI

- [ ] Introduce shared command-output and error helpers.
- [ ] Normalize help text, version output, colors, verbosity, JSON output, and
  exit codes.
- [ ] Implement and test configuration discovery and `--file` consistently.
- [ ] Add command-level integration tests for success, invalid input, existing
  files, signals, and missing dependencies.
- [ ] Update generated project README and all CLI docs from actual output.

## 3. Improve documentation

- [ ] Rewrite the start path for a new user and a new coding agent.
- [ ] Add troubleshooting and migration pages for one-crate releases.
- [ ] Add runnable examples for modules, tools, jobs, auth, validation,
  health, OpenAPI, testing, and thingd modes.
- [ ] Link docs to source/tests without private AI files.
- [ ] Add a docs drift checklist to the release process.

## 4. Add performance evidence

- [ ] Add a benchmark crate or package under the Arqen repository without
  making it a published library package.
- [ ] Benchmark the workloads defined in `interfaces.md`.
- [ ] Add a repeatable benchmark command and artifact/report format.
- [ ] Investigate regressions before changing hot paths; add focused tests for
  every optimization.

## 5. Standardize quality tooling

- [ ] Pin Prettier and add `format`, `format:check`, `lint`, and `build` docs
  scripts.
- [ ] Add CI checks for Rust, docs formatting, Markdownlint, links, package
  dry-run, examples, and benchmark compilation.
- [ ] Add editor-independent instructions to `CONTRIBUTING.md`.
- [ ] Ensure generated files and local dependencies are ignored.

## 6. Define coding standards

- [ ] Document public API naming, feature gating, errors, logging, secrets,
  tests, compatibility, and changelog rules.
- [ ] Add review checklists for API changes, performance changes, and docs.
- [ ] Keep the feature-status matrix and phase status evidence current.

## 7. Validate and hand off

- [ ] Run every command in `test-plan.md` from a clean checkout.
- [ ] Verify no Watchloom, thingd, or thingd-cloud source changed.
- [ ] Verify no private AI documents are tracked.
- [ ] Record benchmark results, known limits, and follow-up issues in
  `handoff.md`.
