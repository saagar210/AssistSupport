//! YouTube transcript ingestion module for AssistSupport
//! Uses yt-dlp to extract video transcripts/captions

use crate::db::{Database, IngestRunCompletion, IngestSource};
use crate::kb::indexer::{KbIndexer, ParsedDocument, Section};
use crate::kb::network::NetworkError;
use super::{CancellationToken, IngestError, IngestPhase, IngestProgress, IngestResult, IngestedDocument, ProgressCallback};
use std::process::Command;
use std::time::Duration;
use tokio::time::timeout;

/// YouTube ingestion configuration
#[derive(Debug, Clone)]
pub struct YouTubeIngestConfig {
    /// Path to yt-dlp binary
    pub ytdlp_path: String,
    /// Maximum transcript size in bytes
    pub max_transcript_size: usize,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Preferred language for captions (ISO 639-1)
    pub preferred_language: String,
}

impl Default for YouTubeIngestConfig {
    fn default() -> Self {
        Self {
            ytdlp_path: "yt-dlp".into(), // Assume in PATH
            max_transcript_size: 2 * 1024 * 1024, // 2MB
            timeout_secs: 60,
            preferred_language: "en".into(),
        }
    }
}

/// Video metadata from YouTube
#[derive(Debug, Clone)]
pub struct VideoMetadata {
    pub video_id: String,
    pub title: String,
    pub channel: Option<String>,
    pub duration_secs: Option<u64>,
    pub description: Option<String>,
}

/// Transcript entry
#[derive(Debug, Clone)]
pub struct TranscriptEntry {
    pub start_secs: f64,
    pub duration_secs: f64,
    pub text: String,
}

/// YouTube video ingester
pub struct YouTubeIngester {
    config: YouTubeIngestConfig,
}

impl YouTubeIngester {
    /// Create a new YouTube ingester
    pub fn new(config: YouTubeIngestConfig) -> Self {
        Self { config }
    }

    /// Extract video ID from various YouTube URL formats
    pub fn extract_video_id(url: &str) -> Option<String> {
        // Handle various YouTube URL formats:
        // - https://www.youtube.com/watch?v=VIDEO_ID
        // - https://youtu.be/VIDEO_ID
        // - https://www.youtube.com/embed/VIDEO_ID
        // - https://www.youtube.com/v/VIDEO_ID

        if let Ok(parsed) = url::Url::parse(url) {
            let host = parsed.host_str()?;

            // youtu.be format
            if host == "youtu.be" {
                return parsed.path().strip_prefix('/').map(|s| s.to_string());
            }

            // youtube.com formats
            if host.contains("youtube.com") {
                // /watch?v= format
                if parsed.path() == "/watch" {
                    for (key, value) in parsed.query_pairs() {
                        if key == "v" {
                            return Some(value.to_string());
                        }
                    }
                }

                // /embed/ or /v/ format
                if let Some(path) = parsed.path().strip_prefix("/embed/") {
                    return Some(path.split('/').next()?.to_string());
                }
                if let Some(path) = parsed.path().strip_prefix("/v/") {
                    return Some(path.split('/').next()?.to_string());
                }
            }
        }

        // Try to extract from plain video ID (11 characters)
        if url.len() == 11 && url.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Some(url.to_string());
        }

