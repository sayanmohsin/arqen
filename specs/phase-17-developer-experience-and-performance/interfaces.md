# Interfaces and contracts

## CLI contract

The CLI is a binary in the published `arqen` package behind the `cli` feature.
Each command must expose:

- human-readable output by default;
- `--json` output where the result is consumed by automation;
- stable non-zero exit codes documented in `docs/commands.md`;
- errors that identify the failed input and a useful next action;
- no secrets in output, logs, or generated files.

The CLI must not silently rewrite existing files. Generators report conflicts
and exit non-zero.

## Configuration contract

Precedence remains CLI overrides → environment → `arqen.toml` → defaults.
Discovery and invalid configuration errors must be documented and covered by
tests. The library API remains usable without the CLI feature.

## Documentation contract

Every public feature has:

1. a short conceptual explanation;
2. a runnable or clearly labeled example;
3. status and limitation language;
4. links to the relevant Rust API and tests;
5. an explicit security and operational note when applicable.

The root README must work on GitHub and crates.io. Site-only links may not be
the sole path to a feature guide.

## Performance contract

Benchmarks are comparative evidence, not universal guarantees. Each benchmark
records workload, environment, feature flags, storage mode, and commit. A
regression gate must use an agreed tolerance and report rather than silently
fail on unavailable hardware.

Initial budgets are targets for the benchmark harness, not release claims:

| Workload | Target |
|---|---|
| In-memory health route | p95 under 1 ms in the benchmark environment |
| In-memory manifest generation | p95 under 2 ms for 100 tools |
| In-memory object CRUD | p95 under 2 ms for the baseline fixture |
| Job enqueue/dequeue | p95 under 2 ms for the baseline fixture |

Native durable and HTTP adapter budgets must be measured separately.

## Formatting contract

- Rust: `cargo fmt --all -- --check`.
- Rust lint: `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Docs/site: pinned Prettier plus Markdownlint.
- Links: Lychee against versioned files in a clean checkout.
- Generated output: CLI smoke tests and `cargo check`.

Private AI files are not part of the onboarding or quality contract.
