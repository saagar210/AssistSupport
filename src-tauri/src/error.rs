//! Application error types for AssistSupport
//!
//! Provides a unified error model across all commands with:
//! - Stable error codes for frontend handling
//! - User-friendly messages
//! - Optional internal details for logging
//! - Retry hints for UI

use serde::{Deserialize, Serialize};
use std::fmt;

/// Error categories for grouping and UI display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorCategory {
    /// Input validation errors (bad paths, invalid format)
    Validation,
    /// Security-related errors (auth, permissions)
    Security,
    /// Network errors (connection, timeout)
    Network,
    /// File I/O errors (read, write, disk space)
    Io,
    /// Internal errors (unexpected state, bugs)
    Internal,
    /// Resource not found
    NotFound,
    /// Operation cancelled by user
    Cancelled,
    /// Database errors
    Database,
    /// Model/AI errors
    Model,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation => write!(f, "validation"),
            Self::Security => write!(f, "security"),
            Self::Network => write!(f, "network"),
            Self::Io => write!(f, "io"),
            Self::Internal => write!(f, "internal"),
            Self::NotFound => write!(f, "not_found"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Database => write!(f, "database"),
            Self::Model => write!(f, "model"),
        }
    }
}

/// Stable error codes for frontend handling
/// Format: CATEGORY_SPECIFIC_ERROR
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorCode(pub String);

impl ErrorCode {
    // Validation errors
    pub const VALIDATION_INVALID_PATH: &'static str = "VALIDATION_INVALID_PATH";
    pub const VALIDATION_PATH_TRAVERSAL: &'static str = "VALIDATION_PATH_TRAVERSAL";
    pub const VALIDATION_SENSITIVE_PATH: &'static str = "VALIDATION_SENSITIVE_PATH";
    pub const VALIDATION_EMPTY_INPUT: &'static str = "VALIDATION_EMPTY_INPUT";
    pub const VALIDATION_INPUT_TOO_LARGE: &'static str = "VALIDATION_INPUT_TOO_LARGE";
    pub const VALIDATION_INVALID_FORMAT: &'static str = "VALIDATION_INVALID_FORMAT";
    pub const VALIDATION_INVALID_URL: &'static str = "VALIDATION_INVALID_URL";

    // Security errors
    pub const SECURITY_AUTH_FAILED: &'static str = "SECURITY_AUTH_FAILED";
    pub const SECURITY_PERMISSION_DENIED: &'static str = "SECURITY_PERMISSION_DENIED";
    pub const SECURITY_ENCRYPTION_FAILED: &'static str = "SECURITY_ENCRYPTION_FAILED";
    pub const SECURITY_DECRYPTION_FAILED: &'static str = "SECURITY_DECRYPTION_FAILED";
    pub const SECURITY_PASSPHRASE_REQUIRED: &'static str = "SECURITY_PASSPHRASE_REQUIRED";
    pub const SECURITY_HTTPS_REQUIRED: &'static str = "SECURITY_HTTPS_REQUIRED";

    // Network errors
    pub const NETWORK_CONNECTION_FAILED: &'static str = "NETWORK_CONNECTION_FAILED";
    pub const NETWORK_TIMEOUT: &'static str = "NETWORK_TIMEOUT";
    pub const NETWORK_RATE_LIMITED: &'static str = "NETWORK_RATE_LIMITED";

    // I/O errors
    pub const IO_FILE_NOT_FOUND: &'static str = "IO_FILE_NOT_FOUND";
    pub const IO_PERMISSION_DENIED: &'static str = "IO_PERMISSION_DENIED";
    pub const IO_DISK_FULL: &'static str = "IO_DISK_FULL";
    pub const IO_READ_ERROR: &'static str = "IO_READ_ERROR";
    pub const IO_WRITE_ERROR: &'static str = "IO_WRITE_ERROR";

    // Database errors
    pub const DB_NOT_INITIALIZED: &'static str = "DB_NOT_INITIALIZED";
    pub const DB_QUERY_FAILED: &'static str = "DB_QUERY_FAILED";
    pub const DB_INTEGRITY_ERROR: &'static str = "DB_INTEGRITY_ERROR";
    pub const DB_LOCK_FAILED: &'static str = "DB_LOCK_FAILED";

    // Model errors
    pub const MODEL_NOT_LOADED: &'static str = "MODEL_NOT_LOADED";
    pub const MODEL_LOAD_FAILED: &'static str = "MODEL_LOAD_FAILED";
    pub const MODEL_GENERATION_FAILED: &'static str = "MODEL_GENERATION_FAILED";
    pub const MODEL_ENGINE_NOT_INITIALIZED: &'static str = "MODEL_ENGINE_NOT_INITIALIZED";

    // Not found errors
    pub const NOT_FOUND_DOCUMENT: &'static str = "NOT_FOUND_DOCUMENT";
    pub const NOT_FOUND_TEMPLATE: &'static str = "NOT_FOUND_TEMPLATE";
    pub const NOT_FOUND_DRAFT: &'static str = "NOT_FOUND_DRAFT";
    pub const NOT_FOUND_NAMESPACE: &'static str = "NOT_FOUND_NAMESPACE";

