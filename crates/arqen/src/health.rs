//! Health and readiness module for Arqen.
//!
//! Provides dependency checks with timeouts and degraded states.

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Health status of a dependency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// Dependency is healthy.
    Healthy,
    /// Dependency is degraded but functional.
    Degraded { reason: String },
    /// Dependency is unhealthy.
    Unhealthy { reason: String },
}

impl HealthStatus {
    /// Check if status is healthy.
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    /// Check if status is degraded.
    pub fn is_degraded(&self) -> bool {
        matches!(self, HealthStatus::Degraded { .. })
    }

    /// Check if status is unhealthy.
    pub fn is_unhealthy(&self) -> bool {
        matches!(self, HealthStatus::Unhealthy { .. })
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded { reason } => write!(f, "degraded: {}", reason),
            HealthStatus::Unhealthy { reason } => write!(f, "unhealthy: {}", reason),
        }
    }
}

/// Trait for health checks.
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Name of the dependency.
    fn name(&self) -> &str;

    /// Perform the health check.
    async fn check(&self) -> HealthStatus;

    /// Timeout for the check.
    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}

/// Result of a single health check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Name of the dependency.
    pub name: String,
    /// Status of the check.
    pub status: HealthStatus,
    /// Duration of the check in milliseconds.
    pub duration_ms: u64,
}

/// Overall health report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Overall status.
    pub status: HealthStatus,
    /// Individual check results.
    pub checks: Vec<CheckResult>,
    /// Timestamp of the report.
    pub timestamp: String,
}

/// Registry for health checks.
pub struct HealthRegistry {
    checks: Vec<Arc<dyn HealthCheck>>,
}

