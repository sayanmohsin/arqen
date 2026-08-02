# Contributing to Arqen

Arqen is early-stage and documentation-first in its public contracts. Read
[`AGENTS.md`](AGENTS.md) and [`specs/README.md`](specs/README.md) before
starting work.

## Before opening a pull request

- Keep changes scoped to Arqen. Do not edit Watchloom, thingd, or thingd-cloud.
- Add documentation and contract tests before implementation features.
- Preserve in-memory and durable adapter parity.
- Run `cargo fmt --all -- --check`, `cargo check --workspace`, and `cargo test --workspace`.
- Run `cd docs && pnpm install --frozen-lockfile && pnpm build` for documentation changes.
- Describe incomplete work, compatibility assumptions, and security impact.

Small, focused pull requests are easier to review. Use the issue templates for
bugs, features, and documentation gaps.
