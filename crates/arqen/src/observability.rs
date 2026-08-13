//! Observability module for Arqen.
//!
//! Provides request metrics, latency histograms, and monitoring.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetric {
    pub method: String,
    pub route: String,
    pub status: u16,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetric {
    pub operation: String,
    pub backend: String,
    pub duration_ms: u64,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetric {
    pub operation: String,
    pub hit: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetric {
    pub queue: String,
    pub operation: String,
    pub duration_ms: u64,
    pub success: bool,
}

/// A replication operation metric. The event contains operational counters,
/// never credentials, request bodies, or replicated payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMetric {
    pub operation: String,
    pub mode: String,
    pub duration_ms: u64,
    pub retries: u32,
    pub cursor: u64,
    pub conflicts: u64,
    pub snapshot_fallback: bool,
    pub success: bool,
}

/// Vendor-neutral metrics integration point. Implementations may forward
/// these events to Prometheus, OpenTelemetry, or an application-owned system.
pub trait MetricsSink: Send + Sync {
    fn record_request(&self, event: RequestMetric);
    fn record_storage(&self, event: StorageMetric);
    fn record_cache(&self, event: CacheMetric);
    fn record_job(&self, event: JobMetric);

    /// Record a Thingd replication operation.
    fn record_sync(&self, _event: SyncMetric) {}
}

#[derive(Debug, Default)]
pub struct NoopMetricsSink;

impl MetricsSink for NoopMetricsSink {
    fn record_request(&self, _event: RequestMetric) {}
    fn record_storage(&self, _event: StorageMetric) {}
    fn record_cache(&self, _event: CacheMetric) {}
    fn record_job(&self, _event: JobMetric) {}
}

pub type SharedMetricsSink = Arc<dyn MetricsSink>;

/// Request metrics for monitoring.
pub struct RequestMetrics {
    inner: Mutex<MetricsInner>,
    requests_total: AtomicU64,
    requests_success: AtomicU64,
    requests_client_error: AtomicU64,
    requests_server_error: AtomicU64,
    requests_timeout: AtomicU64,
    requests_dependency_error: AtomicU64,
}

struct MetricsInner {
    durations: VecDeque<u64>,
    response_bytes: VecDeque<u64>,
    by_path: HashMap<String, u64>,
    by_method: HashMap<String, u64>,
    by_status: HashMap<u16, u64>,
    start_time: Instant,
}

const MAX_LATENCY_SAMPLES: usize = 4096;
const MAX_ROUTE_LABELS: usize = 256;

impl RequestMetrics {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(MetricsInner {
                durations: VecDeque::with_capacity(MAX_LATENCY_SAMPLES),
                response_bytes: VecDeque::with_capacity(MAX_LATENCY_SAMPLES),
                by_path: HashMap::new(),
                by_method: HashMap::new(),
                by_status: HashMap::new(),
                start_time: Instant::now(),
            }),
            requests_total: AtomicU64::new(0),
            requests_success: AtomicU64::new(0),
            requests_client_error: AtomicU64::new(0),
            requests_server_error: AtomicU64::new(0),
            requests_timeout: AtomicU64::new(0),
            requests_dependency_error: AtomicU64::new(0),
        }
    }

    pub fn record(&self, method: &str, path: &str, status: u16, duration_ms: u64) {
        self.record_with_size(method, path, status, duration_ms, 0);
    }

    pub fn record_with_size(
        &self,
        method: &str,
        path: &str,
        status: u16,
        duration_ms: u64,
        response_bytes: u64,
    ) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        match status {
            200..=299 => self.requests_success.fetch_add(1, Ordering::Relaxed),
            400..=499 => self.requests_client_error.fetch_add(1, Ordering::Relaxed),
            500..=599 => self.requests_server_error.fetch_add(1, Ordering::Relaxed),
            _ => 0,
        };
        if matches!(status, 408 | 504) {
            self.requests_timeout.fetch_add(1, Ordering::Relaxed);
        }
        if status == 502 {
            self.requests_dependency_error
                .fetch_add(1, Ordering::Relaxed);
        }
        let Ok(mut inner) = self.inner.lock() else {
            return;
        };
        if inner.durations.len() == MAX_LATENCY_SAMPLES {
            inner.durations.pop_front();
            inner.response_bytes.pop_front();
        }
        inner.durations.push_back(duration_ms);
        inner.response_bytes.push_back(response_bytes);
        let route = if inner.by_path.contains_key(path) || inner.by_path.len() < MAX_ROUTE_LABELS {
            path.to_string()
        } else {
            "__other__".to_string()
        };
        *inner.by_path.entry(route).or_insert(0) += 1;
        *inner.by_method.entry(method.to_string()).or_insert(0) += 1;
        *inner.by_status.entry(status).or_insert(0) += 1;
    }

    pub fn total_requests(&self) -> u64 {
        self.requests_total.load(Ordering::Relaxed)
    }

    pub fn avg_duration_ms(&self) -> f64 {
        let Ok(inner) = self.inner.lock() else {
            return 0.0;
        };
        if inner.durations.is_empty() {
            return 0.0;
        }
        let sum: u64 = inner.durations.iter().sum();
        sum as f64 / inner.durations.len() as f64
    }

    /// Get uptime in seconds.
    pub fn uptime_seconds(&self) -> u64 {
        self.inner
            .lock()
            .map(|i| i.start_time.elapsed().as_secs())
            .unwrap_or(0)
    }

    /// Get error rate (5xx / total).
    pub fn error_rate(&self) -> f64 {
        let total = self.requests_total.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        self.requests_server_error.load(Ordering::Relaxed) as f64 / total as f64
    }

    pub fn to_report(&self) -> MetricsReport {
        let Ok(inner) = self.inner.lock() else {
            return MetricsReport::default();
        };
        let mut sorted: Vec<u64> = inner.durations.iter().copied().collect();
        sorted.sort_unstable();
        let len = sorted.len();
        MetricsReport {
            requests_total: self.requests_total.load(Ordering::Relaxed),
            requests_success: self.requests_success.load(Ordering::Relaxed),
            requests_client_error: self.requests_client_error.load(Ordering::Relaxed),
            requests_server_error: self.requests_server_error.load(Ordering::Relaxed),
            requests_timeout: self.requests_timeout.load(Ordering::Relaxed),
            requests_dependency_error: self.requests_dependency_error.load(Ordering::Relaxed),
            avg_duration_ms: if len > 0 {
                sorted.iter().sum::<u64>() as f64 / len as f64
            } else {
                0.0
            },
            p50_duration_ms: percentile(&sorted, 0.50),
            p95_duration_ms: percentile(&sorted, 0.95),
            p99_duration_ms: percentile(&sorted, 0.99),
            max_duration_ms: sorted.last().copied().unwrap_or(0),
            by_path: inner.by_path.clone(),
            by_method: inner.by_method.clone(),
            by_status: inner.by_status.clone(),
            avg_response_bytes: if inner.response_bytes.is_empty() {
                0.0
            } else {
                inner.response_bytes.iter().sum::<u64>() as f64 / inner.response_bytes.len() as f64
            },
            uptime_seconds: inner.start_time.elapsed().as_secs(),
        }
    }
}

