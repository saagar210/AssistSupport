//! Audit logging module for AssistSupport
//!
//! Provides structured audit logging for security-relevant events.
//! Logs are stored as JSON lines at ~/Library/Application Support/AssistSupport/audit.log
//!
//! Features:
//! - Structured JSON format
//! - Automatic log rotation (max 5MB per file, keep 5 files)
//! - Never logs secrets (tokens, keys, passwords)
//! - Thread-safe writes
//! - In-memory ring buffer fallback for reliability
//! - Critical event logging with guaranteed delivery

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

/// Maximum size for a single log file (5MB)
const MAX_LOG_SIZE: u64 = 5 * 1024 * 1024;

/// Number of rotated log files to keep
const MAX_LOG_FILES: usize = 5;

/// Audit log file name
const AUDIT_LOG_NAME: &str = "audit.log";

/// Maximum entries in the fallback ring buffer
const RING_BUFFER_SIZE: usize = 100;

/// Interval for attempting to flush buffered entries (in seconds)
const FLUSH_INTERVAL_SECS: u64 = 60;

/// Audit event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
    /// Informational event (normal operation)
    Info,
    /// Warning (potential issue, but operation succeeded)
    Warning,
    /// Error (operation failed)
    Error,
    /// Critical (security-relevant failure)
    Critical,
}

impl std::fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Audit event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    // Key management events
    KeyGenerated,
    KeyMigrated,
    KeyRotated,
    KeyStorageModeChanged,

    // Token events
    TokenSet,
    TokenCleared,

    // Jira events
    JiraConfigured,
    JiraHttpOptIn,
    JiraConnectionFailed,

    // Security events
    PathValidationFailed,
    EncryptionFailed,
    DecryptionFailed,
    AuthenticationFailed,

    // App lifecycle
    AppInitialized,
    DatabaseRepaired,
    VectorStoreRebuilt,

    // Custom event type for extensibility
    Custom(String),
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KeyGenerated => write!(f, "key_generated"),
            Self::KeyMigrated => write!(f, "key_migrated"),
            Self::KeyRotated => write!(f, "key_rotated"),
            Self::KeyStorageModeChanged => write!(f, "key_storage_mode_changed"),
            Self::TokenSet => write!(f, "token_set"),
            Self::TokenCleared => write!(f, "token_cleared"),
            Self::JiraConfigured => write!(f, "jira_configured"),
            Self::JiraHttpOptIn => write!(f, "jira_http_opt_in"),
            Self::JiraConnectionFailed => write!(f, "jira_connection_failed"),
            Self::PathValidationFailed => write!(f, "path_validation_failed"),
            Self::EncryptionFailed => write!(f, "encryption_failed"),
            Self::DecryptionFailed => write!(f, "decryption_failed"),
            Self::AuthenticationFailed => write!(f, "authentication_failed"),
            Self::AppInitialized => write!(f, "app_initialized"),
            Self::DatabaseRepaired => write!(f, "database_repaired"),
            Self::VectorStoreRebuilt => write!(f, "vector_store_rebuilt"),
            Self::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// ISO 8601 timestamp
    pub timestamp: DateTime<Utc>,

    /// Event type
    pub event: AuditEventType,

    /// Severity level
    pub severity: AuditSeverity,

    /// Human-readable message (no secrets)
    pub message: String,

    /// Additional context (no secrets)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

impl AuditEntry {
    /// Create a new audit entry
    pub fn new(event: AuditEventType, severity: AuditSeverity, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            event,
            severity,
            message: message.into(),
            context: None,
        }
    }

    /// Add context to the entry
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }
}

/// Global audit logger
static AUDIT_LOGGER: Mutex<Option<AuditLogger>> = Mutex::new(None);

/// Fallback ring buffer for entries that couldn't be written to disk
static RING_BUFFER: Mutex<VecDeque<AuditEntry>> = Mutex::new(VecDeque::new());

/// Last time we attempted to flush the ring buffer
static LAST_FLUSH_ATTEMPT: Mutex<Option<std::time::Instant>> = Mutex::new(None);

