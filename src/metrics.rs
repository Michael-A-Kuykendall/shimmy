// Basic metrics collection
#![allow(dead_code)]

use axum::response::Json;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Serialize)]
pub struct Metrics {
    pub requests_total: u64,
    pub generation_errors: u64,
    pub uptime_seconds: u64,
}

pub struct MetricsCollector {
    pub requests: AtomicU64,
    pub errors: AtomicU64,
    pub start_time: std::time::Instant,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new_inner()
    }
}

impl MetricsCollector {
    fn new_inner() -> Self {
        Self {
            requests: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn new() -> Arc<Self> {
        Arc::new(Self::new_inner())
    }

    pub fn record_request(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_metrics(&self) -> Metrics {
        Metrics {
            requests_total: self.requests.load(Ordering::Relaxed),
            generation_errors: self.errors.load(Ordering::Relaxed),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }
}

pub async fn metrics_handler(metrics: Arc<MetricsCollector>) -> Json<Metrics> {
    Json(metrics.get_metrics())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_new() {
        let metrics = MetricsCollector::new();
        assert_eq!(metrics.requests.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.errors.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_record_request() {
        let metrics = MetricsCollector::new();
        metrics.record_request();
        assert_eq!(metrics.requests.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_record_error() {
        let metrics = MetricsCollector::new();
        metrics.record_error();
        assert_eq!(metrics.errors.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_get_metrics() {
        let metrics = MetricsCollector::new();
        metrics.record_request();
        metrics.record_error();

        let result = metrics.get_metrics();
        assert_eq!(result.requests_total, 1);
        assert_eq!(result.generation_errors, 1);
        assert!(result.uptime_seconds < 60);
    }
}
