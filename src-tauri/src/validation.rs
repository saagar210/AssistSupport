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
    ".docker",           // Docker credentials
    ".kube",             // Kubernetes configs and tokens
    ".netrc",            // Network credentials (FTP, curl, etc.)
    ".git-credentials",  // Git credential store
    ".azure",            // Azure CLI credentials
    ".gcloud",           // Google Cloud credentials
];

/// Maximum size for text inputs (10MB)
pub const MAX_TEXT_INPUT_BYTES: usize = 10 * 1024 * 1024;

/// Maximum size for search queries
pub const MAX_QUERY_BYTES: usize = 10_000;

/// Maximum length for namespace IDs
pub const MAX_NAMESPACE_ID_LEN: usize = 64;

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
    #[error("Invalid namespace ID: {0}")]
    InvalidNamespaceId(String),
}

/// Normalize a user-provided namespace name into a valid slug ID.
/// Converts to lowercase, replaces spaces/underscores with hyphens,
/// removes invalid characters, and trims to max length.
///
/// # Examples
/// ```
/// use assistsupport_lib::validation::normalize_namespace_id;
/// assert_eq!(normalize_namespace_id("My Namespace"), "my-namespace");
/// assert_eq!(normalize_namespace_id("Product_Docs"), "product-docs");
/// assert_eq!(normalize_namespace_id("UPPERCASE"), "uppercase");
/// ```
pub fn normalize_namespace_id(input: &str) -> String {
    let normalized: String = input
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();

    // Collapse multiple hyphens and trim leading/trailing hyphens
    let mut result = String::new();
    let mut last_was_hyphen = true; // Start true to trim leading hyphens
    for c in normalized.chars() {
        if c == '-' {
            if !last_was_hyphen {
                result.push(c);
                last_was_hyphen = true;
            }
        } else {
            result.push(c);
            last_was_hyphen = false;
        }
    }

    // Trim trailing hyphen
    if result.ends_with('-') {
        result.pop();
    }

    // Truncate to max length
    if result.len() > MAX_NAMESPACE_ID_LEN {
        result.truncate(MAX_NAMESPACE_ID_LEN);
        // Don't end on a hyphen after truncation
        while result.ends_with('-') {
            result.pop();
        }
    }

    result
}

/// Validate a namespace ID against the slug pattern `[a-z0-9-]{1,64}`.
/// IDs must be 1-64 characters, lowercase alphanumeric with hyphens.
/// Cannot start or end with a hyphen.
///
/// # Arguments
/// * `id` - The namespace ID to validate
///
/// # Returns
/// * `Ok(())` if valid
/// * `Err(ValidationError::InvalidNamespaceId)` if invalid
pub fn validate_namespace_id(id: &str) -> Result<(), ValidationError> {
    if id.is_empty() {
        return Err(ValidationError::InvalidNamespaceId(
            "Namespace ID cannot be empty".into(),
        ));
    }

    if id.len() > MAX_NAMESPACE_ID_LEN {
        return Err(ValidationError::InvalidNamespaceId(format!(
            "Namespace ID exceeds maximum length of {} characters",
            MAX_NAMESPACE_ID_LEN
        )));
    }

    // Check pattern: [a-z0-9-]+, no leading/trailing hyphens
    if id.starts_with('-') || id.ends_with('-') {
        return Err(ValidationError::InvalidNamespaceId(
            "Namespace ID cannot start or end with a hyphen".into(),
        ));
    }

    for c in id.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-' {
            return Err(ValidationError::InvalidNamespaceId(format!(
                "Invalid character '{}'. Only lowercase letters, digits, and hyphens are allowed.",
                c
            )));
        }
    }

    Ok(())
}

/// Normalize and validate a namespace ID in one step.
/// First normalizes the input, then validates the result.
///
/// # Arguments
/// * `input` - The user-provided namespace name or ID
///
/// # Returns
/// * `Ok(String)` - The normalized and validated namespace ID
/// * `Err(ValidationError)` - If the normalized ID is still invalid
pub fn normalize_and_validate_namespace_id(input: &str) -> Result<String, ValidationError> {
    let normalized = normalize_namespace_id(input);
    if normalized.is_empty() {
        return Err(ValidationError::InvalidNamespaceId(
            "Input cannot be normalized to a valid namespace ID".into(),
        ));
    }
    validate_namespace_id(&normalized)?;
    Ok(normalized)
}

/// Validate that a path is within an allowed directory (prevents path traversal)
/// Returns the canonicalized path if valid
pub fn validate_path_within(path: &Path, allowed_root: &Path) -> Result<PathBuf, ValidationError> {
    // Check path exists first
    if !path.exists() {
        return Err(ValidationError::PathNotFound(path.display().to_string()));
    }

    // Canonicalize both paths to resolve symlinks and relative components
    let canonical = path
        .canonicalize()
        .map_err(|_| ValidationError::PathNotFound(path.display().to_string()))?;
    let canonical_root = allowed_root
        .canonicalize()
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

    let canonical_parent = parent
        .canonicalize()
        .map_err(|_| ValidationError::PathNotFound(parent.display().to_string()))?;
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
    fs::create_dir_all(&target_path).map_err(|e| {
        ValidationError::InvalidFormat(format!("Failed to create directory: {}", e))
    })?;

    Ok(target_path)
}