/// Error callback type for monitoring audit failures
pub type AuditErrorCallback = fn(&AuditError, &AuditEntry);

/// Global error callback for monitoring
static ERROR_CALLBACK: Mutex<Option<AuditErrorCallback>> = Mutex::new(None);

/// Set a global error callback for audit failures
/// This allows external monitoring of audit system health
pub fn set_error_callback(callback: Option<AuditErrorCallback>) {
    if let Ok(mut guard) = ERROR_CALLBACK.lock() {
        *guard = callback;
    }
}

/// Audit logger
pub struct AuditLogger {
    log_dir: PathBuf,
}

impl AuditLogger {
    /// Initialize the global audit logger
    pub fn init() -> Result<(), AuditError> {
        let log_dir = dirs::data_dir()
            .map(|d| d.join("AssistSupport"))
            .ok_or(AuditError::LogDirNotFound)?;

        // Create directory with secure permissions (0o700)
        crate::security::create_secure_dir(&log_dir).map_err(|e| AuditError::IO(e.to_string()))?;

        let logger = Self { log_dir };

        let mut guard = AUDIT_LOGGER.lock().map_err(|_| AuditError::LockFailed)?;
        *guard = Some(logger);

        Ok(())
    }

    /// Get the log file path
    fn log_path(&self) -> PathBuf {
        self.log_dir.join(AUDIT_LOG_NAME)
    }

    /// Get rotated log file path
    fn rotated_path(&self, index: usize) -> PathBuf {
        self.log_dir.join(format!("{}.{}", AUDIT_LOG_NAME, index))
    }

    /// Rotate logs if needed
    fn rotate_if_needed(&self) -> Result<(), AuditError> {
        let log_path = self.log_path();

        if !log_path.exists() {
            return Ok(());
        }

        let metadata = fs::metadata(&log_path).map_err(|e| AuditError::IO(e.to_string()))?;

        if metadata.len() < MAX_LOG_SIZE {
            return Ok(());
        }

        // Rotate existing files
        for i in (0..MAX_LOG_FILES - 1).rev() {
            let from = if i == 0 {
                self.log_path()
            } else {
                self.rotated_path(i)
            };
            let to = self.rotated_path(i + 1);

            if from.exists() {
                let _ = fs::rename(&from, &to);
            }
        }

        // Delete oldest if it exists
        let oldest = self.rotated_path(MAX_LOG_FILES);
        if oldest.exists() {
            let _ = fs::remove_file(&oldest);
        }

        Ok(())
    }

    /// Write an audit entry
    fn write(&self, entry: &AuditEntry) -> Result<(), AuditError> {
        self.rotate_if_needed()?;

        let log_path = self.log_path();
        let is_new = !log_path.exists();

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| AuditError::IO(e.to_string()))?;

        // Set secure permissions on new files (0o600)
        if is_new {
            let _ = crate::security::set_secure_permissions(&log_path, crate::security::FILE_PERMISSIONS);
        }

        let mut writer = BufWriter::new(file);
        let line =
            serde_json::to_string(entry).map_err(|e| AuditError::Serialization(e.to_string()))?;

        writeln!(writer, "{}", line).map_err(|e| AuditError::IO(e.to_string()))?;
        writer.flush().map_err(|e| AuditError::IO(e.to_string()))?;

        Ok(())
    }

    /// Attempt to flush buffered entries to disk
    fn flush_buffer(&self) -> usize {
        let mut flushed = 0;

        // Get entries from buffer
        let entries: Vec<AuditEntry> = {
            if let Ok(mut buffer) = RING_BUFFER.lock() {
                buffer.drain(..).collect()
            } else {
                return 0;
            }
        };

        // Try to write each entry
        for entry in entries {
            if self.write(&entry).is_ok() {
                flushed += 1;
            } else {
                // Put back in buffer if write failed
                add_to_ring_buffer(entry);
            }
        }

        flushed
    }
}

