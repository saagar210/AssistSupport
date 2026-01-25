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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;

/// Maximum size for a single log file (5MB)
const MAX_LOG_SIZE: u64 = 5 * 1024 * 1024;

/// Number of rotated log files to keep
const MAX_LOG_FILES: usize = 5;

/// Audit log file name
const AUDIT_LOG_NAME: &str = "audit.log";

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

/// Log an audit event
pub fn log_audit(entry: AuditEntry) -> Result<(), AuditError> {
    let guard = AUDIT_LOGGER.lock().map_err(|_| AuditError::LockFailed)?;
    let logger = guard.as_ref().ok_or(AuditError::NotInitialized)?;
    logger.write(&entry)
}

/// Log an audit event (fire and forget, ignores errors)
pub fn log_audit_async(entry: AuditEntry) {
    std::thread::spawn(move || {
        let _ = log_audit(entry);
    });
}

// Convenience functions for common events

/// Log key generation
pub fn audit_key_generated(mode: &str) {
    log_audit_async(AuditEntry::new(
        AuditEventType::KeyGenerated,
        AuditSeverity::Info,
        format!("Master key generated (mode: {})", mode),
    ));
}

/// Log key migration
pub fn audit_key_migrated(from: &str, to: &str) {
    log_audit_async(AuditEntry::new(
        AuditEventType::KeyMigrated,
        AuditSeverity::Info,
        format!("Master key migrated from {} to {}", from, to),
    ));
}

/// Log key rotation
pub fn audit_key_rotated() {
    log_audit_async(AuditEntry::new(
        AuditEventType::KeyRotated,
        AuditSeverity::Info,
        "Master key rotated",
    ));
}

/// Log storage mode change
pub fn audit_storage_mode_changed(new_mode: &str) {
    log_audit_async(AuditEntry::new(
        AuditEventType::KeyStorageModeChanged,
        AuditSeverity::Info,
        format!("Key storage mode changed to {}", new_mode),
    ));
}

/// Log token set (never logs the token value)
pub fn audit_token_set(token_name: &str) {
    log_audit_async(AuditEntry::new(
        AuditEventType::TokenSet,
        AuditSeverity::Info,
        format!("Token set: {}", token_name),
    ));
}

/// Log token cleared
pub fn audit_token_cleared(token_name: &str) {
    log_audit_async(AuditEntry::new(
        AuditEventType::TokenCleared,
        AuditSeverity::Info,
        format!("Token cleared: {}", token_name),
    ));
}

/// Log Jira HTTP opt-in (security warning)
pub fn audit_jira_http_opt_in(base_url: &str) {
    log_audit_async(
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

/// Log Jira configured
pub fn audit_jira_configured(is_https: bool) {
    log_audit_async(
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

/// Log path validation failure
pub fn audit_path_validation_failed(path: &str, reason: &str) {
    log_audit_async(
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

/// Log app initialization
pub fn audit_app_initialized(is_first_run: bool) {
    log_audit_async(
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
}
