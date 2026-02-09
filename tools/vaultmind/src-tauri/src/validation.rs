use std::path::{Path, PathBuf};

use crate::error::AppError;

// --- Path Validation ---

/// Canonicalize a file path and reject symlinks, ensuring the path exists
/// and resides under an allowed directory (user home).
pub fn validate_file_path(raw_path: &str) -> Result<PathBuf, AppError> {
    let path = Path::new(raw_path);

    // Must exist
    if !path.exists() {
        return Err(AppError::Validation(format!(
            "File does not exist: {}",
            raw_path
        )));
    }

    // Canonicalize resolves symlinks and ".." components
    let canonical = path.canonicalize().map_err(|e| {
        AppError::Validation(format!("Cannot resolve file path '{}': {}", raw_path, e))
    })?;

    // Reject symlinks: the original path's metadata (without following) must match
    let meta = std::fs::symlink_metadata(path).map_err(|e| {
        AppError::Validation(format!("Cannot read file metadata '{}': {}", raw_path, e))
    })?;
    if meta.file_type().is_symlink() {
        return Err(AppError::Validation(format!(
            "Symlinks are not allowed: {}",
            raw_path
        )));
    }

    // Must be a regular file
    if !canonical.is_file() {
        return Err(AppError::Validation(format!(
            "Path is not a regular file: {}",
            raw_path
        )));
    }

    // Restrict to user's home directory (defense-in-depth)
    if let Some(home) = dirs::home_dir() {
        if !canonical.starts_with(&home) {
            return Err(AppError::Validation(format!(
                "File must be within user home directory: {}",
                raw_path
            )));
        }
    }

    Ok(canonical)
}

// --- SSRF Protection ---

const ALLOWED_OLLAMA_HOSTS: &[&str] = &[
    "localhost",
    "127.0.0.1",
    "::1",
    "0.0.0.0",
];

/// Validate that an Ollama host is localhost-only (prevent SSRF).
pub fn validate_ollama_host(host: &str) -> Result<(), AppError> {
    let normalized = host.trim().to_lowercase();
    if normalized.is_empty() {
        return Err(AppError::Validation(
            "Ollama host cannot be empty".into(),
        ));
    }
    if !ALLOWED_OLLAMA_HOSTS.contains(&normalized.as_str()) {
        return Err(AppError::Validation(format!(
            "Ollama host must be localhost (got '{}').\
             Only local connections are allowed for security.",
            host
        )));
    }
    Ok(())
}

/// Validate that an Ollama port is a valid TCP port number.
pub fn validate_ollama_port(port: &str) -> Result<u16, AppError> {
    let trimmed = port.trim();
    let port_num: u16 = trimmed.parse().map_err(|_| {
        AppError::Validation(format!(
            "Invalid port number '{}': must be 1-65535",
            port
        ))
    })?;
    if port_num == 0 {
        return Err(AppError::Validation("Port cannot be 0".into()));
    }
    Ok(port_num)
}

// --- Model Name Validation ---

/// Validate an Ollama model name against allowed patterns.
/// Model names follow the format: namespace/name:tag or name:tag or name
/// Only alphanumeric, hyphens, underscores, dots, colons, and slashes allowed.
pub fn validate_model_name(name: &str) -> Result<(), AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("Model name cannot be empty".into()));
    }
    if trimmed.len() > 256 {
        return Err(AppError::Validation(
            "Model name too long (max 256 characters)".into(),
        ));
    }
    // Whitelist: alphanumeric, hyphens, underscores, dots, colons, forward slashes
    if !trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || "-_.:/@".contains(c))
    {
        return Err(AppError::Validation(format!(
            "Model name '{}' contains invalid characters.\
             Only alphanumeric, hyphens, underscores, dots, colons, and slashes are allowed.",
            name
        )));
    }
    Ok(())
}

// --- Settings Validation ---

/// Known settings keys and their validation rules.
pub fn validate_setting(key: &str, value: &str) -> Result<(), AppError> {
    match key {
        "ollama_host" => validate_ollama_host(value),
        "ollama_port" => validate_ollama_port(value).map(|_| ()),
        "embedding_model" | "chat_model" => validate_model_name(value),
        "chunk_size" => {
            let size: usize = value.parse().map_err(|_| {
                AppError::Validation(format!("Invalid chunk_size '{}': must be a number", value))
            })?;
            if !(64..=8192).contains(&size) {
                return Err(AppError::Validation(format!(
                    "chunk_size must be between 64 and 8192 (got {})",
                    size
                )));
            }
            Ok(())
        }
        "chunk_overlap" => {
            let overlap: usize = value.parse().map_err(|_| {
                AppError::Validation(format!(
                    "Invalid chunk_overlap '{}': must be a number",
                    value
                ))
            })?;
            if overlap > 4096 {
                return Err(AppError::Validation(format!(
                    "chunk_overlap must be <= 4096 (got {})",
                    overlap
                )));
            }
            Ok(())
        }
        "theme" => {
            if !["system", "light", "dark"].contains(&value) {
                return Err(AppError::Validation(format!(
                    "Invalid theme '{}': must be system, light, or dark",
                    value
                )));
            }
            Ok(())
        }
        "auto_ner" | "auto_relationships" => {
            if !["true", "false"].contains(&value) {
                return Err(AppError::Validation(format!(
                    "Invalid boolean setting '{}': must be true or false",
                    value
                )));
            }
            Ok(())
        }
        "context_token_budget" | "history_token_budget" => {
            let budget: usize = value.parse().map_err(|_| {
                AppError::Validation(format!("Invalid token budget '{}': must be a number", value))
            })?;
            if !(256..=131072).contains(&budget) {
                return Err(AppError::Validation(format!(
                    "Token budget must be between 256 and 131072 (got {})",
                    budget
                )));
            }
            Ok(())
        }
        // Allow unknown keys but log a warning
        _ => {
            tracing::warn!("Unknown setting key '{}' — allowing but unvalidated", key);
            Ok(())
        }
    }
}

