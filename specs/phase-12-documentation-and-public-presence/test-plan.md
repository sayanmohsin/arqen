# Test plan

## Local checks

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cd docs
pnpm install --frozen-lockfile
pnpm build
```

## Static checks

- Extract internal Markdown links and ensure every target page exists.
- Compare CLI command names in `docs/commands.md` with the `Commands` enum.
- Search public docs for unqualified “complete”, “production-ready”, or cloud claims and review each result.
- Confirm `git diff --name-only` contains no files under sibling repositories.

## CI checks

GitHub Actions runs the docs build/deploy, Rust checks, Markdown/link checks,
dependency audits, and Docker Compose configuration smoke validation. A full
image build remains a parent-context operation because the current Dockerfile
needs the sibling thingd crate.
