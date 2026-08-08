# API stability

Arqen is approaching beta, but not every module has the same compatibility
promise.

## Supported surface

Use `arqen::prelude` for application code. The prelude includes application
state, configuration, storage factories and adapters, validation, modules,
jobs, health, and vendor-neutral metrics. `arqen::http` is the supported HTTP
facade; applications do not need to import Axum directly.

The `ThingdBackend` contract, memory backend, native adapter, HTTP policy, and
scoped backend are supported APIs. Native thingd's synchronous store remains
available as an explicit advanced API, while Arqen's async adapter always
runs it on blocking threads.

## Experimental surface

Cloud storage, synchronization, JWKS rotation, and cursor-based sync APIs are
experimental or blocked until thingd publishes versioned public contracts.
Arqen will discover those capabilities and expose lifecycle/readiness state;
it will not define a private replication protocol.

Experimental modules may change without the normal deprecation window. They
are marked in Rust documentation and the feature-status table.

## SemVer and deprecations

Public types and trait methods follow SemVer. Breaking changes require a
major release. Deprecated APIs remain available for at least one minor release
when practical and include a replacement in their Rust docs and the migration
guide. Internal modules under `crates/arqen/src` are not a compatibility
promise unless re-exported from the crate root or prelude.

Before upgrading, run:

```bash
cargo test -p arqen --all-features
cargo clippy -p arqen --all-targets --all-features -- -D warnings
cargo doc -p arqen --all-features --no-deps
```
