//! Input validation module for AssistSupport
//! Provides security checks for paths, sizes, and input formats

use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Sensitive directories under home that should never be indexed
/// These contain credentials, keys, and other sensitive data
const SENSITIVE_HOME_PATHS: &[&str] = &[
    ".ssh",              // SSH private keys
    ".aws",              // AWS credentials
    ".gnupg",            // GPG keys
    ".pgp",              // PGP keys
    ".config",           // App configs (often contain tokens)
    "Library/Keychains", // macOS Keychains
];

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

/// Check if a path is within a sensitive subdirectory of home
fn is_sensitive_path(path: &Path, home: &Path) -> bool {
    // Get relative path from home
    let Ok(relative) = path.strip_prefix(home) else {
        return false;
    };

    // Check if it starts with any sensitive path
    for sensitive in SENSITIVE_HOME_PATHS {
        if relative.starts_with(sensitive) {
            return true;
        }
    }

    false
}

/// Validate that a path is within the user's home directory
/// Auto-creates the directory if the parent is valid
/// Blocks access to sensitive subdirectories (.ssh, .aws, .gnupg, .config, Library/Keychains)
///
/// # Arguments
/// * `path` - The path to validate
///
/// # Returns
/// * `Ok(PathBuf)` - The canonicalized path if valid
/// * `Err(ValidationError)` - If the path is outside home, in a sensitive directory, or invalid
pub fn validate_within_home(path: &Path) -> Result<PathBuf, ValidationError> {
    let home = dirs::home_dir().ok_or(ValidationError::InvalidFormat(
        "Cannot determine home directory".into(),
    ))?;

    // If path exists, validate it
    if path.exists() {
        let canonical = path
            .canonicalize()
            .map_err(|_| ValidationError::PathNotFound(path.display().to_string()))?;
        let canonical_home = home
            .canonicalize()
            .map_err(|_| ValidationError::PathNotFound(home.display().to_string()))?;

        // Check if under home
        if !canonical.starts_with(&canonical_home) {
            return Err(ValidationError::PathTraversal);
        }

        // Check if it's a sensitive subdirectory
        if is_sensitive_path(&canonical, &canonical_home) {
            return Err(ValidationError::InvalidFormat(
                "This directory contains sensitive data and cannot be used".into(),
            ));
        }

        return Ok(canonical);
    }

    // Path doesn't exist - check parent and auto-create
    let parent = path
        .parent()
        .ok_or(ValidationError::InvalidFormat("Invalid path".into()))?;

    if !parent.exists() {
        return Err(ValidationError::PathNotFound(
            "Parent directory does not exist".into(),
        ));
    }

    let canonical_parent = parent.canonicalize().map_err(|_| {
        ValidationError::PathNotFound(parent.display().to_string())
    })?;
    let canonical_home = home
        .canonicalize()
        .map_err(|_| ValidationError::PathNotFound(home.display().to_string()))?;

    // Check parent is under home
    if !canonical_parent.starts_with(&canonical_home) {
        return Err(ValidationError::PathTraversal);
    }

    // Check if target would be in sensitive location
    let file_name = path
        .file_name()
        .ok_or(ValidationError::InvalidFormat("Invalid path".into()))?;
    let target_path = canonical_parent.join(file_name);

    if is_sensitive_path(&target_path, &canonical_home) {
        return Err(ValidationError::InvalidFormat(
            "This directory contains sensitive data and cannot be used".into(),
        ));
    }

    // Create the directory
    fs::create_dir_all(&target_path)
        .map_err(|e| ValidationError::InvalidFormat(format!("Failed to create directory: {}", e)))?;

    Ok(target_path)
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

    #[test]
    fn test_is_sensitive_path() {
        let home = dirs::home_dir().unwrap();

        // Should be sensitive
        assert!(super::is_sensitive_path(&home.join(".ssh"), &home));
        assert!(super::is_sensitive_path(&home.join(".ssh/id_rsa"), &home));
        assert!(super::is_sensitive_path(&home.join(".aws"), &home));
        assert!(super::is_sensitive_path(&home.join(".aws/credentials"), &home));
        assert!(super::is_sensitive_path(&home.join(".gnupg"), &home));
        assert!(super::is_sensitive_path(&home.join(".config"), &home));
        assert!(super::is_sensitive_path(&home.join("Library/Keychains"), &home));

        // Should not be sensitive
        assert!(!super::is_sensitive_path(&home.join("Documents"), &home));
        assert!(!super::is_sensitive_path(&home.join("Desktop"), &home));
        assert!(!super::is_sensitive_path(&home.join(".bashrc"), &home)); // file, not dir
    }

    #[test]
    fn test_validate_within_home_blocks_system_paths() {
        // These should fail (outside home)
        assert!(matches!(
            validate_within_home(Path::new("/etc")),
            Err(ValidationError::PathTraversal) | Err(ValidationError::PathNotFound(_))
        ));
        assert!(matches!(
            validate_within_home(Path::new("/var/log")),
            Err(ValidationError::PathTraversal) | Err(ValidationError::PathNotFound(_))
        ));
        assert!(matches!(
            validate_within_home(Path::new("/usr/local")),
            Err(ValidationError::PathTraversal) | Err(ValidationError::PathNotFound(_))
        ));
    }

    #[test]
    fn test_validate_within_home_blocks_sensitive_dirs() {
        let home = dirs::home_dir().unwrap();

        // These should fail (sensitive directories)
        if home.join(".ssh").exists() {
            let result = validate_within_home(&home.join(".ssh"));
            assert!(matches!(result, Err(ValidationError::InvalidFormat(_))));
        }

        if home.join(".aws").exists() {
            let result = validate_within_home(&home.join(".aws"));
            assert!(matches!(result, Err(ValidationError::InvalidFormat(_))));
        }

        if home.join(".config").exists() {
            let result = validate_within_home(&home.join(".config"));
            assert!(matches!(result, Err(ValidationError::InvalidFormat(_))));
        }
    }

    #[test]
    fn test_validate_within_home_allows_normal_dirs() {
        let home = dirs::home_dir().unwrap();

        // Documents should be allowed (if it exists)
        if home.join("Documents").exists() {
            let result = validate_within_home(&home.join("Documents"));
            assert!(result.is_ok());
        }

        // Desktop should be allowed (if it exists)
        if home.join("Desktop").exists() {
            let result = validate_within_home(&home.join("Desktop"));
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_validate_within_home_path_traversal() {
        let home = dirs::home_dir().unwrap();

        // Traversal attempt - should fail
        let traversal_path = home.join("Documents").join("..").join("..").join("etc");
        let result = validate_within_home(&traversal_path);
        assert!(matches!(
            result,
            Err(ValidationError::PathTraversal) | Err(ValidationError::PathNotFound(_))
        ));
    }
}
