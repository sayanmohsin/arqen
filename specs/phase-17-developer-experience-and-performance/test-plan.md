# Test plan

## Clean checkout

Run the following from a clean clone:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-features
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo doc --workspace --no-deps
cargo package --package arqen --allow-dirty --no-verify
```

## CLI

```bash
cargo run -p arqen --features cli --bin arqen -- --help
cargo run -p arqen --features cli --bin arqen -- --version
cargo run -p arqen --features cli --bin arqen -- new phase17-smoke
cargo check --manifest-path phase17-smoke/Cargo.toml
cargo run -p arqen --features cli --bin arqen -- generate module audit
cargo run -p arqen --features cli --bin arqen -- check --help
```

Repeat generation in an existing directory and verify it fails without
overwriting files. Exercise invalid flags, missing config, JSON output, and
Ctrl-C shutdown in automated integration tests.

## Documentation

```bash
cd docs
pnpm install --frozen-lockfile
pnpm format:check
pnpm lint
pnpm build
```

Run Lychee against tracked Markdown files from a clean checkout. Verify the
deployed Pages URL after deployment rather than using it as the only local
build check.

## Performance

```bash
cargo bench --bench framework
```

Store the benchmark command, feature flags, hardware, fixture sizes, and
results in the phase handoff. Compare against the committed baseline and
investigate any regression beyond the agreed tolerance.
