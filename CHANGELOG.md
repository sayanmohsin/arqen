## [0.2.0](https://github.com/sayanmohsin/arqen/releases/tag/arqen-v0.2.0) (2026-08-02)

### Features

- add public arqen facade crate (5ea19ac)

### Bug Fixes

- keep cli workspace dependency version agnostic (198763e)
- render a single brace-style logo (026d2a1)
- correct Pages logo asset path (b01a65f)
- align native adapter with public thingd API (58f80c4)

# Changelog

## Unreleased

### Features

- harden CLI with exit codes, JSON output, config discovery, global flags, and integration tests
- generate compiling projects with `arqen = "0.4"` and `logging` feature
- add Criterion benchmarks for routing, manifest, validation, in-memory thingd, jobs, and health
- add comprehensive documentation guides (troubleshooting, migration, standards, examples, health, performance)
- add Prettier and Markdownlint tooling with CI checks
- expand README with 10-minute quickstart and coding agent section

Arqen remains early-stage; see [`docs/feature-status.md`](docs/feature-status.md)
for the current capability boundary.
