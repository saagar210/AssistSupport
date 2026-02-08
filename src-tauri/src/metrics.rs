use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::Instant;

use serde::{Deserialize, Serialize};

/// Application-level metrics using atomic counters for zero-overhead tracking.
pub struct AppMetrics {
    pub documents_ingested: AtomicU64,
    pub chunks_created: AtomicU64,
    pub searches_executed: AtomicU64,
    pub chat_messages_sent: AtomicU64,
    pub embedding_api_calls: AtomicU64,
    pub embedding_api_errors: AtomicU64,
    pub avg_search_latency_ms: RwLock<f64>,
    pub avg_ingestion_time_ms: RwLock<f64>,
    pub vector_index_size: AtomicU64,
    pub uptime_start: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub documents_ingested: u64,
    pub chunks_created: u64,
    pub searches_executed: u64,
    pub chat_messages_sent: u64,
    pub embedding_api_calls: u64,
    pub embedding_api_errors: u64,
    pub avg_search_latency_ms: f64,
    pub avg_ingestion_time_ms: f64,
    pub vector_index_size: u64,
    pub uptime_seconds: u64,
}

#[allow(dead_code)]
pub enum MetricCounter {
    DocumentsIngested,
    ChunksCreated,
    SearchesExecuted,
    ChatMessagesSent,
    EmbeddingApiCalls,
    EmbeddingApiErrors,
}

#[allow(dead_code)]
pub enum LatencyMetric {
    SearchLatency,
    IngestionTime,
}

impl AppMetrics {
    pub fn new() -> Self {
        AppMetrics {
            documents_ingested: AtomicU64::new(0),
            chunks_created: AtomicU64::new(0),
            searches_executed: AtomicU64::new(0),
            chat_messages_sent: AtomicU64::new(0),
            embedding_api_calls: AtomicU64::new(0),
            embedding_api_errors: AtomicU64::new(0),
            avg_search_latency_ms: RwLock::new(0.0),
            avg_ingestion_time_ms: RwLock::new(0.0),
            vector_index_size: AtomicU64::new(0),
            uptime_start: Instant::now(),
        }
    }

    pub fn increment(&self, counter: MetricCounter) {
        match counter {
            MetricCounter::DocumentsIngested => {
                self.documents_ingested.fetch_add(1, Ordering::Relaxed);
            }
            MetricCounter::ChunksCreated => {
                self.chunks_created.fetch_add(1, Ordering::Relaxed);
            }
            MetricCounter::SearchesExecuted => {
                self.searches_executed.fetch_add(1, Ordering::Relaxed);
            }
            MetricCounter::ChatMessagesSent => {
                self.chat_messages_sent.fetch_add(1, Ordering::Relaxed);
            }
            MetricCounter::EmbeddingApiCalls => {
                self.embedding_api_calls.fetch_add(1, Ordering::Relaxed);
            }
            MetricCounter::EmbeddingApiErrors => {
                self.embedding_api_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn increment_by(&self, counter: MetricCounter, count: u64) {
        match counter {
            MetricCounter::ChunksCreated => {
                self.chunks_created.fetch_add(count, Ordering::Relaxed);
            }
            _ => { /* Only ChunksCreated supports batch increment */ }
        }
    }

    /// Record a latency observation using exponential moving average.
    pub fn record_latency(&self, metric: LatencyMetric, duration_ms: f64) {
        let lock = match metric {
            LatencyMetric::SearchLatency => &self.avg_search_latency_ms,
            LatencyMetric::IngestionTime => &self.avg_ingestion_time_ms,
        };
        if let Ok(mut avg) = lock.write() {
            if *avg == 0.0 {
                *avg = duration_ms;
            } else {
                // EMA with alpha = 0.2
                *avg = *avg * 0.8 + duration_ms * 0.2;
            }
        }
    }

    #[allow(dead_code)]
    pub fn set_vector_index_size(&self, size: u64) {
        self.vector_index_size.store(size, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            documents_ingested: self.documents_ingested.load(Ordering::Relaxed),
            chunks_created: self.chunks_created.load(Ordering::Relaxed),
            searches_executed: self.searches_executed.load(Ordering::Relaxed),
            chat_messages_sent: self.chat_messages_sent.load(Ordering::Relaxed),
            embedding_api_calls: self.embedding_api_calls.load(Ordering::Relaxed),
            embedding_api_errors: self.embedding_api_errors.load(Ordering::Relaxed),
            avg_search_latency_ms: self.avg_search_latency_ms.read().map(|v| *v).unwrap_or(0.0),
            avg_ingestion_time_ms: self.avg_ingestion_time_ms.read().map(|v| *v).unwrap_or(0.0),
            vector_index_size: self.vector_index_size.load(Ordering::Relaxed),
            uptime_seconds: self.uptime_start.elapsed().as_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment_counters() {
        let metrics = AppMetrics::new();
        metrics.increment(MetricCounter::DocumentsIngested);
        metrics.increment(MetricCounter::DocumentsIngested);
        metrics.increment(MetricCounter::SearchesExecuted);

        let snap = metrics.snapshot();
        assert_eq!(snap.documents_ingested, 2);
        assert_eq!(snap.searches_executed, 1);
        assert_eq!(snap.chat_messages_sent, 0);
    }

    #[test]
    fn test_increment_by() {
        let metrics = AppMetrics::new();
        metrics.increment_by(MetricCounter::ChunksCreated, 15);
        let snap = metrics.snapshot();
        assert_eq!(snap.chunks_created, 15);
    }

    #[test]
    fn test_record_latency_ema() {
        let metrics = AppMetrics::new();
        metrics.record_latency(LatencyMetric::SearchLatency, 100.0);
        let snap1 = metrics.snapshot();
        assert!((snap1.avg_search_latency_ms - 100.0).abs() < 0.01);

        metrics.record_latency(LatencyMetric::SearchLatency, 200.0);
        let snap2 = metrics.snapshot();
        // EMA: 100 * 0.8 + 200 * 0.2 = 120
        assert!((snap2.avg_search_latency_ms - 120.0).abs() < 0.01);
    }

    #[test]
    fn test_uptime_tracking() {
        let metrics = AppMetrics::new();
        let snap = metrics.snapshot();
        assert!(snap.uptime_seconds < 2); // Just created, should be near 0
    }

    #[test]
    fn test_set_vector_index_size() {
        let metrics = AppMetrics::new();
        metrics.set_vector_index_size(42000);
        let snap = metrics.snapshot();
        assert_eq!(snap.vector_index_size, 42000);
    }
}