        None
    }

    /// Check if yt-dlp is available
    pub fn check_ytdlp_available(&self) -> bool {
        Command::new(&self.config.ytdlp_path)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Fetch video metadata
    pub async fn fetch_metadata(&self, video_id: &str) -> IngestResult<VideoMetadata> {
        let url = format!("https://www.youtube.com/watch?v={}", video_id);

        // Run yt-dlp to get metadata
        let output = tokio::process::Command::new(&self.config.ytdlp_path)
            .args([
                "--dump-json",
                "--no-download",
                "--no-warnings",
                &url,
            ])
            .output()
            .await
            .map_err(IngestError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Private video") || stderr.contains("Sign in") {
                return Err(IngestError::AuthRequired(format!(
                    "Video {} requires authentication",
                    video_id
                )));
            }
            if stderr.contains("Video unavailable") || stderr.contains("not available") {
                return Err(IngestError::NotFound(format!("Video {} not found", video_id)));
            }
            return Err(IngestError::Parse(format!(
                "Failed to fetch metadata: {}",
                stderr
            )));
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| IngestError::Parse(e.to_string()))?;

        Ok(VideoMetadata {
            video_id: video_id.to_string(),
            title: json["title"].as_str().unwrap_or("Untitled").to_string(),
            channel: json["channel"].as_str().map(|s| s.to_string()),
            duration_secs: json["duration"].as_u64(),
            description: json["description"].as_str().map(|s| s.to_string()),
        })
    }

    /// Fetch video transcript/captions
    pub async fn fetch_transcript(&self, video_id: &str) -> IngestResult<Vec<TranscriptEntry>> {
        let url = format!("https://www.youtube.com/watch?v={}", video_id);

        // Use the --dump-json metadata to locate caption URLs
        let output = tokio::process::Command::new(&self.config.ytdlp_path)
            .args([
                "--dump-json",
                "--no-download",
                "--no-warnings",
                &url,
            ])
            .output()
            .await
            .map_err(IngestError::Io)?;

        if !output.status.success() {
            return Err(IngestError::Parse(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| IngestError::Parse(e.to_string()))?;

        // Try to get subtitles URL from automatic_captions or subtitles
        let subs = json["automatic_captions"]
            .get(&self.config.preferred_language)
            .or_else(|| json["subtitles"].get(&self.config.preferred_language))
            .and_then(|v| v.as_array());

        let description = json["description"].as_str().unwrap_or("");

        let subs = match subs {
            Some(subs) if !subs.is_empty() => subs,
            _ => {
                // No captions available; fall back to description if available
                if description.len() > 100 {
                    return Ok(vec![TranscriptEntry {
                        start_secs: 0.0,
                        duration_secs: json["duration"].as_f64().unwrap_or(0.0),
                        text: description.to_string(),
                    }]);
                }
                return Err(IngestError::NotFound(format!(
                    "No captions available for video {} in language {}",
                    video_id, self.config.preferred_language
                )));
            }
        };

        let mut selected: Option<&serde_json::Value> = None;
        for entry in subs {
            let ext = entry.get("ext").and_then(|v| v.as_str()).unwrap_or("");
            if ext.eq_ignore_ascii_case("json3") {
                selected = Some(entry);
                break;
            }
            if ext.eq_ignore_ascii_case("vtt") && selected.is_none() {
                selected = Some(entry);
            }
            if selected.is_none() {
                selected = Some(entry);
            }
        }

        let selected = selected.ok_or_else(|| {
            IngestError::NotFound(format!(
                "No usable caption formats for video {}",
                video_id
            ))
        })?;

        let caption_url = selected
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IngestError::Parse("Caption URL missing".into()))?;
        let caption_ext = selected.get("ext").and_then(|v| v.as_str()).unwrap_or("");

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .build()
            .map_err(|e| IngestError::Network(NetworkError::RequestFailed(e.to_string())))?;

        let response = client
            .get(caption_url)
            .send()
            .await
            .map_err(|e| IngestError::Network(NetworkError::RequestFailed(e.to_string())))?;

        if !response.status().is_success() {
            return Err(IngestError::Network(NetworkError::RequestFailed(format!(
                "HTTP {} fetching captions",
                response.status()
            ))));
        }

        if let Some(content_length) = response.content_length() {
            if content_length as usize > self.config.max_transcript_size {
                return Err(IngestError::ContentTooLarge {
                    size: content_length as usize,
                    max: self.config.max_transcript_size,
                });
            }
        }

        let body = response
            .text()
            .await
            .map_err(|e| IngestError::Network(NetworkError::RequestFailed(e.to_string())))?;

        if body.len() > self.config.max_transcript_size {
            return Err(IngestError::ContentTooLarge {
                size: body.len(),
                max: self.config.max_transcript_size,
            });
        }

        let entries = if caption_ext.eq_ignore_ascii_case("json3") {
            parse_json3_transcript(&body)?
        } else {
            parse_plain_transcript(&body)
        };

        if !entries.is_empty() {
            return Ok(entries);
        }

        if description.len() > 100 {
            return Ok(vec![TranscriptEntry {
                start_secs: 0.0,
                duration_secs: json["duration"].as_f64().unwrap_or(0.0),
                text: description.to_string(),
            }]);
        }

        Err(IngestError::NotFound(format!(
            "No transcript available for video {}",
            video_id
        )))
    }

    /// Ingest a YouTube video into the knowledge base
    pub async fn ingest_video(
        &self,
        db: &Database,
        url: &str,
        namespace_id: &str,
        cancel_token: &CancellationToken,
        progress: Option<&ProgressCallback>,
    ) -> IngestResult<IngestedDocument> {
        if cancel_token.is_cancelled() {
            return Err(IngestError::Cancelled);
        }

        // Extract video ID
        let video_id = Self::extract_video_id(url)
            .ok_or_else(|| IngestError::InvalidSource(format!("Invalid YouTube URL: {}", url)))?;

        let source_uri = format!("youtube://{}", video_id);

        // Report progress
        if let Some(progress) = progress {
            progress(IngestProgress {
                phase: IngestPhase::Fetching,
                current: 0,
                total: None,
                message: format!("Fetching metadata for video {}", video_id),
            });
        }

        // Fetch metadata with timeout
        let metadata = timeout(
            Duration::from_secs(self.config.timeout_secs),
            self.fetch_metadata(&video_id)
        )
        .await
        .map_err(|_| IngestError::Timeout(format!("Metadata fetch timed out for {}", video_id)))??;

        if cancel_token.is_cancelled() {
            return Err(IngestError::Cancelled);
        }

        // Report progress
        if let Some(progress) = progress {
            progress(IngestProgress {
                phase: IngestPhase::Fetching,
                current: 0,
                total: None,
                message: format!("Fetching transcript for: {}", metadata.title),
            });
        }

        // Fetch transcript with timeout
        let transcript = timeout(
            Duration::from_secs(self.config.timeout_secs),
            self.fetch_transcript(&video_id)
        )
        .await
        .map_err(|_| IngestError::Timeout(format!("Transcript fetch timed out for {}", video_id)))??;

        if cancel_token.is_cancelled() {
            return Err(IngestError::Cancelled);
        }

        // Combine transcript entries
        let full_transcript: String = transcript
            .iter()
            .map(|e| e.text.clone())
            .collect::<Vec<_>>()
            .join(" ");

        // Check size limit
        if full_transcript.len() > self.config.max_transcript_size {
            return Err(IngestError::ContentTooLarge {
                size: full_transcript.len(),
                max: self.config.max_transcript_size,
            });
        }

        let now = chrono::Utc::now().to_rfc3339();

        // Create or update source
        let source = match db.find_ingest_source("youtube", &source_uri, namespace_id)? {
            Some(mut existing) => {
                existing.title = Some(metadata.title.clone());
                existing.last_ingested_at = Some(now.clone());
                existing.status = "active".to_string();
                existing.updated_at = now.clone();
                existing.metadata_json = Some(serde_json::json!({
                    "channel": metadata.channel,
                    "duration_secs": metadata.duration_secs,
                }).to_string());
                db.save_ingest_source(&existing)?;
                existing
            }
            None => {
                let source = IngestSource {
                    id: uuid::Uuid::new_v4().to_string(),
                    source_type: "youtube".to_string(),
                    source_uri: source_uri.clone(),
                    namespace_id: namespace_id.to_string(),
                    title: Some(metadata.title.clone()),
                    etag: None,
                    last_modified: None,
                    content_hash: None,
                    last_ingested_at: Some(now.clone()),
                    status: "active".to_string(),
                    error_message: None,
                    metadata_json: Some(serde_json::json!({
                        "channel": metadata.channel,
                        "duration_secs": metadata.duration_secs,
                    }).to_string()),
                    created_at: now.clone(),
                    updated_at: now.clone(),
                };
                db.save_ingest_source(&source)?;
                source
            }
        };

        // Create ingest run
        let run_id = db.create_ingest_run(&source.id)?;

        // Compute content hash for incremental ingestion
        let content_hash = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(full_transcript.as_bytes());
            hex::encode(hasher.finalize())
        };

        // Check if content is unchanged (incremental ingestion)
        let existing_doc: Option<(String, String, i32)> = db.conn()
            .query_row(
                "SELECT id, file_hash, chunk_count FROM kb_documents WHERE source_id = ?",
                rusqlite::params![source.id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .ok();

        if let Some((doc_id, existing_hash, chunk_count)) = existing_doc {
            if existing_hash == content_hash {
                // Content unchanged, skip re-indexing
                if let Some(progress) = progress {
                    progress(IngestProgress {
                        phase: IngestPhase::Complete,
                        current: 0,
                        total: Some(0),
                        message: "Content unchanged, skipping re-index".to_string(),
                    });
                }

                // Complete the run with no changes
                db.complete_ingest_run(IngestRunCompletion {
                    run_id: &run_id,
                    status: "completed",
                    docs_added: 0,
                    docs_updated: 0,
                    docs_removed: 0,
                    chunks_added: 0,
                    error_message: None,
                })?;

                return Ok(IngestedDocument {
                    id: doc_id,
                    title: metadata.title.clone(),
                    source_uri,
                    chunk_count: chunk_count as usize,
                    word_count: full_transcript.split_whitespace().count(),
                });
            }
        }

        // Report progress
        if let Some(progress) = progress {
            progress(IngestProgress {
                phase: IngestPhase::Chunking,
                current: 0,
                total: None,
                message: "Chunking transcript".to_string(),
            });
        }

        // Create parsed document
        let title = metadata.title.clone();
        let parsed = ParsedDocument {
            title: Some(title.clone()),
            sections: vec![Section {
                heading: Some(title.clone()),
                level: 1,
                content: full_transcript.clone(),
            }],
        };

        // Chunk the document
        let indexer = KbIndexer::new();
        let chunks = indexer.chunk_document(&parsed);
        let chunk_count = chunks.len();
        let word_count = full_transcript.split_whitespace().count();

        // Delete existing document for this source
        db.delete_documents_for_source(&source.id)?;

        // Report progress
        if let Some(progress) = progress {
            progress(IngestProgress {
                phase: IngestPhase::Indexing,
                current: 0,
                total: Some(chunk_count),
                message: format!("Indexing {} chunks", chunk_count),
            });
        }

        // Insert document
        let doc_id = uuid::Uuid::new_v4().to_string();
        // content_hash already computed earlier for incremental check

        db.conn().execute(
            "INSERT INTO kb_documents (id, file_path, file_hash, title, indexed_at, chunk_count,
                    namespace_id, source_type, source_id)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                doc_id,
                source_uri,
                content_hash,
                title,
                now,
                chunk_count as i32,
                namespace_id,
                "youtube",
                source.id,
            ],
        )?;

        // Insert chunks
        for (i, chunk) in chunks.iter().enumerate() {
            if cancel_token.is_cancelled() {
                db.conn().execute("DELETE FROM kb_documents WHERE id = ?", [&doc_id])?;
                db.complete_ingest_run(IngestRunCompletion {
                    run_id: &run_id,
                    status: "cancelled",
                    docs_added: 0,
                    docs_updated: 0,
                    docs_removed: 0,
                    chunks_added: 0,
                    error_message: None,
                })?;
                return Err(IngestError::Cancelled);
            }

            let chunk_id = uuid::Uuid::new_v4().to_string();
            db.conn().execute(
                "INSERT INTO kb_chunks (id, document_id, chunk_index, heading_path, content, word_count, namespace_id)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    chunk_id,
                    doc_id,
                    i as i32,
                    chunk.heading_path,
                    chunk.content,
                    chunk.word_count as i32,
                    namespace_id,
                ],
            )?;
        }

        // Complete ingest run
        db.complete_ingest_run(IngestRunCompletion {
            run_id: &run_id,
            status: "completed",
            docs_added: 1,
            docs_updated: 0,
            docs_removed: 0,
            chunks_added: chunk_count as i32,
            error_message: None,
        })?;

        // Report progress
        if let Some(progress) = progress {
            progress(IngestProgress {
                phase: IngestPhase::Complete,
                current: chunk_count,
                total: Some(chunk_count),
                message: format!("Indexed {} chunks", chunk_count),
            });
        }

        Ok(IngestedDocument {
            id: doc_id,
            title,
            source_uri,
            chunk_count,
            word_count,
        })
    }
}

