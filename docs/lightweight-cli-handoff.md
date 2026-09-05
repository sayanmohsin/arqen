# Lightweight CLI handoff — 2026-09-05

The basic `cli` feature no longer enables native migration. Migration users must
explicitly enable `cli,thingd-migration`; default and HTTP-only users need no
native toolchain. See lightweight-cli.md for the installation contract.

Watcher diagnostics are verbose-only, with optional cargo-watch and a one-second
debounce. Readiness handles spawn failure, early service exit, timeout and Ctrl+C.
Unix signal registration precedes spawning; service process groups receive shutdown.
HTTP-only test utilities and the HTTP benchmark are gated for minimal builds.

Local verification:

- Dependency graph regression script: default, HTTP client and CLI exclude native dependencies.
- Basic CLI help launch passed with LIBCLANG_PATH, DYLD_LIBRARY_PATH and LLVM_CONFIG_PATH unset.
- Minimal tests: 195 unit + 9 contract tests passed.
- Default tests: 318 unit + 12 contract tests passed.
- CLI tests: 340 unit + 20 CLI + 12 contract tests passed.
- All-feature tests: 355 unit + 20 CLI + 12 contract tests passed.
- Clippy with warnings denied: minimal, default, CLI and all-feature configurations passed.
- Formatting and diff checks passed.

Initial socket tests needed execution outside the filesystem sandbox. A live
cancellation regression exposed a startup signal-registration race, fixed and
verified in the final matrix. No release, commit or push performed. GoodOne's
published pin remains unchanged; an isolated temporary checkout with a Cargo
patch compiled successfully against this local Arqen library.

Cross-repository live launcher verification is recorded in GoodOne's
docs/development-startup-handoff.md. Linux/Windows runtime behavior and remote CI
have not been executed in this macOS session.
