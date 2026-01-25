//! Input validation module for AssistSupport
//! Provides security checks for paths, sizes, and input formats

use std::path::{Path, PathBuf};
use thiserror::Error;

/// Maximum size for text inputs (10MB)
pub const MAX_TEXT_INPUT_BYTES: usize = 10 * 1024 * 1024;

/// Maximum size for search queries
pub const MAX_QUERY_BYTES: usize = 10_000;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Path traversal detected: path escapes allowed directory")]
    PathTraversal,
    #[error("Path not found: {0}")]
    PathNotFound(String),
    #[error("Input exceeds size limit: {size} bytes (max: {max} bytes)")]
    InputTooLarge { size: usize, max: usize },
    #[error("Invalid input format: {0}")]
    InvalidFormat(String),
    #[error("Empty input not allowed")]
    EmptyInput,
}

/// Validate that a path is within an allowed directory (prevents path traversal)
/// Returns the canonicalized path if valid
pub fn validate_path_within(path: &Path, allowed_root: &Path) -> Result<PathBuf, ValidationError> {
    // Check path exists first
    if !path.exists() {
        return Err(ValidationError::PathNotFound(path.display().to_string()));
    }

    // Canonicalize both paths to resolve symlinks and relative components
    let canonical = path.canonicalize()
        .map_err(|_| ValidationError::PathNotFound(path.display().to_string()))?;
    let canonical_root = allowed_root.canonicalize()
        .map_err(|_| ValidationError::PathNotFound(allowed_root.display().to_string()))?;

    // Verify the path is within the allowed root
    if !canonical.starts_with(&canonical_root) {
        return Err(ValidationError::PathTraversal);
    }

    Ok(canonical)
}

/// Validate text input size
pub fn validate_text_size(text: &str, max_bytes: usize) -> Result<(), ValidationError> {
    let size = text.len();
    if size > max_bytes {
        return Err(ValidationError::InputTooLarge { size, max: max_bytes });
    }
    Ok(())
}

/// Validate non-empty input
pub fn validate_non_empty(text: &str) -> Result<(), ValidationError> {
    if text.trim().is_empty() {
        return Err(ValidationError::EmptyInput);
    }
    Ok(())
}

/// Validate Jira ticket ID format (e.g., "PROJ-123")
pub fn validate_ticket_id(ticket_id: &str) -> Result<(), ValidationError> {
    // Ticket ID format: PROJECT_KEY-NUMBER
    // PROJECT_KEY: 2-10 uppercase letters
    // NUMBER: 1-7 digits
    let re = regex_lite::Regex::new(r"^[A-Z]{2,10}-\d{1,7}$").unwrap();
    if !re.is_match(ticket_id) {
        return Err(ValidationError::InvalidFormat(
            "Invalid ticket ID format. Expected: PROJECT-123".into()
        ));
    }
    Ok(())
}

/// Validate URL format (basic check)
pub fn validate_url(url: &str) -> Result<(), ValidationError> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ValidationError::InvalidFormat(
            "URL must start with http:// or https://".into()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_validate_text_size_ok() {
        assert!(validate_text_size("hello", 1000).is_ok());
    }

    #[test]
    fn test_validate_text_size_too_large() {
        let large = "x".repeat(1001);
        assert!(matches!(
            validate_text_size(&large, 1000),
            Err(ValidationError::InputTooLarge { .. })
        ));
    }

    #[test]
    fn test_validate_non_empty() {
        assert!(validate_non_empty("hello").is_ok());
        assert!(matches!(validate_non_empty(""), Err(ValidationError::EmptyInput)));
        assert!(matches!(validate_non_empty("   "), Err(ValidationError::EmptyInput)));
    }

    #[test]
    fn test_validate_ticket_id() {
        assert!(validate_ticket_id("PROJ-123").is_ok());
        assert!(validate_ticket_id("ABC-1").is_ok());
        assert!(validate_ticket_id("LONGPROJ-9999999").is_ok());

        assert!(validate_ticket_id("proj-123").is_err()); // lowercase
        assert!(validate_ticket_id("P-123").is_err()); // too short
        assert!(validate_ticket_id("PROJ123").is_err()); // no dash
        assert!(validate_ticket_id("PROJ-").is_err()); // no number
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://localhost:8080").is_ok());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("example.com").is_err());
    }

    #[test]
    fn test_validate_path_within() {
        let temp = env::temp_dir();
        let valid_path = temp.join("test_file.txt");

        // Path doesn't exist, should fail
        assert!(matches!(
            validate_path_within(&valid_path, &temp),
            Err(ValidationError::PathNotFound(_))
        ));
    }
}
