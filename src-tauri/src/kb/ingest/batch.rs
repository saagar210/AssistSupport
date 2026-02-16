//! Batch processing module for AssistSupport
//! Handles batch ingestion of multiple sources

use super::{
    github::{GitHubIngestConfig, GitHubIngester},
    web::{WebIngestConfig, WebIngester},
    youtube::{YouTubeIngestConfig, YouTubeIngester},
    CancellationToken, IngestError, IngestPhase, IngestProgress, IngestResult, IngestedDocument,
    ProgressCallback,
};
use crate::db::Database;
use crate::validation::validate_within_home;
use std::path::Path;

/// Batch ingestion configuration
#[derive(Debug, Clone)]
pub struct BatchIngestConfig {
    pub web: WebIngestConfig,
    pub youtube: YouTubeIngestConfig,
    pub github: GitHubIngestConfig,
    /// Maximum concurrent ingestion jobs
    pub max_concurrent: usize,
}

impl Default for BatchIngestConfig {
    fn default() -> Self {
        Self {
            web: WebIngestConfig::default(),
            youtube: YouTubeIngestConfig::default(),
            github: GitHubIngestConfig::default(),
            max_concurrent: 3,
        }
    }
}

/// Batch ingestion source type
#[derive(Debug, Clone)]
pub enum BatchSource {
    WebPage { url: String },
    YouTube { url: String },
    GitHubRepo { path: String },
}

impl BatchSource {
    /// Parse a source string into a BatchSource
    /// Local paths must be within the user's home directory
    pub fn parse(source: &str) -> Option<Self> {
        let source = source.trim();

        // YouTube URLs
        if source.contains("youtube.com") || source.contains("youtu.be") {
            return Some(BatchSource::YouTube {
                url: source.to_string(),
            });
        }

        // HTTP(S) URLs
        if source.starts_with("http://") || source.starts_with("https://") {
            return Some(BatchSource::WebPage {
                url: source.to_string(),
            });
        }

        // Local paths (for GitHub repos) - must be within home directory
        if Path::new(source).exists() {
            // Validate the path is within home directory and not in sensitive locations
            if let Ok(validated) = validate_within_home(Path::new(source)) {
                return Some(BatchSource::GitHubRepo {
                    path: validated.to_string_lossy().to_string(),
                });
            }
        }

        None
    }

    /// Get source type as string
    pub fn source_type(&self) -> &'static str {
        match self {
            BatchSource::WebPage { .. } => "web",
            BatchSource::YouTube { .. } => "youtube",
            BatchSource::GitHubRepo { .. } => "github",
        }
    }

    /// Get source URI
    pub fn source_uri(&self) -> &str {
        match self {
            BatchSource::WebPage { url } => url,
            BatchSource::YouTube { url } => url,
            BatchSource::GitHubRepo { path } => path,
        }
    }
}

/// Result of a batch ingestion operation
#[derive(Debug)]
pub struct BatchResult {
    pub successful: Vec<IngestedDocument>,
    pub failed: Vec<BatchError>,
    pub cancelled: bool,
}

/// Error for a single source in a batch
#[derive(Debug)]
pub struct BatchError {
    pub source: String,
    pub error: String,
}

/// Batch ingestion manager
pub struct BatchIngester {
    #[allow(dead_code)]
    config: BatchIngestConfig,
    web_ingester: WebIngester,
    youtube_ingester: YouTubeIngester,
    github_ingester: GitHubIngester,
}

impl BatchIngester {
    /// Create a new batch ingester
    ///
    /// This is async because the web ingester requires async DNS resolver initialization.
    pub async fn new(config: BatchIngestConfig) -> IngestResult<Self> {
        let web_ingester = WebIngester::new(config.web.clone()).await?;
        let youtube_ingester = YouTubeIngester::new(config.youtube.clone());
        let github_ingester = GitHubIngester::new(config.github.clone());

        Ok(Self {
            config,
            web_ingester,
            youtube_ingester,
            github_ingester,
        })
    }