/// Validate text input size
pub fn validate_text_size(text: &str, max_bytes: usize) -> Result<(), ValidationError> {
    let size = text.len();
    if size > max_bytes {
        return Err(ValidationError::InputTooLarge {
            size,
            max: max_bytes,
        });
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
            "Invalid ticket ID format. Expected: PROJECT-123".into(),
        ));
    }
    Ok(())
}

/// Validate URL format (basic check)
pub fn validate_url(url: &str) -> Result<(), ValidationError> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ValidationError::InvalidFormat(
            "URL must start with http:// or https://".into(),
        ));
    }
    Ok(())
}

/// Validate URL requires HTTPS
pub fn validate_https_url(url: &str) -> Result<(), ValidationError> {
    if !url.starts_with("https://") {
        return Err(ValidationError::InvalidFormat(
            "URL must use HTTPS for security. HTTP connections require explicit opt-in.".into(),
        ));
    }
    Ok(())
}

/// Check if URL is HTTP (not HTTPS)
pub fn is_http_url(url: &str) -> bool {
    url.starts_with("http://") && !url.starts_with("https://")
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
        assert!(matches!(
            validate_non_empty(""),
            Err(ValidationError::EmptyInput)
        ));
        assert!(matches!(
            validate_non_empty("   "),
            Err(ValidationError::EmptyInput)
        ));
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
        assert!(super::is_sensitive_path(
            &home.join(".aws/credentials"),
            &home
        ));
        assert!(super::is_sensitive_path(&home.join(".gnupg"), &home));
        assert!(super::is_sensitive_path(&home.join(".config"), &home));
        assert!(super::is_sensitive_path(
            &home.join("Library/Keychains"),
            &home
        ));

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

    // Namespace ID tests
    #[test]
    fn test_normalize_namespace_id() {
        // Basic normalization
        assert_eq!(normalize_namespace_id("My Namespace"), "my-namespace");
        assert_eq!(normalize_namespace_id("Product_Docs"), "product-docs");
        assert_eq!(normalize_namespace_id("UPPERCASE"), "uppercase");
        assert_eq!(normalize_namespace_id("already-valid"), "already-valid");

        // Multiple spaces/underscores collapse
        assert_eq!(normalize_namespace_id("a  b"), "a-b");
        assert_eq!(normalize_namespace_id("a__b"), "a-b");
        assert_eq!(normalize_namespace_id("a - b"), "a-b");

        // Special characters removed
        assert_eq!(normalize_namespace_id("a@b#c"), "a-b-c");
        assert_eq!(normalize_namespace_id("hello!world"), "hello-world");

        // Leading/trailing trimmed
        assert_eq!(normalize_namespace_id("-test-"), "test");
        assert_eq!(normalize_namespace_id("  test  "), "test");

        // Numbers preserved
        assert_eq!(normalize_namespace_id("product-v2"), "product-v2");
        assert_eq!(normalize_namespace_id("123"), "123");
    }

    #[test]
    fn test_normalize_namespace_id_truncation() {
        let long_input = "a".repeat(100);
        let normalized = normalize_namespace_id(&long_input);
        assert_eq!(normalized.len(), MAX_NAMESPACE_ID_LEN);
        assert!(normalized.chars().all(|c| c == 'a'));
    }

    #[test]
    fn test_validate_namespace_id_valid() {
        assert!(validate_namespace_id("default").is_ok());
        assert!(validate_namespace_id("my-namespace").is_ok());
        assert!(validate_namespace_id("product-v2").is_ok());
        assert!(validate_namespace_id("a").is_ok());
        assert!(validate_namespace_id("abc123").is_ok());
        assert!(validate_namespace_id("a-b-c").is_ok());
    }

    #[test]
    fn test_validate_namespace_id_invalid() {
        // Empty
        assert!(matches!(
            validate_namespace_id(""),
            Err(ValidationError::InvalidNamespaceId(_))
        ));

        // Too long
        let long_id = "a".repeat(65);
        assert!(matches!(
            validate_namespace_id(&long_id),
            Err(ValidationError::InvalidNamespaceId(_))
        ));

        // Leading/trailing hyphens
        assert!(matches!(
            validate_namespace_id("-test"),
            Err(ValidationError::InvalidNamespaceId(_))
        ));
        assert!(matches!(
            validate_namespace_id("test-"),
            Err(ValidationError::InvalidNamespaceId(_))
        ));

        // Uppercase
        assert!(matches!(
            validate_namespace_id("MyNamespace"),
            Err(ValidationError::InvalidNamespaceId(_))
        ));

        // Special characters
        assert!(matches!(
            validate_namespace_id("test@namespace"),
            Err(ValidationError::InvalidNamespaceId(_))
        ));
        assert!(matches!(
            validate_namespace_id("test namespace"),
            Err(ValidationError::InvalidNamespaceId(_))
        ));
        assert!(matches!(
            validate_namespace_id("test_namespace"),
            Err(ValidationError::InvalidNamespaceId(_))
        ));
    }

    #[test]
    fn test_normalize_and_validate_namespace_id() {
        // Valid after normalization
        assert_eq!(
            normalize_and_validate_namespace_id("My Namespace").unwrap(),
            "my-namespace"
        );
        assert_eq!(
            normalize_and_validate_namespace_id("Product_V2").unwrap(),
            "product-v2"
        );

        // Already valid
        assert_eq!(
            normalize_and_validate_namespace_id("default").unwrap(),
            "default"
        );

        // Invalid - only special chars
        assert!(normalize_and_validate_namespace_id("@#$%").is_err());
    }
}
