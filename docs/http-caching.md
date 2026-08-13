# HTTP caching

Arqen does not automatically cache every `GET` response. Applications must
opt in because user- and tenant-scoped responses must not become public cache
entries.

```rust
use arqen::http::HttpCachePolicy;

let policy = HttpCachePolicy::public("\"catalog-v42\"", 300);
```

The cache middleware adds `ETag`, `Cache-Control`, and
`Vary: Accept-Encoding`. A matching `If-None-Match` request receives `304 Not
Modified` without a response body. Use `HttpCachePolicy::private` for
authenticated responses and omit the policy for personalized data unless the
application has stronger cache isolation.
