# API stability

Arqen’s public APIs do not all have the same compatibility level. Use the
labels below when choosing an integration.

## Supported APIs

The following are intended for application code:

- `arqen::prelude` for application state, configuration, storage factories,
  validation, modules, jobs, health, and metrics;
- `arqen::http` helpers and re-exported HTTP types;
- the Arqen-owned `ThingdBackend` contract and memory adapter;
- the HTTP adapter and its public `v1` compatibility probe;
- the optional `thingd-native`, `thingd-maintenance`, `thingd-connectors`, and
  `thingd-migration` features for local native development and migration;
- configuration, validation, error, health, job, module, OpenAPI, and testing
  types documented in the guides;
- Thingd schema reports and the current Thingd sync client types.

Supported does not mean that Arqen chooses your production policy. You still
need to test recovery, credentials, timeouts, tenant isolation, backups, and
the exact Thingd service used by your deployment.

## Experimental APIs

Sync workers, schema inspection, native-to-HTTP migration, and cloud-related
integration are operational or experimental features. They can change as the
public Thingd contracts and deployment experience develop.

Thingd remains responsible for replication, conflicts, encryption, tombstones,
and migrations. Arqen provides the client, configuration, lifecycle, and
metrics integration around those capabilities. Arqen core does not promise
that arbitrary Thingd engine versions are source-compatible with its native
adapter.

## Thingd compatibility

The native feature accepts Thingd <CurrentVersion kind="native-thingd" :label="false" />. The maintenance and
connector features are opt-in native extensions. A minor Thingd upgrade
requires updating Arqen’s range and passing the native contract suite; it may
require an Arqen release if native APIs changed.

The HTTP adapter targets the public Thingd REST API `v1`. Call
`HttpThingdBackend::check_compatibility()` during startup and keep the service
out of readiness when the probe fails. The public health response currently
proves API availability and readiness, not a concrete engine version.

## SemVer and deprecations

Public types and trait methods follow SemVer. Breaking changes require a major
release. Deprecated APIs remain available for at least one minor release when
practical and include a replacement in the Rust documentation and migration
guide.

Modules that are not re-exported from the crate root or prelude should not be
treated as a stable application dependency.

Before upgrading, run:

```bash
cargo test -p arqen --all-features
cargo clippy -p arqen --all-targets --all-features -- -D warnings
cargo doc -p arqen --all-features --no-deps
```
