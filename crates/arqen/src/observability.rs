//! Observability module for Arqen.
//!
//! Provides request metrics and monitoring.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use serde::{Deserialize, Serialize};

/// Request metrics for monitoring.
pub struct RequestMetrics {
    inner: Mutex<MetricsInner>,
}

struct MetricsInner {
    requests_total: u64,
    requests_success: u64,
    requests_client_error: u64,
    requests_server_error: u64,
    durations: Vec<u64>,
    by_path: HashMap<String, u64>,
    by_method: HashMap<String, u64>,
}

impl RequestMetrics {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(MetricsInner {
                requests_total: 0,
                requests_success: 0,
                requests_client_error: 0,
                requests_server_error: 0,
                durations: Vec::new(),
                by_path: HashMap::new(),
                by_method: HashMap::new(),
            }),
        }
    }

    pub fn record(&self, method: &str, path: &str, status: u16, duration_ms: u64) {
        let Ok(mut inner) = self.inner.lock() else {
            return;
        };
        inner.requests_total += 1;
        match status {
            200..=299 => inner.requests_success += 1,
            400..=499 => inner.requests_client_error += 1,
            500..=599 => inner.requests_server_error += 1,
            _ => {}
        }
        inner.durations.push(duration_ms);
        *inner.by_path.entry(path.to_string()).or_insert(0) += 1;
        *inner.by_method.entry(method.to_string()).or_insert(0) += 1;
    }

    pub fn total_requests(&self) -> u64 {
        self.inner.lock().map(|i| i.requests_total).unwrap_or(0)
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

    pub fn to_report(&self) -> MetricsReport {
        let Ok(inner) = self.inner.lock() else {
            return MetricsReport::default();
        };
        let mut sorted = inner.durations.clone();
        sorted.sort_unstable();
        let len = sorted.len();
        MetricsReport {
            requests_total: inner.requests_total,
            requests_success: inner.requests_success,
            requests_client_error: inner.requests_client_error,
            requests_server_error: inner.requests_server_error,
            avg_duration_ms: if len > 0 {
                sorted.iter().sum::<u64>() as f64 / len as f64
            } else {
                0.0
            },
            p95_duration_ms: if len > 0 {
                sorted[(len as f64 * 0.95) as usize % len]
            } else {
                0
            },
            p99_duration_ms: if len > 0 {
                sorted[(len as f64 * 0.99) as usize % len]
            } else {
                0
            },
            by_path: inner.by_path.clone(),
            by_method: inner.by_method.clone(),
        }
    }
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
    pub avg_duration_ms: f64,
    pub p95_duration_ms: u64,
    pub p99_duration_ms: u64,
    pub by_path: HashMap<String, u64>,
    pub by_method: HashMap<String, u64>,
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
    fn test_to_report() {
        let m = RequestMetrics::new();
        m.record("GET", "/health", 200, 10);
        let r = m.to_report();
        assert_eq!(r.requests_total, 1);
        assert_eq!(r.requests_success, 1);
    }

    #[test]
    fn test_timer() {
        let m = RequestMetrics::new();
        let t = RequestTimer::start("GET", "/health");
        t.finish(&m, 200);
        assert_eq!(m.total_requests(), 1);
    }
}