    /// Ingest multiple sources in batch
    pub async fn ingest_batch(
        &self,
        db: &Database,
        sources: &[BatchSource],
        namespace_id: &str,
        cancel_token: &CancellationToken,
        progress: Option<&ProgressCallback>,
    ) -> BatchResult {
        let mut successful = Vec::new();
        let mut failed = Vec::new();
        let total = sources.len();

        for (i, source) in sources.iter().enumerate() {
            if cancel_token.is_cancelled() {
                return BatchResult {
                    successful,
                    failed,
                    cancelled: true,
                };
            }

            // Report overall progress
            if let Some(progress) = progress {
                progress(IngestProgress {
                    phase: IngestPhase::Fetching,
                    current: i,
                    total: Some(total),
                    message: format!("Processing {} of {}: {}", i + 1, total, source.source_uri()),
                });
            }

            let result = match source {
                BatchSource::WebPage { url } => {
                    self.web_ingester
                        .ingest_page(db, url, namespace_id, cancel_token, None)
                        .await
                }
                BatchSource::YouTube { url } => {
                    self.youtube_ingester
                        .ingest_video(db, url, namespace_id, cancel_token, None)
                        .await
                }
                BatchSource::GitHubRepo { path } => {
                    // GitHub ingestion returns multiple documents
                    match self.github_ingester.ingest_local_repo(
                        db,
                        Path::new(path),
                        namespace_id,
                        cancel_token,
                        None,
                    ) {
                        Ok(docs) => {
                            successful.extend(docs);
                            continue;
                        }
                        Err(e) => Err(e),
                    }
                }
            };

            match result {
                Ok(doc) => successful.push(doc),
                Err(IngestError::Cancelled) => {
                    return BatchResult {
                        successful,
                        failed,
                        cancelled: true,
                    };
                }
                Err(e) => {
                    failed.push(BatchError {
                        source: source.source_uri().to_string(),
                        error: e.to_string(),
                    });
                }
            }
        }

        // Report completion
        if let Some(progress) = progress {
            progress(IngestProgress {
                phase: IngestPhase::Complete,
                current: total,
                total: Some(total),
                message: format!(
                    "Batch complete: {} successful, {} failed",
                    successful.len(),
                    failed.len()
                ),
            });
        }

        BatchResult {
            successful,
            failed,
            cancelled: false,
        }
    }

    /// Parse a list of source strings and ingest them
    pub async fn ingest_from_strings(
        &self,
        db: &Database,
        source_strings: &[String],
        namespace_id: &str,
        cancel_token: &CancellationToken,
        progress: Option<&ProgressCallback>,
    ) -> BatchResult {
        let mut sources = Vec::new();
        let mut failed = Vec::new();

        for s in source_strings {
            match BatchSource::parse(s) {
                Some(source) => sources.push(source),
                None => {
                    failed.push(BatchError {
                        source: s.clone(),
                        error: "Could not determine source type".to_string(),
                    });
                }
            }
        }

        let mut result = self
            .ingest_batch(db, &sources, namespace_id, cancel_token, progress)
            .await;

        result.failed.extend(failed);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_source_parse_web() {
        let source = BatchSource::parse("https://example.com/page");
        assert!(matches!(source, Some(BatchSource::WebPage { .. })));
    }

    #[test]
    fn test_batch_source_parse_youtube() {
        let source = BatchSource::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        assert!(matches!(source, Some(BatchSource::YouTube { .. })));

        let source = BatchSource::parse("https://youtu.be/dQw4w9WgXcQ");
        assert!(matches!(source, Some(BatchSource::YouTube { .. })));
    }

    #[test]
    fn test_batch_source_parse_invalid() {
        let source = BatchSource::parse("not-a-valid-source");
        assert!(source.is_none());
    }
}
