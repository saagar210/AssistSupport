//! Job system for background task management
//!
//! Provides a reliable, observable, and cancelable job queue for long-running
//! operations like content ingestion.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queued => write!(f, "queued"),
            Self::Running => write!(f, "running"),
            Self::Succeeded => write!(f, "succeeded"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for JobStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(()),
        }
    }
}

impl JobStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }
}

/// Job type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    IngestWeb,
    IngestYoutube,
    IngestGithub,
    IngestBatch,
    IndexKb,
    GenerateEmbeddings,
    Custom(String),
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IngestWeb => write!(f, "ingest_web"),
            Self::IngestYoutube => write!(f, "ingest_youtube"),
            Self::IngestGithub => write!(f, "ingest_github"),
            Self::IngestBatch => write!(f, "ingest_batch"),
            Self::IndexKb => write!(f, "index_kb"),
            Self::GenerateEmbeddings => write!(f, "generate_embeddings"),
            Self::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

impl std::str::FromStr for JobType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "ingest_web" => Self::IngestWeb,
            "ingest_youtube" => Self::IngestYoutube,
            "ingest_github" => Self::IngestGithub,
            "ingest_batch" => Self::IngestBatch,
            "index_kb" => Self::IndexKb,
            "generate_embeddings" => Self::GenerateEmbeddings,
            other => {
                if let Some(custom) = other.strip_prefix("custom:") {
                    Self::Custom(custom.to_string())
                } else {
                    Self::Custom(other.to_string())
                }
            }
        })
    }
}

/// Job record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub job_type: JobType,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress: f32,
    pub progress_message: Option<String>,
    pub error: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl Job {
    pub fn new(job_type: JobType) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            job_type,
            status: JobStatus::Queued,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            progress: 0.0,
            progress_message: None,
            error: None,
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Job log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobLog {
    pub id: i64,
    pub job_id: String,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug => write!(f, "debug"),
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Job progress event (for Tauri events)
#[derive(Debug, Clone, Serialize)]
pub struct JobProgressEvent {
    pub job_id: String,
    pub phase: String,
    pub percent: f32,
    pub message: String,
}

/// Job log event (for Tauri events)
#[derive(Debug, Clone, Serialize)]
pub struct JobLogEvent {
    pub job_id: String,
    pub level: LogLevel,
    pub message: String,
}

/// Job done event (for Tauri events)
#[derive(Debug, Clone, Serialize)]
pub struct JobDoneEvent {
    pub job_id: String,
    pub status: JobStatus,
    pub error: Option<String>,
}

/// Cancellation token for jobs
#[derive(Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Job manager for tracking active jobs
pub struct JobManager {
    /// Active cancellation tokens by job ID
    cancellation_tokens: std::sync::Mutex<HashMap<String, CancellationToken>>,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            cancellation_tokens: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Register a job and get its cancellation token
    pub fn register_job(&self, job_id: &str) -> CancellationToken {
        let token = CancellationToken::new();
        let mut tokens = self.cancellation_tokens.lock().unwrap();
        tokens.insert(job_id.to_string(), token.clone());
        token
    }

    /// Cancel a job
    pub fn cancel_job(&self, job_id: &str) -> bool {
        let tokens = self.cancellation_tokens.lock().unwrap();
        if let Some(token) = tokens.get(job_id) {
            token.cancel();
            true
        } else {
            false
        }
    }

    /// Unregister a completed job
    pub fn unregister_job(&self, job_id: &str) {
        let mut tokens = self.cancellation_tokens.lock().unwrap();
        tokens.remove(job_id);
    }

    /// Check if a job is registered
    pub fn is_job_active(&self, job_id: &str) -> bool {
        let tokens = self.cancellation_tokens.lock().unwrap();
        tokens.contains_key(job_id)
    }
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Job runner context for executing jobs with event emission
pub struct JobContext {
    pub job_id: String,
    pub cancel_token: CancellationToken,
}

impl JobContext {
    pub fn new(job_id: String, cancel_token: CancellationToken) -> Self {
        Self { job_id, cancel_token }
    }

    /// Check if the job has been cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// Create a progress event
    pub fn progress_event(&self, phase: &str, percent: f32, message: &str) -> JobProgressEvent {
        JobProgressEvent {
            job_id: self.job_id.clone(),
            phase: phase.to_string(),
            percent,
            message: message.to_string(),
        }
    }

    /// Create a log event
    pub fn log_event(&self, level: LogLevel, message: &str) -> JobLogEvent {
        JobLogEvent {
            job_id: self.job_id.clone(),
            level,
            message: message.to_string(),
        }
    }

    /// Create a done event
    pub fn done_event(&self, status: JobStatus, error: Option<String>) -> JobDoneEvent {
        JobDoneEvent {
            job_id: self.job_id.clone(),
            status,
            error,
        }
    }
}

/// Event names for Tauri
pub mod events {
    pub const JOB_PROGRESS: &str = "job:progress";
    pub const JOB_LOG: &str = "job:log";
    pub const JOB_DONE: &str = "job:done";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_display() {
        assert_eq!(JobStatus::Queued.to_string(), "queued");
        assert_eq!(JobStatus::Running.to_string(), "running");
        assert_eq!(JobStatus::Succeeded.to_string(), "succeeded");
        assert_eq!(JobStatus::Failed.to_string(), "failed");
        assert_eq!(JobStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_job_status_terminal() {
        assert!(!JobStatus::Queued.is_terminal());
        assert!(!JobStatus::Running.is_terminal());
        assert!(JobStatus::Succeeded.is_terminal());
        assert!(JobStatus::Failed.is_terminal());
        assert!(JobStatus::Cancelled.is_terminal());
    }

    #[test]
    fn test_job_type_display() {
        assert_eq!(JobType::IngestWeb.to_string(), "ingest_web");
        assert_eq!(JobType::Custom("test".to_string()).to_string(), "custom:test");
    }

    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_job_manager() {
        let manager = JobManager::new();
        let token = manager.register_job("job-1");

        assert!(manager.is_job_active("job-1"));
        assert!(!token.is_cancelled());

        manager.cancel_job("job-1");
        assert!(token.is_cancelled());

        manager.unregister_job("job-1");
        assert!(!manager.is_job_active("job-1"));
    }
}