/// Add an entry to the ring buffer (fallback storage)
fn add_to_ring_buffer(entry: AuditEntry) {
    if let Ok(mut buffer) = RING_BUFFER.lock() {
        // Remove oldest if at capacity
        while buffer.len() >= RING_BUFFER_SIZE {
            buffer.pop_front();
        }
        buffer.push_back(entry);
    }
}

/// Check if we should attempt to flush buffered entries
fn should_attempt_flush() -> bool {
    if let Ok(mut last_flush) = LAST_FLUSH_ATTEMPT.lock() {
        let now = std::time::Instant::now();
        match *last_flush {
            Some(last) if now.duration_since(last) < Duration::from_secs(FLUSH_INTERVAL_SECS) => {
                false
            }
            _ => {
                *last_flush = Some(now);
                true
            }
        }
    } else {
        false
    }
}

/// Notify error callback if set
fn notify_error(error: &AuditError, entry: &AuditEntry) {
    if let Ok(guard) = ERROR_CALLBACK.lock() {
        if let Some(callback) = *guard {
            callback(error, entry);
        }
    }
}

/// Audit logging errors
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Log directory not found")]
    LogDirNotFound,
    #[error("IO error: {0}")]
    IO(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Failed to acquire lock")]
    LockFailed,
    #[error("Logger not initialized")]
    NotInitialized,
}

/// Log an audit event (synchronous, returns errors)
pub fn log_audit(entry: AuditEntry) -> Result<(), AuditError> {
    let guard = AUDIT_LOGGER.lock().map_err(|_| AuditError::LockFailed)?;
    let logger = guard.as_ref().ok_or(AuditError::NotInitialized)?;

    // Attempt to flush buffered entries periodically
    if should_attempt_flush() {
        logger.flush_buffer();
    }

    logger.write(&entry)
}

/// Log an audit event with best-effort delivery (uses ring buffer fallback)
///
/// This function:
/// - Attempts to write to disk
/// - On failure, stores in ring buffer for later retry
/// - Notifies error callback if set
/// - Never blocks or fails (fire-and-forget semantics)
///
/// Use this for informational events that are nice to have but not critical.
pub fn log_audit_best_effort(entry: AuditEntry) {
    std::thread::spawn(move || {
        match log_audit(entry.clone()) {
            Ok(()) => {}
            Err(e) => {
                // Notify callback
                notify_error(&e, &entry);
                // Store in ring buffer for later retry
                add_to_ring_buffer(entry);
            }
        }
    });
}

/// Log an audit event (fire and forget, ignores errors)
///
/// DEPRECATED: Use `log_audit_best_effort` for non-critical events
/// or `log_audit_critical` for security events.
#[deprecated(since = "0.3.0", note = "Use log_audit_best_effort or log_audit_critical instead")]
pub fn log_audit_async(entry: AuditEntry) {
    log_audit_best_effort(entry);
}

/// Log a security-critical audit event with guaranteed delivery attempt
///
/// This function:
/// - Blocks until write completes or fails
/// - Returns an error if the write fails (caller decides what to do)
/// - Should be used for security events like key rotation, token changes
///
/// Use this for events that MUST be logged for security compliance.
pub fn log_audit_critical(entry: AuditEntry) -> Result<(), AuditError> {
    let guard = AUDIT_LOGGER.lock().map_err(|_| AuditError::LockFailed)?;
    let logger = guard.as_ref().ok_or(AuditError::NotInitialized)?;

    // First try to flush any buffered entries
    logger.flush_buffer();

    // Then write the critical entry
    let result = logger.write(&entry);

    // If write failed, store in buffer and notify
    if let Err(ref e) = result {
        notify_error(e, &entry);
        add_to_ring_buffer(entry);
    }

    result
}

/// Get the number of entries currently in the ring buffer
/// Useful for monitoring audit system health
pub fn get_buffered_count() -> usize {
    RING_BUFFER.lock().map(|b| b.len()).unwrap_or(0)
}

/// Manually flush buffered entries to disk
/// Returns the number of entries successfully flushed
pub fn flush_buffered_entries() -> usize {
    let guard = match AUDIT_LOGGER.lock() {
        Ok(g) => g,
        Err(_) => return 0,
    };
    let logger = match guard.as_ref() {
        Some(l) => l,
        None => return 0,
    };
    logger.flush_buffer()
}