// --- EPUB/ZIP Path Validation ---

/// Validate a path extracted from a ZIP archive (EPUB, DOCX) to prevent
/// path traversal attacks. Handles URL-encoded ".." and other evasions.
pub fn validate_zip_entry_path(path: &str) -> Result<(), AppError> {
    // URL-decode the path first to catch encoded traversal
    let decoded = url_decode(path);

    // Check for path traversal patterns
    let normalized = decoded.replace('\\', "/");
    for component in normalized.split('/') {
        if component == ".." {
            return Err(AppError::Validation(format!(
                "Path traversal detected in archive entry: {}",
                path
            )));
        }
    }

    // Reject absolute paths inside archives
    if normalized.starts_with('/') {
        return Err(AppError::Validation(format!(
            "Absolute path in archive entry not allowed: {}",
            path
        )));
    }

    Ok(())
}

/// Simple URL percent-decoding (covers %2e%2e = ".." and similar).
fn url_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (
                hex_digit(bytes[i + 1]),
                hex_digit(bytes[i + 2]),
            ) {
                result.push((hi << 4 | lo) as char);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

// --- ZIP Decompression Limits ---

/// Maximum decompressed size for a single ZIP entry (100 MB).
pub const MAX_ZIP_ENTRY_SIZE: u64 = 100 * 1024 * 1024;

/// Maximum total decompressed size for all entries in a ZIP (500 MB).
pub const MAX_ZIP_TOTAL_SIZE: u64 = 500 * 1024 * 1024;

/// Maximum number of entries in a ZIP archive.
pub const MAX_ZIP_ENTRIES: usize = 10_000;

/// Validate a ZIP archive against decompression limits (zip bomb defense).
pub fn validate_zip_archive(archive: &mut zip::ZipArchive<std::fs::File>) -> Result<(), AppError> {
    if archive.len() > MAX_ZIP_ENTRIES {
        return Err(AppError::Validation(format!(
            "Archive contains too many entries ({}, max {})",
            archive.len(),
            MAX_ZIP_ENTRIES
        )));
    }

    let mut total_size: u64 = 0;
    for i in 0..archive.len() {
        // Use raw file info to check sizes without decompressing
        if let Ok(file) = archive.by_index_raw(i) {
            let uncompressed = file.size();
            if uncompressed > MAX_ZIP_ENTRY_SIZE {
                return Err(AppError::Validation(format!(
                    "Archive entry '{}' too large ({} bytes, max {} bytes)",
                    file.name(),
                    uncompressed,
                    MAX_ZIP_ENTRY_SIZE
                )));
            }
            total_size += uncompressed;
            if total_size > MAX_ZIP_TOTAL_SIZE {
                return Err(AppError::Validation(format!(
                    "Archive total decompressed size exceeds limit ({} bytes)",
                    MAX_ZIP_TOTAL_SIZE
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Path Validation Tests ---

    #[test]
    fn test_validate_file_path_nonexistent() {
        let result = validate_file_path("/nonexistent/file.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_validate_file_path_directory_rejected() {
        // /tmp exists and is a directory
        let result = validate_file_path("/tmp");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_path_valid_file() {
        // Create a temp file under user's home directory
        let home = dirs::home_dir().expect("No home dir");
        let test_dir = home.join(".vaultmind_test_tmp");
        std::fs::create_dir_all(&test_dir).unwrap();
        let file_path = test_dir.join("validation_test.txt");
        std::fs::write(&file_path, "hello").unwrap();

        let result = validate_file_path(file_path.to_str().unwrap());
        // Cleanup
        let _ = std::fs::remove_file(&file_path);
        let _ = std::fs::remove_dir(&test_dir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_path_symlink_rejected() {
        let home = dirs::home_dir().expect("No home dir");
        let test_dir = home.join(".vaultmind_test_tmp");
        std::fs::create_dir_all(&test_dir).unwrap();

        let real_file = test_dir.join("real_symlink_test.txt");
        std::fs::write(&real_file, "hello").unwrap();

        let link_path = test_dir.join("link_symlink_test.txt");
        let _ = std::fs::remove_file(&link_path); // cleanup any leftover
        std::os::unix::fs::symlink(&real_file, &link_path).unwrap();

        let result = validate_file_path(link_path.to_str().unwrap());
        // Cleanup
        let _ = std::fs::remove_file(&link_path);
        let _ = std::fs::remove_file(&real_file);
        let _ = std::fs::remove_dir(&test_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Symlinks"));
    }

    // --- SSRF Protection Tests ---

    #[test]
    fn test_validate_ollama_host_localhost() {
        assert!(validate_ollama_host("localhost").is_ok());
        assert!(validate_ollama_host("127.0.0.1").is_ok());
        assert!(validate_ollama_host("::1").is_ok());
        assert!(validate_ollama_host("0.0.0.0").is_ok());
    }

    #[test]
    fn test_validate_ollama_host_remote_rejected() {
        assert!(validate_ollama_host("evil.com").is_err());
        assert!(validate_ollama_host("192.168.1.1").is_err());
        assert!(validate_ollama_host("10.0.0.1").is_err());
        assert!(validate_ollama_host("").is_err());
    }

    #[test]
    fn test_validate_ollama_host_case_insensitive() {
        assert!(validate_ollama_host("LOCALHOST").is_ok());
        assert!(validate_ollama_host("LocalHost").is_ok());
    }

    #[test]
    fn test_validate_ollama_port_valid() {
        assert_eq!(validate_ollama_port("11434").unwrap(), 11434);
        assert_eq!(validate_ollama_port("8080").unwrap(), 8080);
        assert_eq!(validate_ollama_port("1").unwrap(), 1);
        assert_eq!(validate_ollama_port("65535").unwrap(), 65535);
    }

    #[test]
    fn test_validate_ollama_port_invalid() {
        assert!(validate_ollama_port("0").is_err());
        assert!(validate_ollama_port("-1").is_err());
        assert!(validate_ollama_port("99999").is_err());
        assert!(validate_ollama_port("abc").is_err());
        assert!(validate_ollama_port("").is_err());
    }

    // --- Model Name Validation Tests ---

    #[test]
    fn test_validate_model_name_valid() {
        assert!(validate_model_name("llama3.2").is_ok());
        assert!(validate_model_name("nomic-embed-text").is_ok());
        assert!(validate_model_name("library/model:latest").is_ok());
        assert!(validate_model_name("user/repo:v1.0").is_ok());
    }

    #[test]
    fn test_validate_model_name_invalid() {
        assert!(validate_model_name("").is_err());
        assert!(validate_model_name("model name with spaces").is_err());
        assert!(validate_model_name("model;rm -rf /").is_err());
        assert!(validate_model_name("model\nname").is_err());
        let long_name = "a".repeat(257);
        assert!(validate_model_name(&long_name).is_err());
    }

    // --- Settings Validation Tests ---

    #[test]
    fn test_validate_setting_host() {
        assert!(validate_setting("ollama_host", "localhost").is_ok());
        assert!(validate_setting("ollama_host", "evil.com").is_err());
    }

    #[test]
    fn test_validate_setting_port() {
        assert!(validate_setting("ollama_port", "11434").is_ok());
        assert!(validate_setting("ollama_port", "abc").is_err());
    }

    #[test]
    fn test_validate_setting_chunk_size() {
        assert!(validate_setting("chunk_size", "512").is_ok());
        assert!(validate_setting("chunk_size", "10").is_err()); // too small
        assert!(validate_setting("chunk_size", "99999").is_err()); // too large
    }

    #[test]
    fn test_validate_setting_theme() {
        assert!(validate_setting("theme", "dark").is_ok());
        assert!(validate_setting("theme", "light").is_ok());
        assert!(validate_setting("theme", "system").is_ok());
        assert!(validate_setting("theme", "rainbow").is_err());
    }

    #[test]
    fn test_validate_setting_unknown_key_allowed() {
        assert!(validate_setting("some_future_key", "whatever").is_ok());
    }

    // --- ZIP Path Validation Tests ---

    #[test]
    fn test_validate_zip_entry_path_normal() {
        assert!(validate_zip_entry_path("OEBPS/chapter1.xhtml").is_ok());
        assert!(validate_zip_entry_path("word/document.xml").is_ok());
    }

    #[test]
    fn test_validate_zip_entry_path_traversal() {
        assert!(validate_zip_entry_path("../../../etc/passwd").is_err());
        assert!(validate_zip_entry_path("content/../../secret").is_err());
    }

    #[test]
    fn test_validate_zip_entry_path_url_encoded_traversal() {
        // %2e = '.', so %2e%2e = '..'
        assert!(validate_zip_entry_path("%2e%2e/%2e%2e/etc/passwd").is_err());
        assert!(validate_zip_entry_path("content/%2e%2e/secret").is_err());
    }

    #[test]
    fn test_validate_zip_entry_path_absolute() {
        assert!(validate_zip_entry_path("/etc/passwd").is_err());
    }

    // --- URL Decode Tests ---

    #[test]
    fn test_url_decode() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("%2e%2e"), "..");
        assert_eq!(url_decode("normal"), "normal");
        assert_eq!(url_decode("%"), "%"); // incomplete sequence
    }
}
