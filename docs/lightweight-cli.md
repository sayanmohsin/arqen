# Lightweight CLI and native storage

Install the basic CLI with `cargo install arqen --locked --features cli`.
The basic CLI, default library, and HTTP client do not require Thingd or LLVM.
Native storage is opt-in via `thingd-native`. Migration commands require
`cargo install arqen --locked --features cli,thingd-migration`.

Native builds require the Thingd native toolchain, including libclang. On macOS,
configure a compatible Homebrew LLVM installation before building; ensure both
`LIBCLANG_PATH` and the dynamic loader search path point to its library directory.
Arqen does not install system packages or silently replace durable storage with memory.

`arqen dev --watch` requires optional cargo-watch. File-change explanations are
verbose-only. `arqen up --wait-ready` reports readiness only after the configured
HTTP probe succeeds, and stops the other services on startup failure or exit.
