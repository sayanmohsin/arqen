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

## crates.io publishing

The repository includes a gated `Release crates` workflow. It runs on tags in
the form `arqen-v0.1.0` or by manual dispatch, verifies formatting and
publishable packages, then publishes the workspace crates in dependency order.
Already-published versions are skipped.

Configure the `CARGO_REGISTRY_TOKEN` secret in the `crates-io` GitHub
environment before publishing. The tag must match the workspace version, and
publishing remains intentionally separate from ordinary pushes to `main`.
