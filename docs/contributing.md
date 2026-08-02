# Contributing

Arqen is early-stage. Contributions should make the public contract clearer,
keep adapter behavior aligned, or add evidence for a feature.

Before opening a change:

1. Read `AGENTS.md`, `specs/README.md`, and the relevant phase specification.
2. Keep changes inside Arqen; do not modify Watchloom, thingd, or thingd-cloud.
3. Add or update docs and contract tests before introducing new implementation.
4. Run `cargo fmt --all -- --check`, `cargo check --workspace`, and `cargo test --workspace`.
5. For docs, run `pnpm install --frozen-lockfile` and `pnpm build` in `docs/`.

Use focused commits and describe known limitations honestly. See the root
[contribution guide](https://github.com/sayanmohsin/arqen/blob/main/CONTRIBUTING.md)
for the pull request checklist.
