# Thingd startup bootstrap

Remote Thingd services may become reachable after the Arqen process starts.
For application-owned catalog seeding, use the retry-aware helper instead of
an uncoordinated detached task:

```rust
use std::sync::Arc;
use arqen::{seed_with_retry, BootstrapPolicy, ThingdBackend};

seed_with_retry(
    Arc::clone(&thingd),
    BootstrapPolicy::default(),
).await?;
```

`BootstrapPolicy` bounds attempts and exponential backoff. Only transient
unavailable, timeout, and dependency errors are retried. Validation,
authorization, and other permanent failures return immediately.

For an operator-driven seed, use:

```bash
arqen thingd seed https://thingd.internal --token "$THINGD_AUTH_TOKEN"
```

Arqen does not seed application data automatically.