    // Cancelled
    pub const CANCELLED_BY_USER: &'static str = "CANCELLED_BY_USER";

    // Internal errors
    pub const INTERNAL_ERROR: &'static str = "INTERNAL_ERROR";
    pub const INTERNAL_LOCK_FAILED: &'static str = "INTERNAL_LOCK_FAILED";

    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Application error type for all commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    /// Stable error code for frontend handling
    pub code: String,
    /// User-friendly error message
    pub message: String,
    /// Optional internal details for logging (not shown to user)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Whether the operation can be retried
    pub retryable: bool,
    /// Error category for grouping
    pub category: ErrorCategory,
}

impl AppError {
    /// Create a new application error
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        category: ErrorCategory,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            detail: None,
            retryable: false,
            category,
        }
    }

    /// Add internal detail for logging
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Mark as retryable
    pub fn retryable(mut self) -> Self {
        self.retryable = true;
        self
    }

    // =========================================================================
    // Convenience constructors for common errors
    // =========================================================================

    /// Validation error: invalid path
    pub fn invalid_path(message: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::VALIDATION_INVALID_PATH,
            message,
            ErrorCategory::Validation,
        )
    }

    /// Validation error: path traversal attempt
    pub fn path_traversal() -> Self {
        Self::new(
            ErrorCode::VALIDATION_PATH_TRAVERSAL,
            "Path must be within your home directory",
            ErrorCategory::Validation,
        )
    }

    /// Validation error: sensitive path blocked
    pub fn sensitive_path() -> Self {
        Self::new(
            ErrorCode::VALIDATION_SENSITIVE_PATH,
            "This directory contains sensitive data and cannot be used",
            ErrorCategory::Validation,
        )
    }

    /// Validation error: empty input
    pub fn empty_input(field: &str) -> Self {
        Self::new(
            ErrorCode::VALIDATION_EMPTY_INPUT,
            format!("{} cannot be empty", field),
            ErrorCategory::Validation,
        )
    }

    /// Validation error: input too large
    pub fn input_too_large(field: &str, max: usize) -> Self {
        Self::new(
            ErrorCode::VALIDATION_INPUT_TOO_LARGE,
            format!("{} exceeds maximum size of {} bytes", field, max),
            ErrorCategory::Validation,
        )
    }

    /// Validation error: invalid format
    pub fn invalid_format(message: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::VALIDATION_INVALID_FORMAT,
            message,
            ErrorCategory::Validation,
        )
    }

    /// Validation error: invalid URL
    pub fn invalid_url(message: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::VALIDATION_INVALID_URL,
            message,
            ErrorCategory::Validation,
        )
    }

    /// Security error: HTTPS required
    pub fn https_required() -> Self {
        Self::new(
            ErrorCode::SECURITY_HTTPS_REQUIRED,
            "HTTPS is required for this connection",
            ErrorCategory::Security,
        )
    }

    /// Security error: passphrase required
    pub fn passphrase_required() -> Self {
        Self::new(
            ErrorCode::SECURITY_PASSPHRASE_REQUIRED,
            "Passphrase required to unlock key storage",
            ErrorCategory::Security,
        )
    }

    /// Security error: authentication failed
    pub fn auth_failed(message: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::SECURITY_AUTH_FAILED,
            message,
            ErrorCategory::Security,
        )
        .retryable()
    }

    /// Database error: not initialized
    pub fn db_not_initialized() -> Self {
        Self::new(
            ErrorCode::DB_NOT_INITIALIZED,
            "Database not initialized",
            ErrorCategory::Database,
        )
    }

    /// Database error: query failed
    pub fn db_query_failed(detail: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::DB_QUERY_FAILED,
            "Database operation failed",
            ErrorCategory::Database,
        )
        .with_detail(detail)
    }

    /// Database error: lock failed
    pub fn db_lock_failed() -> Self {
        Self::new(
            ErrorCode::DB_LOCK_FAILED,
            "Failed to acquire database lock",
            ErrorCategory::Database,
        )
        .retryable()
    }

    /// Model error: not loaded
    pub fn model_not_loaded() -> Self {
        Self::new(
            ErrorCode::MODEL_NOT_LOADED,
            "No model is currently loaded",
            ErrorCategory::Model,
        )
    }

    /// Model error: engine not initialized
    pub fn engine_not_initialized(engine: &str) -> Self {
        Self::new(
            ErrorCode::MODEL_ENGINE_NOT_INITIALIZED,
            format!("{} engine not initialized", engine),
            ErrorCategory::Model,
        )
    }

    /// Model error: load failed
    pub fn model_load_failed(detail: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::MODEL_LOAD_FAILED,
            "Failed to load model",
            ErrorCategory::Model,
        )
        .with_detail(detail)
    }

    /// Not found error: document
    pub fn document_not_found(id: &str) -> Self {
        Self::new(
            ErrorCode::NOT_FOUND_DOCUMENT,
            format!("Document not found: {}", id),
            ErrorCategory::NotFound,
        )
    }

    /// Not found error: template
    pub fn template_not_found(id: &str) -> Self {
        Self::new(
            ErrorCode::NOT_FOUND_TEMPLATE,
            format!("Template not found: {}", id),
            ErrorCategory::NotFound,
        )
    }

    /// Not found error: draft
    pub fn draft_not_found(id: &str) -> Self {
        Self::new(
            ErrorCode::NOT_FOUND_DRAFT,
            format!("Draft not found: {}", id),
            ErrorCategory::NotFound,
        )
    }

    /// I/O error: file not found
    pub fn file_not_found(path: &str) -> Self {
        Self::new(
            ErrorCode::IO_FILE_NOT_FOUND,
            format!("File not found: {}", path),
            ErrorCategory::Io,
        )
    }

    /// Network error: connection failed
    pub fn connection_failed(detail: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::NETWORK_CONNECTION_FAILED,
            "Connection failed",
            ErrorCategory::Network,
        )
        .with_detail(detail)
        .retryable()
    }

    /// Cancelled by user
    pub fn cancelled() -> Self {
        Self::new(
            ErrorCode::CANCELLED_BY_USER,
            "Operation cancelled",
            ErrorCategory::Cancelled,
        )
    }

    /// Internal error
    pub fn internal(detail: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::INTERNAL_ERROR,
            "An internal error occurred",
            ErrorCategory::Internal,
        )
        .with_detail(detail)
    }

    /// Lock error
    pub fn lock_failed(what: &str) -> Self {
        Self::new(
            ErrorCode::INTERNAL_LOCK_FAILED,
            format!("Failed to acquire lock on {}", what),
            ErrorCategory::Internal,
        )
        .retryable()
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

// Convert from common error types
impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        Self::db_query_failed(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => Self::new(
                ErrorCode::IO_FILE_NOT_FOUND,
                "File or directory not found",
                ErrorCategory::Io,
            )
            .with_detail(e.to_string()),
            std::io::ErrorKind::PermissionDenied => Self::new(
                ErrorCode::IO_PERMISSION_DENIED,
                "Permission denied",
                ErrorCategory::Io,
            )
            .with_detail(e.to_string()),
            _ => Self::new(ErrorCode::IO_READ_ERROR, "I/O error", ErrorCategory::Io)
                .with_detail(e.to_string()),
        }
    }
}

