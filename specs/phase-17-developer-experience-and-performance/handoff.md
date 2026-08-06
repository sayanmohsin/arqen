# Handoff

## Implementation record

- Status: completed
- Implementer: opencode
- Start commit: pending (pre-phase-17 state)
- Completion commit: pending

## CLI surface

Commands: `new`, `generate` (module/tool/job), `dev`, `start`, `up`, `check`, `doctor`
Global flags: `--version`, `--verbose`, `--quiet`, `--color <auto|always|never>`, `--json`, `--file <PATH>`
Exit codes: 0 success, 2 usage, 3 configuration, 4 dependency, 5 runtime, 130 interrupted

## Changed public interfaces

- Added `AppConfig::load_with_file(cli, path)` for config-file discovery
- Added `cli` module (feature-gated) with exit codes, output helpers, and command implementations
- No breaking changes to existing public API

## Benchmark environment

- OS: macOS (Apple Silicon)
- Rust: stable (record rustc version at handoff)
- Feature flags: all features enabled
- Storage mode: memory

## Benchmark results

Run `cargo bench --bench framework` and record p50/p95/p99 for each workload.
Reports are in `target/criterion/`.

## Documentation pages added

- `docs/troubleshooting.md` - common issues and solutions
- `docs/migration.md` - upgrade notes
- `docs/standards.md` - coding standards
- `docs/examples.md` - examples guide
- `docs/health.md` - health and readiness guide
- `docs/performance.md` - benchmark methodology and results

## Documentation pages updated

- `docs/commands.md` - full CLI reference with exit codes
- `docs/configuration.md` - fixed [storage] table, added discovery rules
- `docs/getting-started.md` - expanded 10-minute quickstart
- `docs/deployment.md` - expanded deployment guide
- `docs/repository-structure.md` - expanded structure docs
- `docs/agent-guide.md` - expanded agent integration guide
- `docs/index.md` - updated roadmap link
- `README.md` - added coding agent section
- `CONTRIBUTING.md` - added coding standards and quality checklist

## Known limitations

- Native durable and HTTP adapter benchmarks are not included in this phase
- Allocation counts are not measured
- `arqen dev` does not include an integrated file watcher
- The CLI does not prompt for interactive input
- Generated projects do not include a `src/routes/` directory (use built-in routes)

## Follow-up phases

- Add native durable and HTTP adapter benchmarks
- Add allocation counting to benchmarks
- Add file watcher to `arqen dev`
- Add interactive prompts for project scaffolding