impl HealthRegistry {
    /// Create a new health registry.
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }

    /// Register a health check.
    pub fn register(&mut self, check: Arc<dyn HealthCheck>) {
        self.checks.push(check);
    }

    /// Run all health checks and return a report.
    pub async fn check_all(&self) -> HealthReport {
        let mut results = Vec::new();
        let mut overall_status = HealthStatus::Healthy;

        for check in &self.checks {
            let start = Instant::now();
            let status = run_check_with_timeout(check.as_ref(), check.timeout()).await;
            let duration_ms = start.elapsed().as_millis() as u64;

            results.push(CheckResult {
                name: check.name().to_string(),
                status: status.clone(),
                duration_ms,
            });

            // Update overall status
            match &status {
                HealthStatus::Unhealthy { .. } => {
                    overall_status = status;
                    break;
                }
                HealthStatus::Degraded { .. } => {
                    if overall_status.is_healthy() {
                        overall_status = status;
                    }
                }
                _ => {}
            }
        }

        HealthReport {
            status: overall_status,
            checks: results,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl Default for HealthRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Run a health check with a timeout.
async fn run_check_with_timeout(check: &dyn HealthCheck, timeout: Duration) -> HealthStatus {
    match tokio::time::timeout(timeout, check.check()).await {
        Ok(status) => status,
        Err(_) => HealthStatus::Unhealthy {
            reason: format!("check timed out after {}ms", timeout.as_millis()),
        },
    }
}

/// Always healthy check.
pub struct AlwaysHealthy;

#[async_trait]
impl HealthCheck for AlwaysHealthy {
    fn name(&self) -> &str {
        "always_healthy"
    }

    async fn check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

/// Always degraded check.
pub struct AlwaysDegraded {
    reason: String,
}

impl AlwaysDegraded {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

#[async_trait]
impl HealthCheck for AlwaysDegraded {
    fn name(&self) -> &str {
        "always_degraded"
    }

    async fn check(&self) -> HealthStatus {
        HealthStatus::Degraded {
            reason: self.reason.clone(),
        }
    }
}

/// Always unhealthy check.
pub struct AlwaysUnhealthy {
    reason: String,
}

impl AlwaysUnhealthy {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

#[async_trait]
impl HealthCheck for AlwaysUnhealthy {
    fn name(&self) -> &str {
        "always_unhealthy"
    }

    async fn check(&self) -> HealthStatus {
        HealthStatus::Unhealthy {
            reason: self.reason.clone(),
        }
    }
}

/// Check that always times out.
pub struct AlwaysTimeout {
    delay: Duration,
}

impl AlwaysTimeout {
    pub fn new(delay: Duration) -> Self {
        Self { delay }
    }
}

#[async_trait]
impl HealthCheck for AlwaysTimeout {
    fn name(&self) -> &str {
        "always_timeout"
    }

    async fn check(&self) -> HealthStatus {
        tokio::time::sleep(self.delay).await;
        HealthStatus::Healthy
    }

    fn timeout(&self) -> Duration {
        Duration::from_millis(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_is_healthy() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(!HealthStatus::Degraded { reason: "test".to_string() }.is_healthy());
        assert!(!HealthStatus::Unhealthy { reason: "test".to_string() }.is_healthy());
    }

    #[test]
    fn test_health_status_is_degraded() {
        assert!(!HealthStatus::Healthy.is_degraded());
        assert!(HealthStatus::Degraded { reason: "test".to_string() }.is_degraded());
        assert!(!HealthStatus::Unhealthy { reason: "test".to_string() }.is_degraded());
    }

    #[test]
    fn test_health_status_is_unhealthy() {
        assert!(!HealthStatus::Healthy.is_unhealthy());
        assert!(!HealthStatus::Degraded { reason: "test".to_string() }.is_unhealthy());
        assert!(HealthStatus::Unhealthy { reason: "test".to_string() }.is_unhealthy());
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(format!("{}", HealthStatus::Healthy), "healthy");
        assert_eq!(
            format!("{}", HealthStatus::Degraded { reason: "slow".to_string() }),
            "degraded: slow"
        );
        assert_eq!(
            format!("{}", HealthStatus::Unhealthy { reason: "down".to_string() }),
            "unhealthy: down"
        );
    }

    #[tokio::test]
    async fn test_always_healthy() {
        let check = AlwaysHealthy;
        assert_eq!(check.name(), "always_healthy");
        assert_eq!(check.check().await, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_always_degraded() {
        let check = AlwaysDegraded::new("slow");
        assert_eq!(check.name(), "always_degraded");
        let status = check.check().await;
        assert!(status.is_degraded());
    }

    #[tokio::test]
    async fn test_always_unhealthy() {
        let check = AlwaysUnhealthy::new("down");
        assert_eq!(check.name(), "always_unhealthy");
        let status = check.check().await;
        assert!(status.is_unhealthy());
    }

    #[tokio::test]
    async fn test_always_timeout() {
        let check = AlwaysTimeout::new(Duration::from_millis(100));
        assert_eq!(check.name(), "always_timeout");
        let status = check.check().await;
        assert!(status.is_healthy());
    }

    #[tokio::test]
    async fn test_health_registry_empty() {
        let registry = HealthRegistry::new();
        let report = registry.check_all().await;
        assert_eq!(report.status, HealthStatus::Healthy);
        assert!(report.checks.is_empty());
    }

    #[tokio::test]
    async fn test_health_registry_healthy() {
        let mut registry = HealthRegistry::new();
        registry.register(Arc::new(AlwaysHealthy));
        let report = registry.check_all().await;
        assert_eq!(report.status, HealthStatus::Healthy);
        assert_eq!(report.checks.len(), 1);
        assert_eq!(report.checks[0].status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_registry_degraded() {
        let mut registry = HealthRegistry::new();
        registry.register(Arc::new(AlwaysHealthy));
        registry.register(Arc::new(AlwaysDegraded::new("slow")));
        let report = registry.check_all().await;
        assert!(report.status.is_degraded());
        assert_eq!(report.checks.len(), 2);
    }

    #[tokio::test]
    async fn test_health_registry_unhealthy() {
        let mut registry = HealthRegistry::new();
        registry.register(Arc::new(AlwaysHealthy));
        registry.register(Arc::new(AlwaysUnhealthy::new("down")));
        let report = registry.check_all().await;
        assert!(report.status.is_unhealthy());
        assert_eq!(report.checks.len(), 2);
    }

    #[tokio::test]
    async fn test_health_registry_timeout() {
        let mut registry = HealthRegistry::new();
        registry.register(Arc::new(AlwaysTimeout::new(Duration::from_millis(100))));
        let report = registry.check_all().await;
        assert!(report.status.is_unhealthy());
    }

    #[test]
    fn test_check_result_serialization() {
        let result = CheckResult {
            name: "test".to_string(),
            status: HealthStatus::Healthy,
            duration_ms: 10,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("healthy"));
    }

    #[test]
    fn test_health_report_serialization() {
        let report = HealthReport {
            status: HealthStatus::Healthy,
            checks: vec![],
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("healthy"));
    }
}