impl From<crate::validation::ValidationError> for AppError {
    fn from(e: crate::validation::ValidationError) -> Self {
        match e {
            crate::validation::ValidationError::PathTraversal => Self::path_traversal(),
            crate::validation::ValidationError::PathNotFound(p) => {
                Self::invalid_path(format!("Path not found: {}", p))
            }
            crate::validation::ValidationError::InputTooLarge { size, max } => Self::new(
                ErrorCode::VALIDATION_INPUT_TOO_LARGE,
                format!("Input too large: {} bytes (max: {} bytes)", size, max),
                ErrorCategory::Validation,
            ),
            crate::validation::ValidationError::InvalidFormat(msg) => {
                if msg.contains("sensitive") {
                    Self::sensitive_path()
                } else {
                    Self::invalid_format(msg)
                }
            }
            crate::validation::ValidationError::EmptyInput => Self::empty_input("Input"),
        }
    }
}

impl From<crate::security::SecurityError> for AppError {
    fn from(e: crate::security::SecurityError) -> Self {
        match e {
            crate::security::SecurityError::PassphraseRequired => Self::passphrase_required(),
            crate::security::SecurityError::Encryption(msg) => Self::new(
                ErrorCode::SECURITY_ENCRYPTION_FAILED,
                "Encryption failed",
                ErrorCategory::Security,
            )
            .with_detail(msg),
            crate::security::SecurityError::Decryption(msg) => Self::new(
                ErrorCode::SECURITY_DECRYPTION_FAILED,
                "Decryption failed - check your password",
                ErrorCategory::Security,
            )
            .with_detail(msg),
            _ => Self::new(
                ErrorCode::SECURITY_AUTH_FAILED,
                "Security error",
                ErrorCategory::Security,
            )
            .with_detail(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_serialization() {
        let err = AppError::path_traversal();
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("VALIDATION_PATH_TRAVERSAL"));
        assert!(json.contains("validation"));
    }

    #[test]
    fn test_error_with_detail() {
        let err = AppError::db_query_failed("connection timeout");
        assert!(err.detail.is_some());
        assert_eq!(err.detail.unwrap(), "connection timeout");
    }

    #[test]
    fn test_error_retryable() {
        let err = AppError::connection_failed("timeout");
        assert!(err.retryable);

        let err = AppError::path_traversal();
        assert!(!err.retryable);
    }

    #[test]
    fn test_error_display() {
        let err = AppError::model_not_loaded();
        let display = err.to_string();
        assert!(display.contains("MODEL_NOT_LOADED"));
        assert!(display.contains("No model is currently loaded"));
    }
}
