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
