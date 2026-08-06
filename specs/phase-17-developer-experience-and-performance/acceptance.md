# Acceptance criteria

- A new contributor can find purpose, status, architecture, setup, tests,
  coding standards, and contribution rules from the root README in under ten
  minutes.
- A coding agent can implement a scoped change using only tracked README,
  `docs/`, `specs/`, source, and tests; no `AGENTS.md` or private AI file is
  required.
- `arqen --help` and each supported subcommand document valid inputs, output,
  errors, and exit behavior.
- `arqen new` produces a project that uses only `arqen = "0.4"`, has a useful
  README, and passes `cargo check`.
- CLI configuration precedence and conflict behavior are covered by tests.
- README links resolve on GitHub and crates.io; site links resolve after a clean
  Pages deployment.
- All public feature pages state what is implemented, partial, future, or
  application-owned.
- `cargo fmt`, Clippy, tests, docs, Markdownlint, Prettier, links, examples,
  and package dry-run pass in CI.
- Benchmark workloads are repeatable, versioned, and report p50/p95/p99 or
  explicitly explain why a metric is unavailable.
- No performance target is advertised as a universal production guarantee.
- No Watchloom, thingd, or thingd-cloud source files are modified.
- No private AI instruction files are tracked after the phase.
