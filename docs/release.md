# Release

Releases are not yet a promise of framework stability. Until native durable
thingd migration, public HTTP parity, and CLI/template work are complete,
describe Arqen as early-stage in release notes and announcements.

Before a release candidate:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cd docs && pnpm install --frozen-lockfile && pnpm build
```

Review [feature status](feature-status.md), update `CHANGELOG.md`, and include
known blockers. Do not turn a planned adapter, cloud mode, or Node.js package
into a completed claim without acceptance evidence.

## Automated release flow

Arqen follows thingd’s conventional-commit release approach:

```text
feature/* → main → automated release/vX.Y.Z PR → main → publish → GitHub Release
```

On pushes to `main`, the `Release` workflow analyzes conventional commits. A
`fix:` or `perf:` commit creates a patch release, `feat:` creates a minor
release, and `!` or `BREAKING CHANGE:` creates a major release. The workflow
opens a `release/vX.Y.Z` pull request that updates `Cargo.toml`, `Cargo.lock`,
and `CHANGELOG.md`.

Publishing starts only after that release PR is merged. The workflow publishes
the single `arqen` crate, creates the `arqen-vX.Y.Z` tag, and creates a matching
GitHub Release. Existing crate versions and GitHub Releases are skipped.

Configure the `CARGO_REGISTRY_TOKEN` repository secret before publishing. A
manual workflow dispatch may include `publish_version` to retry an existing
release after a transient crates.io failure.

The public entry point is the single `arqen` crate. `arqen-cli` remains a
workspace binary and is not published to crates.io.

The public entry point is the single `arqen` crate. `arqen-cli` remains a
workspace binary and is not published to crates.io.