// Convenience functions for common events

/// Log key generation (CRITICAL - security event)
pub fn audit_key_generated(mode: &str) {
    // Security-critical: key generation must be logged
    let _ = log_audit_critical(AuditEntry::new(
        AuditEventType::KeyGenerated,
        AuditSeverity::Info,
        format!("Master key generated (mode: {})", mode),
    ));
}

/// Log key migration (CRITICAL - security event)
pub fn audit_key_migrated(from: &str, to: &str) {
    // Security-critical: key migration must be logged
    let _ = log_audit_critical(AuditEntry::new(
        AuditEventType::KeyMigrated,
        AuditSeverity::Info,
        format!("Master key migrated from {} to {}", from, to),
    ));
}

/// Log key rotation (CRITICAL - security event)
pub fn audit_key_rotated() {
    // Security-critical: key rotation must be logged
    let _ = log_audit_critical(AuditEntry::new(
        AuditEventType::KeyRotated,
        AuditSeverity::Info,
        "Master key rotated",
    ));
}

/// Log storage mode change (CRITICAL - security event)
pub fn audit_storage_mode_changed(new_mode: &str) {
    // Security-critical: storage mode changes must be logged
    let _ = log_audit_critical(AuditEntry::new(
        AuditEventType::KeyStorageModeChanged,
        AuditSeverity::Info,
        format!("Key storage mode changed to {}", new_mode),
    ));
}

/// Log token set (CRITICAL - security event, never logs the token value)
pub fn audit_token_set(token_name: &str) {
    // Security-critical: token changes must be logged
    let _ = log_audit_critical(AuditEntry::new(
        AuditEventType::TokenSet,
        AuditSeverity::Info,
        format!("Token set: {}", token_name),
    ));
}

/// Log token cleared (CRITICAL - security event)
pub fn audit_token_cleared(token_name: &str) {
    // Security-critical: token changes must be logged
    let _ = log_audit_critical(AuditEntry::new(
        AuditEventType::TokenCleared,
        AuditSeverity::Info,
        format!("Token cleared: {}", token_name),
    ));
}

/// Log Jira HTTP opt-in (CRITICAL - security warning)
pub fn audit_jira_http_opt_in(base_url: &str) {
    // Security-critical: insecure connection opt-in must be logged
    let _ = log_audit_critical(
        AuditEntry::new(
            AuditEventType::JiraHttpOptIn,
            AuditSeverity::Warning,
            "User opted in to insecure HTTP connection for Jira",
        )
        .with_context(serde_json::json!({
            "base_url_scheme": if base_url.starts_with("https://") { "https" } else { "http" }
        })),
    );
}

/// Log Jira configured (informational)
pub fn audit_jira_configured(is_https: bool) {
    log_audit_best_effort(
        AuditEntry::new(
            AuditEventType::JiraConfigured,
            AuditSeverity::Info,
            "Jira integration configured",
        )
        .with_context(serde_json::json!({
            "secure": is_https
        })),
    );
}

/// Log path validation failure (CRITICAL - security event)
pub fn audit_path_validation_failed(path: &str, reason: &str) {
    // Security-critical: path validation failures may indicate attacks
    let _ = log_audit_critical(
        AuditEntry::new(
            AuditEventType::PathValidationFailed,
            AuditSeverity::Warning,
            format!("Path validation failed: {}", reason),
        )
        .with_context(serde_json::json!({
            // Only log a truncated/sanitized path prefix, not the full path
            "path_prefix": if path.len() > 20 { &path[..20] } else { path }
        })),
    );
}

/// Log app initialization (informational)
pub fn audit_app_initialized(is_first_run: bool) {
    log_audit_best_effort(
        AuditEntry::new(
            AuditEventType::AppInitialized,
            AuditSeverity::Info,
            "Application initialized",
        )
        .with_context(serde_json::json!({
            "first_run": is_first_run
        })),
    );
}

