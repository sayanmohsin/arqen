//! Retry-aware startup operations for remote Thingd deployments.

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use crate::core::{AppError, ErrorKind};
use crate::thingd::ThingdBackend;

/// Bounded retry settings for application startup work such as catalog seeding.
#[derive(Debug, Clone)]
pub struct BootstrapPolicy {
    pub max_attempts: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
}

impl Default for BootstrapPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 12,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(30),
        }
    }
}

/// Retry an operation while Thingd is unavailable during startup.
pub async fn retry<F, Fut, T>(policy: BootstrapPolicy, mut operation: F) -> Result<T, AppError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    let attempts = policy.max_attempts.max(1);
    let mut backoff = policy.initial_backoff;
    for attempt in 1..=attempts {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(error) if attempt < attempts && retryable(&error) => {
                tokio::time::sleep(backoff).await;
                backoff = backoff.saturating_mul(2).min(policy.max_backoff);
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!("bootstrap attempts are always at least one")
}

/// Retry the standard Thingd seed operation until the remote store is ready.
pub async fn seed_with_retry(
    backend: Arc<dyn ThingdBackend>,
    policy: BootstrapPolicy,
) -> Result<(), AppError> {
    retry(policy, move || {
        let backend = Arc::clone(&backend);
        async move { backend.seed().await }
    })
    .await
}

fn retryable(error: &AppError) -> bool {
    matches!(
        error.kind,
        ErrorKind::Unavailable | ErrorKind::Timeout | ErrorKind::Dependency
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn retries_transient_failures() {
        let calls = Arc::new(AtomicU32::new(0));
        let result = retry(
            BootstrapPolicy {
                max_attempts: 3,
                initial_backoff: Duration::ZERO,
                max_backoff: Duration::ZERO,
            },
            {
                let calls = Arc::clone(&calls);
                move || {
                    let attempt = calls.fetch_add(1, Ordering::Relaxed);
                    async move {
                        if attempt < 2 {
                            Err(AppError::new(ErrorKind::Unavailable, "not ready"))
                        } else {
                            Ok("ready")
                        }
                    }
                }
            },
        )
        .await
        .unwrap();
        assert_eq!(result, "ready");
        assert_eq!(calls.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn does_not_retry_permanent_errors() {
        let calls = Arc::new(AtomicU32::new(0));
        let error = retry(
            BootstrapPolicy {
                max_attempts: 3,
                initial_backoff: Duration::ZERO,
                max_backoff: Duration::ZERO,
            },
            {
                let calls = Arc::clone(&calls);
                move || {
                    calls.fetch_add(1, Ordering::Relaxed);
                    async { Err::<(), _>(AppError::new(ErrorKind::Validation, "invalid seed")) }
                }
            },
        )
        .await
        .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }
}
