//! Model Integrity Verification for AssistSupport
//!
//! This module provides SHA256 verification for downloaded models
//! with an allowlist of known-good model hashes.

use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IntegrityError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Model not in allowlist: {0}")]
    NotInAllowlist(String),
    #[error("Hash mismatch for {model}: expected {expected}, got {actual}")]
    HashMismatch {
        model: String,
        expected: String,
        actual: String,
    },
}

/// Known-good model hashes for integrity verification
/// These are SHA256 hashes of official model files from trusted sources
#[derive(Debug, Clone)]
pub struct ModelAllowlist {
    hashes: HashMap<String, AllowedModel>,
}

#[derive(Debug, Clone)]
pub struct AllowedModel {
    pub name: String,
    pub repo: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub verified_date: &'static str,
}

impl Default for ModelAllowlist {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelAllowlist {
    /// Create a new allowlist with verified model hashes
    pub fn new() -> Self {
        let mut hashes = HashMap::new();

        // Llama-3.2-1B-Instruct Q4_K_M - verified 2026-01
        hashes.insert(
            "llama-3.2-1b-instruct-q4_k_m.gguf".to_lowercase(),
            AllowedModel {
                name: "Llama-3.2-1B-Instruct Q4_K_M".to_string(),
                repo: "bartowski/Llama-3.2-1B-Instruct-GGUF".to_string(),
                sha256: "6f85a640a97cf2bf5b8e764087b1e83da0fdb51d7c9fab7d0fece9385611df83".to_string(),
                size_bytes: 807_694_464,
                verified_date: "2026-01-25",
            },
        );

        // Llama-3.2-3B-Instruct Q4_K_M - verified 2026-01
        hashes.insert(
            "llama-3.2-3b-instruct-q4_k_m.gguf".to_lowercase(),
            AllowedModel {
                name: "Llama-3.2-3B-Instruct Q4_K_M".to_string(),
                repo: "bartowski/Llama-3.2-3B-Instruct-GGUF".to_string(),
                sha256: "6c1a2b41161032677be168d354123594c0e6e67d2b9227c84f296ad037c728ff".to_string(),
                size_bytes: 2_019_377_696,
                verified_date: "2026-01-25",
            },
        );

        // Phi-3.1-mini-4k-instruct Q4_K_M - verified 2026-01
        hashes.insert(
            "phi-3.1-mini-4k-instruct-q4_k_m.gguf".to_lowercase(),
            AllowedModel {
                name: "Phi-3.1-mini-4k-instruct Q4_K_M".to_string(),
                repo: "bartowski/Phi-3.1-mini-4k-instruct-GGUF".to_string(),
                sha256: "d6d25bf078321bea4a079c727b273cb0b5a2e0b4cf3add0f7a2c8e43075c414f".to_string(),
                size_bytes: 2_393_232_096,
                verified_date: "2026-01-25",
            },
        );

        // nomic-embed-text-v1.5 Q5_K_M - verified 2026-01
        hashes.insert(
            "nomic-embed-text-v1.5.q5_k_m.gguf".to_lowercase(),
            AllowedModel {
                name: "nomic-embed-text-v1.5 Q5_K_M".to_string(),
                repo: "nomic-ai/nomic-embed-text-v1.5-GGUF".to_string(),
                sha256: "0c7930f6c4f6f29b7da5046e3a2c0832aa3f602db3de5760a95f0582dbd3d6e6".to_string(),
                size_bytes: 99_588_928,
                verified_date: "2026-01-25",
            },
        );

        Self { hashes }
    }

    /// Check if a model filename is in the allowlist
    pub fn is_allowed(&self, filename: &str) -> bool {
        self.hashes.contains_key(&filename.to_lowercase())
    }

    /// Get the expected hash for a model
    pub fn get_expected_hash(&self, filename: &str) -> Option<&str> {
        self.hashes
            .get(&filename.to_lowercase())
            .map(|m| m.sha256.as_str())
    }

