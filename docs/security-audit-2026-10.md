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

## Phase 18 implementation follow-up (2026-10-11)

The follow-up review found and addressed these additional gaps. A new
`RequestContext` regression test verifies that subject, tenant, and instance
come from authenticated extensions, while anonymous requests remain unscoped.

### Fixed: scoped search scanned the unbounded backend result set — Medium

`ScopedThingdBackend::search` previously requested `usize::MAX`, then filtered
out results outside the active tenant/instance scope. This allowed a broad
search to allocate and process the entire shared backend result set. The
adapter now requests at most 10,001 matches, returns a typed unavailable error
when the backend reports more than 10,000 matches, and paginates the filtered
scope locally. Reproduce the former behavior by issuing a broad search against
a shared backend and inspecting the search limit passed by the scope wrapper.
The current cap bounds work but can reject a broad legitimate query; the next
improvement is a public server-side collection/scope filter with bounded
keyset pagination.

### Fixed: HTTP Thingd errors lost their public detail — Low

The public Thingd REST error contract uses `error.detail`; the Arqen client
looked only for `error.message`, so callers received the generic fallback.
`HttpThingdBackend` now reads `detail` and maps 400/422 to validation and 501
to not-implemented. A 409 continues to map to conflict. The new HTTP test
checks the CAS conflict mapping and returned detail.

### Fixed: automatic retries could repeat non-idempotent mutations — Medium

The shared HTTP request helper retried all mutation verbs after transport
errors and 429/5xx responses. Thingd did not consume the adapter's generic
`Idempotency-Key` header, so this could repeat an event, delete, or other
mutation after an ambiguous response. Retries are now limited to GET and queue
pushes that carry a caller-provided, stable body idempotency key. Conditional
CAS writes are sent once. A local HTTP test verifies the queue key stays the
same over a 503 retry.

### Fixed: documented queue idempotency was not honored by Thingd REST — Medium

The public queue REST documentation promised `idempotencyKey`, but the route
always generated a UUID. Thingd 0.92.3 now uses a validated 1–256 byte key as
the queue job ID, allowing the engine's existing duplicate-ID behavior to
return the original job. Arqen's HTTP adapter sends that key and `delayMs`,
and retries only keyed pushes with the same key. A server regression test
verifies a repeated request returns the same job ID; Arqen's adapter contract
is available for a Thingd 0.92.3 server.

### Fixed: production validation allowed credentialless auth and wildcard CORS — Medium

`validate_production()` previously accepted `auth.enabled=true` with neither
JWT secret nor API keys and allowed `*` in the CORS allowlist. It now requires
a configured credential source, rejects HMAC JWT secrets and API keys shorter
than 32 bytes, rejects wildcard origins, and requires HTTPS when a remote
Thingd token is configured. Errors redact credential values. Regression tests
cover these checks. This built-in profile does not currently model external
custom-auth providers; applications using those must configure a supported
credential or use their own startup validation.

### Fixed: conditional writes lacked an adapter contract — Medium

`ThingdBackend` had no CAS method although public Thingd already supports
`If-Match` and create-only version zero. Arqen's object type also omitted the
version returned by Thingd. The additive conditional-write and versioned-read
methods return `NotImpl` by default for third-party adapters. Memory and native
adapters use authoritative versions; HTTP reads expose Thingd's version and
conditional writes send `If-Match` without automatic retry; cache and scoped
decorators preserve the operations. Tests cover versioned reads, create,
matching writes, stale conflicts, and namespace isolation. At the initial
audit pass, the native regression test could not run because `libclang.dylib`
was not on the toolchain search path. The Phase 18 follow-up ran the shared
contract successfully with the installed Homebrew LLVM 22 library; see the
validation record below.

## Remaining risks and next phase gates

1. Run the new HTTP isolation and mutation contracts against Thingd 0.92.3;
   add batch-failure and adversarial-scope-ID cases after the server contract
   run.
2. Add Thingd public keyset pagination with query-bound versioned cursors and
   non-snapshot semantics. Preserve offsets and propagate the additive page
   shape through Rust, native, REST, Node, CLI/MCP, and Arqen.
3. Add explicit API/worker process roles, bounded backpressure, vendor-neutral
   exporter hooks, and restart-based backup/restore exercises.
