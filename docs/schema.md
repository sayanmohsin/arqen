---
title: Thingd schema
description: How to keep, validate, inspect, and operate a versioned Thingd schema with Arqen.
---

# Thingd schema

A schema is the versioned contract for the Thingd data your backend uses. Keep
it with the application, validate it before startup, and inspect the remote
service before applying any migration.

## The short workflow

<MermaidDiagram type="schema" />

Arqen deliberately stops at validation and inspection. Thingd remains
authoritative for schema compatibility, migration history, indexes, and data
rewrites.

## 1. Store the schema

Put the schema in an application-owned path and configure it in `arqen.toml`:

```toml
[storage]
mode = "native"
persistent_path = "/var/lib/catalog-api/data"
schema_path = "schema.thingd"
```

Or use the environment variable:

```bash
export ARQEN_THINGD_SCHEMA_PATH="$PWD/schema.thingd"
```

Do not put secrets in the schema file. Encryption keys and Thingd bearer
tokens belong in a secret manager or server-side environment variables.

## 2. Validate a local schema

Use the CLI to load the file and, when a Thingd service is available, ask the
service's authoritative parser to validate it:

```bash
arqen thingd schema-validate schema.thingd \
  --url http://127.0.0.1:8770
```

This checks the local file and reports a stable source hash. A missing or
invalid schema should stop a production startup rather than silently selecting
an empty or incompatible schema.

## 3. Inspect the remote service

Before changing data, inspect what the server currently has:

```bash
arqen thingd schema-remote http://127.0.0.1:8770
```

The report includes the current remote schema and migration history. Compare
that report with the version committed in your application before planning a
change.

## 4. Apply migrations deliberately

Arqen does not apply schema migrations automatically. Use Thingd's supported
operator workflow after reviewing:

- the current remote schema;
- the target schema and source hash;
- the migration history;
- affected collections and indexes;
- backup and restore readiness;
- application compatibility during the transition.

Keep the migration separate from application startup. A backend restart must
not unexpectedly delete, rewrite, or reshape data.

## Native and HTTP differences

| Mode   | Schema responsibility                                                                                              |
| ------ | ------------------------------------------------------------------------------------------------------------------ |
| Memory | Useful for tests and prototypes; no durable schema contract is expected.                                           |
| Native | Arqen loads the local `.thingd` schema and validates it before a strict production startup.                        |
| HTTP   | The remote Thingd service owns the active schema; Arqen can validate the local file and inspect the remote report. |
| Cloud  | Future public contract; do not depend on private cloud control-plane data.                                         |

## Schema and data movement

The native-to-HTTP migration workflow preserves the source and moves logical
records through Thingd snapshot `2.0.0` JSONL. It is not a filesystem-format
conversion:

```bash
arqen thingd migrate \
  --source /var/lib/catalog-api/data \
  --destination https://thingd.internal \
  --auth-token "$THINGD_AUTH_TOKEN" \
  --check
```

Use `--dry-run` to create the resumable spool without importing, and
`--resume` to retry an interrupted import. The destination must be empty, and
the operator should validate object, event, queue, index, and health results
afterward. Read [Migration](./migration.md) for the full cutover procedure.

## Related contracts

- [Thingd integration](./thingd-integration.md): adapters, replication, and
  recorded data families.
- [Configuration](./configuration.md): storage, schema, encryption, and sync
  settings.
- [Deployment](./deployment.md): startup validation, backups, and production
  modes.