fn parse_ms(value: &serde_json::Value) -> Option<f64> {
    value.as_f64().or_else(|| value.as_i64().map(|v| v as f64)).or_else(|| {
        value.as_str().and_then(|s| s.parse::<f64>().ok())
    })
}

fn parse_json3_transcript(body: &str) -> Result<Vec<TranscriptEntry>, IngestError> {
    let json: serde_json::Value =
        serde_json::from_str(body).map_err(|e| IngestError::Parse(e.to_string()))?;
    let mut entries = Vec::new();

    if let Some(events) = json.get("events").and_then(|v| v.as_array()) {
        for event in events {
            let start_ms = parse_ms(&event["tStartMs"]).unwrap_or(0.0);
            let duration_ms = parse_ms(&event["dDurationMs"]).unwrap_or(0.0);
            let mut text = String::new();

            if let Some(segs) = event.get("segs").and_then(|v| v.as_array()) {
                for seg in segs {
                    if let Some(chunk) = seg.get("utf8").and_then(|v| v.as_str()) {
                        text.push_str(chunk);
                    }
                }
            }

            let trimmed = text.trim();
            if !trimmed.is_empty() {
                entries.push(TranscriptEntry {
                    start_secs: start_ms / 1000.0,
                    duration_secs: duration_ms / 1000.0,
                    text: trimmed.to_string(),
                });
            }
        }
    }

    Ok(entries)
}