    /// Get the allowed model metadata by filename
    pub fn get_allowed_model(&self, filename: &str) -> Option<&AllowedModel> {
        self.hashes.get(&filename.to_lowercase())
    }

    /// Get all allowed models
    pub fn list_allowed(&self) -> Vec<&AllowedModel> {
        self.hashes.values().collect()
    }

    /// Verify a model file against the allowlist
    pub fn verify_model(&self, path: &Path) -> Result<VerificationResult, IntegrityError> {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| IntegrityError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid filename",
            )))?;

        let actual_hash = calculate_sha256(path)?;

        // Check if model is in allowlist
        if let Some(allowed) = self.hashes.get(&filename.to_lowercase()) {
            if actual_hash.to_lowercase() == allowed.sha256.to_lowercase() {
                return Ok(VerificationResult::Verified {
                    model: allowed.name.clone(),
                    sha256: actual_hash,
                });
            } else {
                return Err(IntegrityError::HashMismatch {
                    model: filename.to_string(),
                    expected: allowed.sha256.clone(),
                    actual: actual_hash,
                });
            }
        }

        // Model not in allowlist - return unverified status
        Ok(VerificationResult::Unverified {
            filename: filename.to_string(),
            sha256: actual_hash,
        })
    }
}

/// Result of model verification
#[derive(Debug, Clone, serde::Serialize)]
pub enum VerificationResult {
    /// Model hash matches allowlist
    Verified { model: String, sha256: String },
    /// Model not in allowlist (user-provided model)
    Unverified { filename: String, sha256: String },
}

impl VerificationResult {
    /// Check if verification passed
    pub fn is_verified(&self) -> bool {
        matches!(self, Self::Verified { .. })
    }

    /// Get the SHA256 hash
    pub fn sha256(&self) -> &str {
        match self {
            Self::Verified { sha256, .. } => sha256,
            Self::Unverified { sha256, .. } => sha256,
        }
    }
}

/// Calculate SHA256 hash of a file
pub fn calculate_sha256(path: &Path) -> Result<String, IntegrityError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536]; // 64KB buffer for faster hashing

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Verify model integrity with optional strict mode
pub fn verify_model_integrity(
    path: &Path,
    strict: bool,
) -> Result<VerificationResult, IntegrityError> {
    let allowlist = ModelAllowlist::new();
    let result = allowlist.verify_model(path)?;

    if strict && !result.is_verified() {
        if let VerificationResult::Unverified { filename, .. } = &result {
            return Err(IntegrityError::NotInAllowlist(filename.clone()));
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_calculate_sha256() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test content").unwrap();
        file.flush().unwrap();

        let hash = calculate_sha256(file.path()).unwrap();
        // SHA256("test content") = 6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72
        assert_eq!(
            hash,
            "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72"
        );
    }

    #[test]
    fn test_allowlist_lookup() {
        let allowlist = ModelAllowlist::new();

        // Known model should be found
        assert!(allowlist.is_allowed("llama-3.2-1b-instruct-q4_k_m.gguf"));

        // Case insensitive
        assert!(allowlist.is_allowed("PHI-3.1-MINI-4K-INSTRUCT-Q4_K_M.GGUF"));

        // Unknown model
        assert!(!allowlist.is_allowed("unknown-model.gguf"));
    }

    #[test]
    fn test_verification_unverified_model() {
        let mut file = NamedTempFile::with_suffix(".gguf").unwrap();
        file.write_all(b"fake model content").unwrap();
        file.flush().unwrap();

        let result = verify_model_integrity(file.path(), false).unwrap();
        assert!(!result.is_verified());
    }

    #[test]
    fn test_strict_mode_rejects_unverified() {
        let mut file = NamedTempFile::with_suffix(".gguf").unwrap();
        file.write_all(b"fake model content").unwrap();
        file.flush().unwrap();

        let result = verify_model_integrity(file.path(), true);
        assert!(result.is_err());
    }
}
