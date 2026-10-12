# Security audit and dependency refresh (2026-10-11)

## Scope and result

Reviewed Arqen authentication and authorization, configuration secrets,
request logging, Thingd HTTP integration, tool authorization, CLI credential
handling, dependency advisories, and the planned Phase 18 hardening work. Also
refreshed dependencies and checked integration compatibility against public
Thingd 0.92.2. Thingd Cloud was outside scope.

One authentication bug was confirmed and fixed: `JwtAuth::with_issuer()` did
not enforce the configured issuer. A regression test now verifies that a token
from another issuer is rejected. Arqen's RSA JWT feature was replaced with the
AWS-LC backend to remove the unpatched RSA advisory from the dependency graph.

## Findings

### Fixed: JWT issuer was not validated — Medium

Before the fix, `with_issuer()` stored the expected issuer without applying it
to `jsonwebtoken::Validation`. A correctly signed, unexpired token from an
untrusted issuer could therefore pass issuer validation. The method now calls
`Validation::set_issuer`; tests cover both the accepted expected issuer and a
rejected wrong issuer in `crates/arqen/src/auth/mod.rs`. To reproduce, sign a
token with the configured verification key, a future `exp`, and
`iss: "unexpected-issuer"`, then authenticate it with
`JwtAuth::new_secret(key).with_issuer("expected-issuer")`; before the fix it
was accepted, and now it returns `AuthError::Invalid`.

### Remaining: transitive `lru` advisory — Low, dependency-level

RustSec reports RUSTSEC-2026-0253 for `lru 0.16.4`, used transitively by
Tantivy. The fix is in `lru >=0.18.2`, while the current Tantivy dependency
requires the 0.16 line. Thingd's current cache stores `usize` keys and `Block`
values; the advisory's panic-safety condition was not demonstrated in this
usage. Keep the warning visible and revisit when Tantivy permits a patched
`lru`; do not suppress it without a compatibility review.

### Review note: `Secret<T>` serialization — defense-in-depth

`Secret<T>` redacts `Debug` and `Display`, but derives `Serialize`, which emits
the wrapped value. Callers that serialize an `AuthConfig`, `StorageConfig`, or
`SyncConfig` must treat the result as credential-bearing. The audited code did
not show an automatic logging or public-response path serializing these
configs. Consider a redacted/export-safe serialization API or explicitly
skipped secret fields if config serialization is added to user-facing output.

## Other controls reviewed

- API key hashes use SHA-256 and comparisons use `subtle` constant-time
  comparison.
- Request logging redacts sensitive query parameters and omits authorization
  and cookie headers.
- Tool execution checks declared authorization scopes before invoking a
  handler.
- Remote Thingd credentials are supplied through `Secret<String>` config
  values; production deployments should use HTTPS as described in
  `docs/security.md`.
- CLI bearer-token options can be visible in process arguments on systems that
  expose command lines to other local users. Prefer environment or secret
  providers for shared hosts; a future CLI improvement could add hidden
  interactive input and file-descriptor input.

These are source-level checks, not a production deployment penetration test.
Tenant isolation, proxy/TLS configuration, secret-provider policy, and
application-specific authorization still need validation in each deployment.

## Dependency and integration changes

- Updated Cargo lockfiles and manifests in Arqen, its standalone examples,
  Watchloom, and Thingd. The Arqen Thingd range now accepts `>=0.92.2, <0.93.0`.
- Updated pnpm manifests and lockfiles for Arqen docs and Thingd's workspace,
  examples, and docs. The NestJS example stays on TypeScript 6 because Nest CLI
  12 does not support TypeScript 7's compiler API yet.
- Arqen docs pins patched Vite 6.4.4 and KaTeX 0.18.2 through the pnpm
  workspace overrides. Thingd docs pins patched Vite 6.4.4.
- npm advisory audits report no known vulnerabilities for either docs tree or
  the Thingd workspace. Cargo audit still reports the transitive `lru`
  advisory described above.

## Recommended Phase 18 order

1. **Isolation and mutation safety:** add verified typed request context,
   tenant/instance-scoped storage helpers, multi-subject isolation contracts,
   optimistic concurrency, and durable idempotency.
2. **Production boundary:** add production configuration guardrails,
   tenant-aware HTTP adapter contracts, typed Thingd error parsing, explicit
   connect/request timeouts, and retries limited to safe operations.
3. **Identity and sync integration:** support rotating JWKS keys and a public
   cloud customer API boundary; consume Thingd-owned sync capabilities and
   durable event cursors without duplicating replication semantics.
4. **Operations:** separate API and worker roles, add pagination/backpressure,
   schema/version envelopes, exporter hooks, and backup/restore and recovery
   exercises.

## Additional developer experience improvements

- The Arqen docs build succeeds but emits a client chunk over 500 kB. Lazy-load
  the heavier diagrams or split the bundle to improve docs startup time.
- The Thingd docs build falls back to plain text highlighting for `thingd` and
  `caddyfile` code fences. Register those grammars or use supported language
  labels so examples receive syntax highlighting.
