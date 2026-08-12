# Migration

Upgrade notes for Arqen releases.

## Native Thingd to standalone HTTP Thingd

Use the explicit migration workflow when moving an embedded native store to a
standalone Thingd server. It uses Thingd logical snapshot `2.0.0` JSONL; it is
not a filesystem-format conversion and never modifies the source.

```bash
arqen thingd migrate \
  --source /data/app/thingd.db \
  --destination https://thingd.example.com \
  --auth-token "$THINGD_AUTH_TOKEN" \
  --batch-size 100 \
  --resume
```

Run `--check` to validate the source, snapshot-capable destination,
authentication, destination emptiness, and record counts. `--dry-run` performs
the same validation and creates a resumable JSONL spool without importing it.
The spool is written beside the source by default; pass an application-owned
`snapshot_path` through the library API when another location is required.

The library API is `NativeToHttpMigrator::{validate,migrate}` with
`ThingdMigrationOptions`. A migration uses bounded pages and a bounded HTTP
stream. Retry with `--resume` after interruption; accepted object, event, and
queue records are safe to replay because Thingd uses stable keys and event
idempotency keys. Keep application writes paused for the export/import window
when a consistent cutover is required; Arqen does not claim a live-write
snapshot guarantee.

The destination must be empty. Arqen does not delete or overwrite a
non-empty destination. Destination credentials are sent only as a bearer token
over the configured URL; use HTTPS for remote servers. Thingd-owned schema
metadata and functional indexes are re-applied after records when the server
exposes those APIs. Replication system records are excluded unless
`--include-replication` is explicitly supplied.

Search indexes are destination-owned and should be rebuilt or verified after
the import. Validate the result with a fresh Thingd snapshot, object/event/job
counts, representative object IDs and bodies, event sequence order, queue
terminal states, and application health checks. Native mode is appropriate for
small, single-process deployments; use standalone HTTP mode below 2 GB RAM
when large search or enrichment workloads are expected.

## Alpha → beta startup behavior

`arqen start` is now the strict production path. It validates configuration
before binding and rejects memory storage, incomplete native or HTTP storage,
unsupported cloud storage, disabled authentication, pretty logs, missing
credentials, and unsafe worker settings. Use `arqen dev` for local memory-mode
development and tests. Applications that call the library directly should
also call `AppConfig::validate_production()` before starting a production
listener.

The new `ScopedThingdBackend` should wrap an application backend whenever data
is tenant- or user-owned. Its tenant, instance, and subject values must come
from verified authentication or trusted server configuration, never from a
request body.

## 0.3 → 0.4

### Single published crate

Arqen moved from a multi-crate workspace to a single published crate. The
CLI binary is feature-gated behind `cli` and is not a separate package.

**Action required:**

Update your dependency:

```toml
[dependencies]
arqen = { version = "0.4", features = ["logging", "http-server"] }
```

Remove any workspace-level references to internal crates that no longer exist.

### Config section rename

The `[thingd]` section in `arqen.toml` was renamed to `[storage]`.

Before (0.3):

```toml
[thingd]
mode = "memory"
```

After (0.4):

```toml
[storage]
mode = "memory"
```

The environment variable `ARQEN_THINGD_URL` still sets the HTTP URL for
thingd connectivity, but the config file key is now `storage.http_url`.

### Module trait changes

The `Module` trait now requires `Send + Sync` on implementors and provides:

- `register(&self, ctx: &mut ModuleContext<'_>)` for explicit tool and
  health check registration.
- `dependencies()` for declaring inter-module dependencies.
- `health_check()` returning `ModuleHealth` for module-level health.

If you have custom modules, update them to implement the current trait
shape. The `EmptyModule` test helper is available for simple cases.

### Breaking changes

See `CHANGELOG.md` for the full list. Key breaking changes:

- `ModuleBuilder::validate()` now returns `Result<(), ModuleGraphError>`.
- `HealthReport::probe_type` field added.
- `HealthRegistry::register()` now takes `Arc<dyn HealthCheck>` instead of
  boxed trait objects.

## General upgrade steps

1. Update the version in `Cargo.toml`.
2. Run `cargo update -p arqen`.
3. Fix any compilation errors from API changes.
4. Run `cargo test -p arqen --all-features` to verify.
5. Update any `arqen.toml` config files to match new section names.
