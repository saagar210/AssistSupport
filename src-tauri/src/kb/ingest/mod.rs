//! Content ingestion modules for AssistSupport
//! Handles web pages, YouTube videos, GitHub repos, and batch processing

pub mod batch;
pub mod github;
pub mod web;
pub mod youtube;

use thiserror::Error;

/// Common ingestion error type
#[derive(Debug, Error)]
pub enum IngestError {
    #[error("Network error: {0}")]
    Network(#[from] crate::kb::network::NetworkError),
    #[error("Database error: {0}")]
    Database(#[from] crate::db::DbError),
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Parsing error: {0}")]
    Parse(String),
    #[error("Content too large: {size} bytes (max: {max} bytes)")]
    ContentTooLarge { size: usize, max: usize },
    #[error("Content not found: {0}")]
    NotFound(String),
    #[error("Authentication required: {0}")]
    AuthRequired(String),
    #[error("Rate limited: {0}")]
    RateLimited(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("Cancelled by user")]
    Cancelled,
    #[error("Invalid source: {0}")]
    InvalidSource(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for ingestion operations
pub type IngestResult<T> = Result<T, IngestError>;

/// Progress callback for long-running ingestion operations
pub type ProgressCallback = Box<dyn Fn(IngestProgress) + Send + Sync>;

/// Progress information for ingestion
#[derive(Debug, Clone)]
pub struct IngestProgress {
    pub phase: IngestPhase,
    pub current: usize,
    pub total: Option<usize>,
    pub message: String,
}

/// Phases of ingestion
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IngestPhase {
    Fetching,
    Parsing,
    Chunking,
    Indexing,
    Vectorizing,
    Complete,
    Failed,
}

/// Result of a single document ingestion
#[derive(Debug, Clone)]
pub struct IngestedDocument {
    pub id: String,
    pub title: String,
    pub source_uri: String,
    pub chunk_count: usize,
    pub word_count: usize,
}

/// Cancellation token for long-running operations
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::SeqCst)
    }
}
