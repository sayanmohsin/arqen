# Contributing to Arqen

Arqen is early-stage and documentation-first in its public contracts. Read
the [README](README.md), [public documentation](https://sayanmohsin.github.io/arqen/),
and [`specs/README.md`](specs/README.md) before starting work.

## Before opening a pull request

- Keep changes scoped to Arqen. Do not edit Watchloom, thingd, or thingd-cloud.
- Add documentation and contract tests before implementation features.
- Preserve in-memory and durable adapter parity.
- Run `cargo fmt --all -- --check`, `cargo check --workspace`, and `cargo test --workspace`.
- Run `cd docs && pnpm install --frozen-lockfile && pnpm build` for documentation changes.
- Describe incomplete work, compatibility assumptions, and security impact.

Small, focused pull requests are easier to review. Use the issue templates for
bugs, features, and documentation gaps.

## Coding standards

See `docs/standards.md` for the full coding standards. Key rules:

- Use `AppError` with `ErrorKind` for structured errors
- Use `Secret<T>` for sensitive values; never log or expose them
- Use tracing macros for structured logging
- Add tests for all public API changes
- Update `CHANGELOG.md` for user-facing changes
- Run benchmarks before performance optimizations

## Quality checklist

Before opening a pull request:

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --workspace --all-features`
- [ ] `cargo test --workspace --all-features`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo doc --workspace --no-deps`
- [ ] Documentation build: `cd docs && pnpm install --frozen-lockfile && pnpm build`
- [ ] Generated project compiles: `cargo run -p arqen --features cli --bin arqen -- new test-project && cargo check --manifest-path test-project/Cargo.toml`

## Documentation drift

Before a release, verify:

- [ ] `docs/commands.md` matches `arqen --help` output
- [ ] `docs/configuration.md` matches the actual config file format
- [ ] `docs/feature-status.md` matches implementation status
- [ ] Code examples in docs compile or are labeled as conceptual
