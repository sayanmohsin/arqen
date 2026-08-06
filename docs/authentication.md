# Authentication and authorization

Arqen provides authentication building blocks; your application owns identity
issuance, user lookup, policy decisions, and secret rotation.

Available integrations include API keys, JWT validation, and session-token
hooks through the `Authentication` trait. Policy combinators include
`AllOf`, `AnyOf`, and `RequireRole`. HTTP applications can use
`auth_middleware`, `optional_auth_middleware`, and the `Authenticated`
extractor.

```rust
let protected = Router::new()
    .route("/reports", get(reports))
    .layer(auth_middleware(auth));
```

Use HTTPS, keep signing keys and API keys outside source control, prefer
short-lived credentials, and return the same external error shape for failed
authentication and authorization. API keys are compared in constant time and
stored as hashes by the built-in helper; applications remain responsible for
storage and rotation.

Arqen does not currently promise attribute macros such as `#[auth(role =
"admin")]`. Declarative policy can be layered on top of the public traits once
an application has established its identity model.
