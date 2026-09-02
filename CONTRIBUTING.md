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
- [ ] Release documentation audit: `bash scripts/check-release-docs.sh`
- [ ] Generated project compiles: `cargo run -p arqen --features cli --bin arqen -- new test-project && cargo check --manifest-path test-project/Cargo.toml`

## Native RocksDB build (thingd-native feature)

The `thingd-native` feature pulls in a transitive native dependency chain:

    arqen → thingd → rocksdb → librocksdb-sys → bindgen → clang-sys

`librocksdb-sys` builds RocksDB from C++ source and requires `libclang` at
build time. On macOS, the linker must resolve a `libclang.dylib` whose LLVM
symbols match the rustc-bundled LLVM version — otherwise the build fails with
a symbol mismatch or missing library error.

### macOS setup

Run the toolchain detection script before building with `thingd-native`:

    source scripts/setup-llvm.sh
    cargo check --features thingd-native

The script detects the LLVM major version from `rustc --version --verbose`,
finds the matching Homebrew formula (`llvm@<MAJOR>`), and sets `LIBCLANG_PATH`,
`LLVM_CONFIG_PATH`, and `DYLD_LIBRARY_PATH`. If no matching formula exists, it
prints a clear install command.

### Linux / CI

CI and Docker use apt-managed packages. The standard installation is sufficient:

    sudo apt-get install -y clang libclang-dev llvm

### Docker

The `Dockerfile` builder stage installs `clang`, `libclang-dev`, and `llvm`.
The default production build uses HTTP-only mode (`--features cli,http-client`)
which does not require the native toolchain.

## Documentation drift

Before a release, verify:

- [ ] `docs/commands.md` matches `arqen --help` output
- [ ] `docs/configuration.md` matches the actual config file format
- [ ] `docs/feature-status.md` matches implementation status
- [ ] Code examples in docs compile or are labeled as conceptual