fn percentile(sorted: &[u64], p: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() - 1) as f64 * p) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

impl Default for RequestMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricsReport {
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_client_error: u64,
    pub requests_server_error: u64,
    pub requests_timeout: u64,
    pub requests_dependency_error: u64,
    pub avg_duration_ms: f64,
    pub p50_duration_ms: u64,
    pub p95_duration_ms: u64,
    pub p99_duration_ms: u64,
    pub max_duration_ms: u64,
    pub by_path: HashMap<String, u64>,
    pub by_method: HashMap<String, u64>,
    pub by_status: HashMap<u16, u64>,
    pub avg_response_bytes: f64,
    pub uptime_seconds: u64,
}

pub struct RequestTimer {
    start: Instant,
    method: String,
    path: String,
}

impl RequestTimer {
    pub fn start(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            method: method.into(),
            path: path.into(),
        }
    }

    pub fn finish(self, metrics: &RequestMetrics, status: u16) {
        let duration_ms = self.start.elapsed().as_millis() as u64;
        metrics.record(&self.method, &self.path, status, duration_ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = RequestMetrics::new();
        assert_eq!(m.total_requests(), 0);
    }

    #[test]
    fn test_record() {
        let m = RequestMetrics::new();
        m.record("GET", "/health", 200, 10);
        assert_eq!(m.total_requests(), 1);
    }

    #[test]
    fn test_response_size_is_reported() {
        let metrics = RequestMetrics::new();
        metrics.record_with_size("GET", "/items", 200, 4, 1024);
        metrics.record_with_size("GET", "/items", 200, 6, 2048);
        assert_eq!(metrics.to_report().avg_response_bytes, 1536.0);
    }

    #[test]
    fn test_latency_samples_and_route_labels_are_bounded() {
        let m = RequestMetrics::new();
        for index in 0..5000 {
            m.record("GET", &format!("/title/{index}"), 200, index);
        }

        let report = m.to_report();
        assert_eq!(report.requests_total, 5000);
        assert!(report.by_path.len() <= MAX_ROUTE_LABELS + 1);
        assert!(report.max_duration_ms >= 4999 - MAX_LATENCY_SAMPLES as u64);
    }

    #[test]
    fn test_timeout_and_dependency_counters() {
        let m = RequestMetrics::new();
        m.record("GET", "/timeout", 504, 10);
        m.record("GET", "/dependency", 502, 11);

        let report = m.to_report();
        assert_eq!(report.requests_timeout, 1);
        assert_eq!(report.requests_dependency_error, 1);
    }

    #[test]
    fn test_record_multiple() {
        let m = RequestMetrics::new();
        m.record("GET", "/health", 200, 10);
        m.record("POST", "/agent", 201, 50);
        m.record("GET", "/health", 200, 15);
        assert_eq!(m.total_requests(), 3);
    }

    #[test]
    fn test_status_codes() {
        let m = RequestMetrics::new();
        m.record("GET", "/health", 200, 10);
        m.record("GET", "/error", 404, 5);
        m.record("GET", "/fail", 500, 100);
        let r = m.to_report();
        assert_eq!(r.requests_success, 1);
        assert_eq!(r.requests_client_error, 1);
        assert_eq!(r.requests_server_error, 1);
    }

    #[test]
    fn test_avg_duration() {
        let m = RequestMetrics::new();
        m.record("GET", "/health", 200, 10);
        m.record("GET", "/health", 200, 20);
        assert_eq!(m.avg_duration_ms(), 15.0);
    }

    #[test]
    fn test_avg_duration_empty() {
        let m = RequestMetrics::new();
        assert_eq!(m.avg_duration_ms(), 0.0);
    }

    #[test]
    fn test_error_rate() {
        let m = RequestMetrics::new();
        m.record("GET", "/ok", 200, 10);
        m.record("GET", "/ok", 200, 10);
        m.record("GET", "/fail", 500, 100);
        let rate = m.error_rate();
        assert!((rate - 1.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_error_rate_empty() {
        let m = RequestMetrics::new();
        assert_eq!(m.error_rate(), 0.0);
    }

    #[test]
    fn test_uptime() {
        let m = RequestMetrics::new();
        let uptime = m.uptime_seconds();
        assert!(uptime < 2);
    }

    #[test]
    fn test_to_report() {
        let m = RequestMetrics::new();
        m.record("GET", "/health", 200, 10);
        let r = m.to_report();
        assert_eq!(r.requests_total, 1);
        assert_eq!(r.requests_success, 1);
        assert_eq!(r.p50_duration_ms, 10);
        assert_eq!(r.p95_duration_ms, 10);
        assert_eq!(r.p99_duration_ms, 10);
        assert_eq!(r.max_duration_ms, 10);
        assert!(r.uptime_seconds < 2);
    }

    #[test]
    fn test_percentiles() {
        let m = RequestMetrics::new();
        for i in 1..=100 {
            m.record("GET", "/test", 200, i);
        }
        let r = m.to_report();
        assert_eq!(r.p50_duration_ms, 50);
        assert_eq!(r.p95_duration_ms, 95);
        assert_eq!(r.p99_duration_ms, 99);
        assert_eq!(r.max_duration_ms, 100);
    }

    #[test]
    fn test_by_status() {
        let m = RequestMetrics::new();
        m.record("GET", "/ok", 200, 10);
        m.record("GET", "/ok", 200, 10);
        m.record("GET", "/err", 500, 100);
        let r = m.to_report();
        assert_eq!(r.by_status.get(&200), Some(&2));
        assert_eq!(r.by_status.get(&500), Some(&1));
    }

    #[test]
    fn test_timer() {
        let m = RequestMetrics::new();
        let t = RequestTimer::start("GET", "/health");
        t.finish(&m, 200);
        assert_eq!(m.total_requests(), 1);
    }
}
