//! Model download manager for AssistSupport
//! Supports HuggingFace downloads with resume, progress, and checksum verification

use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use std::fs::{File, OpenOptions};
use thiserror::Error;
use sha2::{Sha256, Digest};
use tokio::sync::mpsc;

use crate::security::{FileKeyStore, TOKEN_HUGGINGFACE};

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Download cancelled")]
    Cancelled,
    #[error("HuggingFace API error: {0}")]
    HuggingFaceApi(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
}

/// Download progress event
#[derive(Debug, Clone, serde::Serialize)]
pub enum DownloadProgress {
    Started { url: String, total_bytes: Option<u64> },
    Progress { downloaded: u64, total: Option<u64>, speed_bps: u64 },
    Completed { path: PathBuf, sha256: String },
    Error { message: String },
    Cancelled,
}

/// Model source information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelSource {
    pub name: String,
    pub repo: String,
    pub filename: String,
    pub size_bytes: Option<u64>,
    pub sha256: Option<String>,
}

impl ModelSource {
    /// Create HuggingFace model source
    pub fn huggingface(repo: &str, filename: &str) -> Self {
        Self {
            name: filename.to_string(),
            repo: repo.to_string(),
            filename: filename.to_string(),
            size_bytes: None,
            sha256: None,
        }
    }

    /// Get download URL
    pub fn download_url(&self) -> String {
        format!(
            "https://huggingface.co/{}/resolve/main/{}",
            self.repo, self.filename
        )
    }
}

/// Download manager
pub struct DownloadManager {
    models_dir: PathBuf,
    downloads_dir: PathBuf,
}

impl DownloadManager {
    /// Create a new download manager
    pub fn new(app_data_dir: &Path) -> Self {
        Self {
            models_dir: app_data_dir.join("models"),
            downloads_dir: app_data_dir.join("downloads"),
        }
    }

    /// Ensure directories exist
    pub fn init(&self) -> Result<(), DownloadError> {
        std::fs::create_dir_all(&self.models_dir)?;
        std::fs::create_dir_all(&self.downloads_dir)?;
        Ok(())
    }

    /// Get models directory
    pub fn models_dir(&self) -> &Path {
        &self.models_dir
    }

    /// List downloaded models
    pub fn list_models(&self) -> Result<Vec<PathBuf>, DownloadError> {
        let mut models = Vec::new();
        if self.models_dir.exists() {
            for entry in std::fs::read_dir(&self.models_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("gguf") {
                    models.push(path);
                }
            }
        }
        Ok(models)
    }

