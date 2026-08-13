# Release

Releases are not yet a promise of framework stability. Arqen is currently
early-stage: the core framework and CLI are implemented, while durability,
public HTTP compatibility, security review, and application-level operational
validation remain deployment gates.

Before a release candidate:

```bash
cargo fmt --all -- --check
cargo check -p arqen --all-features
cargo test -p arqen --all-features
cargo clippy -p arqen --all-targets --all-features -- -D warnings
cargo bench --bench framework -- --noplot
cargo doc -p arqen --all-features --no-deps
cargo package -p arqen --allow-dirty
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

Release Please owns the Arqen package version. Feature, fix, performance, and
documentation pull requests must not manually change
`crates/arqen/Cargo.toml` or `.release-please-manifest.json`; CI rejects those
changes outside a generated release pull request. This prevents a manually
prepared version from being incremented a second time when the release PR is
created. The Rust workflow also runs a default-feature Clippy gate with
warnings denied, matching the feature set used by the published crate.

Publishing starts only after that release PR is merged. The workflow publishes
the single `arqen` crate, creates the `arqen-vX.Y.Z` tag, and creates a matching
GitHub Release. Existing crate versions and GitHub Releases are skipped.

Configure the `CARGO_REGISTRY_TOKEN` repository secret before publishing. A
manual workflow dispatch may include `publish_version` to retry an existing
release after a transient crates.io failure.

The public entry point is the single `arqen` crate. Its CLI binary is enabled
with the `cli` feature and is published as part of the same package; there is
no separate `arqen-cli` package.