fn parse_plain_transcript(body: &str) -> Vec<TranscriptEntry> {
    let text = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with("WEBVTT"))
        .filter(|line| !line.contains("-->"))
        .collect::<Vec<_>>()
        .join(" ");

    if text.is_empty() {
        Vec::new()
    } else {
        vec![TranscriptEntry {
            start_secs: 0.0,
            duration_secs: 0.0,
            text,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_video_id() {
        // Standard watch URL
        assert_eq!(
            YouTubeIngester::extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );

        // Short URL
        assert_eq!(
            YouTubeIngester::extract_video_id("https://youtu.be/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );

        // Embed URL
        assert_eq!(
            YouTubeIngester::extract_video_id("https://www.youtube.com/embed/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );

        // Just video ID
        assert_eq!(
            YouTubeIngester::extract_video_id("dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );

        // Invalid
        assert_eq!(YouTubeIngester::extract_video_id("https://example.com"), None);
    }

    #[test]
    fn test_parse_json3_transcript_basic() {
        let body = r#"
        {
          "events": [
            {
              "tStartMs": 1000,
              "dDurationMs": 2000,
              "segs": [
                {"utf8": "Hello "},
                {"utf8": "world"}
              ]
            }
          ]
        }
        "#;

        let entries = parse_json3_transcript(body).expect("Failed to parse json3");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].text, "Hello world");
        assert!((entries[0].start_secs - 1.0).abs() < 0.01);
        assert!((entries[0].duration_secs - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_plain_transcript_vtt() {
        let body = r#"
        WEBVTT

        00:00:00.000 --> 00:00:02.000
        Hello

        00:00:02.000 --> 00:00:04.000
        world
        "#;

        let entries = parse_plain_transcript(body);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].text, "Hello world");
    }
}