    /// Download a model with progress reporting
    pub async fn download(
        &self,
        source: &ModelSource,
        progress_tx: mpsc::Sender<DownloadProgress>,
        cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<PathBuf, DownloadError> {
        use std::sync::atomic::Ordering;

        self.init()?;

        let url = source.download_url();
        let dest_path = self.models_dir.join(&source.filename);
        let partial_path = self.downloads_dir.join(format!("{}.partial", source.filename));

        // Get HuggingFace token from file-based storage (optional)
        let hf_token = FileKeyStore::get_token(TOKEN_HUGGINGFACE).ok().flatten();

        // Build HTTP client
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(token) = &hf_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
                    .map_err(|e| DownloadError::Network(e.to_string()))?,
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| DownloadError::Network(e.to_string()))?;

        // Check for existing partial download
        let mut resume_from = if partial_path.exists() {
            std::fs::metadata(&partial_path)?.len()
        } else {
            0
        };

        // Start download with range request for resume
        let mut request = client.get(&url);
        if resume_from > 0 {
            request = request.header("Range", format!("bytes={}-", resume_from));
        }

        let response = request.send().await
            .map_err(|e| DownloadError::Network(e.to_string()))?;

        if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
            return Err(DownloadError::Network(format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        if resume_from > 0 && response.status() == reqwest::StatusCode::OK {
            // Server ignored Range header; restart from scratch to avoid corruption.
            resume_from = 0;
        }

        // Get total size
        let total_bytes = response.content_length()
            .map(|len| if resume_from > 0 { len + resume_from } else { len });

        let _ = progress_tx.send(DownloadProgress::Started {
            url: url.clone(),
            total_bytes,
        }).await;

        // Open file for writing (append if resuming)
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(resume_from > 0)
            .truncate(resume_from == 0)
            .open(&partial_path)?;

        // Download with progress
        let mut downloaded = resume_from;
        let mut last_progress_time = std::time::Instant::now();
        let mut last_downloaded = downloaded;

        let mut stream = response.bytes_stream();
        use futures::StreamExt;

        while let Some(chunk) = stream.next().await {
            // Check cancellation
            if cancel_flag.load(Ordering::Relaxed) {
                let _ = progress_tx.send(DownloadProgress::Cancelled).await;
                return Err(DownloadError::Cancelled);
            }

            let chunk = chunk.map_err(|e| DownloadError::Network(e.to_string()))?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;

            // Report progress every 100ms
            let now = std::time::Instant::now();
            if now.duration_since(last_progress_time).as_millis() >= 100 {
                let elapsed_secs = now.duration_since(last_progress_time).as_secs_f64();
                let bytes_since = downloaded - last_downloaded;
                let speed_bps = (bytes_since as f64 / elapsed_secs) as u64;

                let _ = progress_tx.send(DownloadProgress::Progress {
                    downloaded,
                    total: total_bytes,
                    speed_bps,
                }).await;

                last_progress_time = now;
                last_downloaded = downloaded;
            }
        }

        // Sync to disk
        file.sync_all()?;
        drop(file);

        // Calculate checksum
        let sha256 = self.calculate_sha256(&partial_path)?;

        // Verify checksum if provided
        if let Some(expected) = &source.sha256 {
            if sha256.to_lowercase() != expected.to_lowercase() {
                let _ = progress_tx.send(DownloadProgress::Error {
                    message: "Checksum mismatch".to_string(),
                }).await;
                return Err(DownloadError::ChecksumMismatch {
                    expected: expected.clone(),
                    actual: sha256,
                });
            }
        }

        // Move to final location
        std::fs::rename(&partial_path, &dest_path)?;

        let _ = progress_tx.send(DownloadProgress::Completed {
            path: dest_path.clone(),
            sha256: sha256.clone(),
        }).await;

        Ok(dest_path)
    }

    /// Calculate SHA256 checksum of a file
    pub fn calculate_sha256(&self, path: &Path) -> Result<String, DownloadError> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 { break; }
            hasher.update(&buffer[..n]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Delete a downloaded model
    pub fn delete_model(&self, filename: &str) -> Result<(), DownloadError> {
        let path = self.models_dir.join(filename);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Clean up partial downloads
    pub fn cleanup_partial(&self) -> Result<(), DownloadError> {
        if self.downloads_dir.exists() {
            for entry in std::fs::read_dir(&self.downloads_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("partial") {
                    std::fs::remove_file(&path)?;
                }
            }
        }
        Ok(())
    }
}

/// Fetch file info from HuggingFace API to get SHA256 checksum
/// Returns (size_bytes, sha256) if successful
pub async fn fetch_hf_file_info(repo: &str, filename: &str) -> Result<(u64, String), DownloadError> {
    // HuggingFace API endpoint for file metadata
    let url = format!(
        "https://huggingface.co/api/models/{}/tree/main",
        repo
    );

    let client = reqwest::Client::new();
    let response = client.get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| DownloadError::Network(e.to_string()))?;

    if !response.status().is_success() {
        return Err(DownloadError::HuggingFaceApi(format!(
            "HTTP {}", response.status()
        )));
    }

    let body: serde_json::Value = response.json()
        .await
        .map_err(|e| DownloadError::Network(e.to_string()))?;

    // Find the file in the response
    let files = body.as_array()
        .ok_or_else(|| DownloadError::HuggingFaceApi("Invalid API response".into()))?;

    for file in files {
        if file.get("path").and_then(|p| p.as_str()) == Some(filename) {
            // Get LFS info which contains the SHA256
            if let Some(lfs) = file.get("lfs") {
                let size = lfs.get("size")
                    .and_then(|s| s.as_u64())
                    .ok_or_else(|| DownloadError::HuggingFaceApi("Missing size".into()))?;
                let sha256 = lfs.get("sha256")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| DownloadError::HuggingFaceApi("Missing sha256".into()))?
                    .to_string();
                return Ok((size, sha256));
            }
            // Non-LFS file - get size from file object
            let _size = file.get("size")
                .and_then(|s| s.as_u64())
                .ok_or_else(|| DownloadError::HuggingFaceApi("Missing size".into()))?;
            return Err(DownloadError::HuggingFaceApi(
                "File is not LFS - no SHA256 available".into()
            ));
        }
    }

    Err(DownloadError::FileNotFound(filename.to_string()))
}

/// Recommended models
pub fn recommended_models() -> Vec<ModelSource> {
    vec![
        ModelSource {
            name: "Qwen2.5-7B-Instruct (Recommended)".to_string(),
            repo: "Qwen/Qwen2.5-7B-Instruct-GGUF".to_string(),
            filename: "qwen2.5-7b-instruct-q5_k_m.gguf".to_string(),
            size_bytes: Some(5_500_000_000),
            sha256: None,
        },
        ModelSource {
            name: "Llama-3.2-3B-Instruct (Fast)".to_string(),
            repo: "bartowski/Llama-3.2-3B-Instruct-GGUF".to_string(),
            filename: "Llama-3.2-3B-Instruct-Q5_K_M.gguf".to_string(),
            size_bytes: Some(2_500_000_000),
            sha256: None,
        },
        ModelSource {
            name: "nomic-embed-text (Embeddings)".to_string(),
            repo: "nomic-ai/nomic-embed-text-v1.5-GGUF".to_string(),
            filename: "nomic-embed-text-v1.5.Q5_K_M.gguf".to_string(),
            size_bytes: Some(550_000_000),
            sha256: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_source_url() {
        let source = ModelSource::huggingface("Qwen/Qwen2.5-7B-Instruct-GGUF", "model.gguf");
        assert_eq!(
            source.download_url(),
            "https://huggingface.co/Qwen/Qwen2.5-7B-Instruct-GGUF/resolve/main/model.gguf"
        );
    }

    #[test]
    fn test_recommended_models() {
        let models = recommended_models();
        assert!(models.len() >= 3);
        assert!(models.iter().any(|m| m.name.contains("Qwen")));
        assert!(models.iter().any(|m| m.name.contains("embed")));
    }
}