4. Start JWKS rotation and tenant/instance discovery only after a versioned
   public Cloud customer API exists. Do not depend on Cloud internals.

Do not mark Phase 18 complete until the acceptance tests for those gates pass.

### Validation recorded for this follow-up

- Arqen default tests: 340 passed, 14 ignored; HTTP-client feature tests: 353
  passed, 15 ignored. Arqen Clippy and CLI all-target check passed.
- Thingd workspace Rust tests: 454 passed. The queue idempotency and
  oversized-key regressions passed; server Clippy and `pnpm check:docs` passed.
- Arqen docs Prettier check and VitePress build passed. VitePress still warns
  that a generated client chunk exceeds 500 kB.
- At the initial audit pass, the Arqen native run was blocked because
  `libclang.dylib` was not on the toolchain search path. The follow-up below
  passed after using the installed Homebrew LLVM library.

### Thingd 0.92.3 follow-up

- `cargo info thingd@0.92.3` downloaded the crate from crates.io; `cargo info
thingdb@0.92.3` confirmed its required ThingDB release is available too.
  Release PR [#270](https://github.com/sayanmohsin/thingd/pull/270) records the
  queue idempotency-key validation and replay changes. Arqen's existing
  `>=0.92.2, <0.93.0` range accepts the release; only the two Cargo.lock
  package resolutions changed.
- The shared Phase 18 contract passed for memory and native adapters. The full
  HTTP-client suite passed with 355 tests, and native plus HTTP all-target
  Clippy passed with warnings denied.
- Arqen now sends `delayMs` as well as `idempotencyKey`. HTTP unit coverage
  checks that both values remain stable across a retry and that an unkeyed
  failed push is attempted once. HTTP integration contracts cover scope
  isolation, CAS, replay after a simulated lost success response, and delay
  visibility using unique queues and resource names.
- The three Thingd 0.92.3 integration contracts were attempted against the
  configured `api.thingd.cloud` host at Arqen's direct `/v1` REST path, first
  without auth and then with the supplied project key. Both attempts returned
  HTTP 401 before assertions. Arqen's `HttpThingdBackend` calls the direct
  Thingd REST API under `/v1` and expects
  the engine's `THINGD_AUTH_TOKEN`/`auth.token`; Cloud project keys target the
  separate MCP endpoint. No valid direct-engine credential was tested. Set
  `ARQEN_THINGD_TEST_URL` to a direct Thingd 0.92.3 REST endpoint and
  `ARQEN_THINGD_AUTH_TOKEN` to that server's token, then run
  `cargo test -p arqen --features http-client --test http_thingd_contract
thingd_0923 -- --ignored`. The endpoint does not expose a stable engine
  version field, and `thingd-server` is not published as a separate crate.
  All test data uses unique identifiers; no test calls a global reset.
- Phase 18.4 keyset pagination remains pending because Thingd 0.92.3 does not
  expose the page API.

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

1. **18.1 Isolation and scoped access:** preserve the existing typed request
   context and scoped backend, close remaining operation-level isolation gaps,
   and keep search scans bounded.
2. **18.2 Safe mutations:** preserve adapter parity for CAS and idempotency,
   including version zero, stale writes, stable retries, and duplicate queue
   behavior.
3. **18.3 Remote and production boundary:** retain typed HTTP errors and safe
   retry behavior; validate production auth, CORS, storage, and logging.
   Cloud identity remains a gated optional lane.
4. **18.4 Scale and operations:** add public keyset pagination, process roles,
   backpressure, exporter hooks, and backup/restore recovery exercises.

## Additional developer experience improvements

- Public Thingd's CLI API-key create contract appears inconsistent: the client
  returns the token at the top level, while the command reads `key.token`, so
  it may print the fallback instead of the new secret. The command has no
  existing-key retrieval action. This is a Thingd follow-up, not changed here.
- The Arqen docs build succeeds but emits a client chunk over 500 kB. Lazy-load
  the heavier diagrams or split the bundle to improve docs startup time.
- The Thingd docs build falls back to plain text highlighting for `thingd` and
  `caddyfile` code fences. Register those grammars or use supported language
  labels so examples receive syntax highlighting.
