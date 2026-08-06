# Scope

## In scope

### CLI

- Define `arqen new`, `generate`, `dev`, `start`, `up`, `check`, and `doctor`
  as the supported command surface.
- Add consistent `--help`, `--version`, `--verbose`, `--quiet`, `--color`, and
  `--json` behavior where meaningful.
- Return documented exit-code classes for usage, configuration, dependency,
  runtime, and interrupted shutdown errors.
- Add config-file discovery rules, `--file`, environment precedence, and
  machine-readable command output.
- Test generated projects and every command without relying on an interactive
  terminal.

### Documentation

- Establish one information architecture for README, `docs/`, API docs, and
  `specs/`.
- Add a ten-minute quickstart, CLI reference, configuration reference, module
  guide, deployment guide, troubleshooting, migration notes, and examples.
- Keep feature status and maturity language synchronized with implementation.
- Make public agent integration documentation explicit, typed, and contract-
  based; private local AI instructions remain ignored.

### Performance

- Add Criterion benchmarks for routing, manifest generation, validation,
  in-memory thingd CRUD/query, job enqueue/dequeue, and health checks.
- Define baseline hardware, Rust profile, dataset sizes, warm-up, and reporting
  rules.
- Measure p50/p95/p99 latency, throughput, allocations where practical, and
  memory growth for representative workloads.
- Review locks, cloning, JSON conversion, logging, and adapter boundaries on
  measured hot paths before optimizing.

### Tooling and standards

- Keep `cargo fmt`, Clippy with `-D warnings`, tests, docs, audit, and package
  checks reproducible locally and in CI.
- Add Prettier for Markdown, YAML, JSON, and TypeScript/VitePress files with
  a pinned version and `pnpm format:check`.
- Add Markdownlint and link validation with an explicit clean-checkout scope.
- Document naming, module boundaries, public API docs, errors, logging,
  feature flags, test layers, and breaking-change rules.

## Out of scope

OpenTelemetry/Prometheus exporters, new authentication protocols, cloud
provisioning, automatic schema derivation, and language SDK implementation are
separate phases unless a measured or documented dependency requires a narrow
interface change.