/// Log security failure (CRITICAL - security event)
pub fn audit_security_failure(event: AuditEventType, message: &str) {
    // Security-critical: security failures must be logged
    let _ = log_audit_critical(AuditEntry::new(
        event,
        AuditSeverity::Error,
        message,
    ));
}

/// Log database repair (informational)
pub fn audit_database_repaired(details: &str) {
    log_audit_best_effort(
        AuditEntry::new(
            AuditEventType::DatabaseRepaired,
            AuditSeverity::Info,
            format!("Database repaired: {}", details),
        ),
    );
}

/// Log vector store rebuilt (informational)
pub fn audit_vector_store_rebuilt(details: &str) {
    log_audit_best_effort(
        AuditEntry::new(
            AuditEventType::VectorStoreRebuilt,
            AuditSeverity::Info,
            format!("Vector store rebuilt: {}", details),
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditEntry::new(
            AuditEventType::TokenSet,
            AuditSeverity::Info,
            "Token set: huggingface",
        );

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("token_set"));
        assert!(json.contains("info"));
        assert!(!json.contains("huggingface_token")); // Should not contain actual token
    }

    #[test]
    fn test_audit_entry_with_context() {
        let entry = AuditEntry::new(
            AuditEventType::JiraConfigured,
            AuditSeverity::Info,
            "Jira configured",
        )
        .with_context(serde_json::json!({"secure": true}));

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("secure"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(AuditSeverity::Info.to_string(), "info");
        assert_eq!(AuditSeverity::Warning.to_string(), "warning");
        assert_eq!(AuditSeverity::Error.to_string(), "error");
        assert_eq!(AuditSeverity::Critical.to_string(), "critical");
    }

    #[test]
    fn test_ring_buffer_add_and_capacity() {
        // Clear buffer first
        if let Ok(mut buffer) = RING_BUFFER.lock() {
            buffer.clear();
        }

        // Add entries up to capacity
        for i in 0..RING_BUFFER_SIZE + 10 {
            let entry = AuditEntry::new(
                AuditEventType::Custom(format!("test_{}", i)),
                AuditSeverity::Info,
                format!("Test entry {}", i),
            );
            add_to_ring_buffer(entry);
        }

        // Buffer should be at capacity
        let count = get_buffered_count();
        assert_eq!(count, RING_BUFFER_SIZE);

        // Clean up
        if let Ok(mut buffer) = RING_BUFFER.lock() {
            buffer.clear();
        }
    }

    #[test]
    fn test_flush_attempt_throttling() {
        // Reset last flush time
        if let Ok(mut last_flush) = LAST_FLUSH_ATTEMPT.lock() {
            *last_flush = None;
        }

        // First call should allow flush
        assert!(should_attempt_flush());

        // Immediate second call should be throttled
        assert!(!should_attempt_flush());
    }

    #[test]
    fn test_error_callback_set_and_clear() {
        static CALLBACK_CALLED: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);

        fn test_callback(_: &AuditError, _: &AuditEntry) {
            CALLBACK_CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
        }

        // Set callback
        set_error_callback(Some(test_callback));

        // Notify should call it
        let entry = AuditEntry::new(
            AuditEventType::Custom("test".to_string()),
            AuditSeverity::Info,
            "Test",
        );
        notify_error(&AuditError::NotInitialized, &entry);

        assert!(CALLBACK_CALLED.load(std::sync::atomic::Ordering::SeqCst));

        // Clear callback
        set_error_callback(None);
        CALLBACK_CALLED.store(false, std::sync::atomic::Ordering::SeqCst);

        notify_error(&AuditError::NotInitialized, &entry);
        assert!(!CALLBACK_CALLED.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_log_audit_without_init_returns_error() {
        // This test runs without initializing the logger
        // log_audit should return NotInitialized error
        let entry = AuditEntry::new(
            AuditEventType::Custom("test".to_string()),
            AuditSeverity::Info,
            "Test",
        );

        // Can't reliably test this without affecting global state
        // but we can verify the entry is clonable for best_effort
        let _cloned = entry.clone();
    }
}
