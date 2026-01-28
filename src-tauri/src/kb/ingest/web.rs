//! Web page ingestion module for AssistSupport
//! Fetches and indexes web pages with SSRF protection
//!
//! Security: This module uses DNS pinning to prevent DNS rebinding attacks.
//! All DNS resolution is performed once at validation time, and the validated
//! IPs are used for the actual HTTP connection.

use super::{
    CancellationToken, IngestError, IngestPhase, IngestProgress, IngestResult, IngestedDocument,
    ProgressCallback,
};
use crate::db::{Database, IngestRunCompletion, IngestSource};
use crate::kb::dns::{build_ip_url, PinnedDnsResolver, ValidatedUrl};
use crate::kb::indexer::{KbIndexer, ParsedDocument, Section};
use crate::kb::network::{
    canonicalize_url, extract_same_origin_links, is_login_page, validate_url_for_ssrf_with_pinning,
    NetworkError, SsrfConfig,
};
use futures::StreamExt;
use reqwest::Client;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

/// Web page ingestion configuration
#[derive(Debug, Clone)]
pub struct WebIngestConfig {
    /// SSRF protection config
    pub ssrf: SsrfConfig,
    /// Maximum page size in bytes
    pub max_page_size: usize,
    /// Maximum pages to crawl in same-origin mode
    pub max_crawl_pages: usize,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Whether to follow same-origin links
    pub crawl_same_origin: bool,
    /// User agent string
    pub user_agent: String,
}

impl Default for WebIngestConfig {
    fn default() -> Self {
        Self {
            ssrf: SsrfConfig::default(),
            max_page_size: 5 * 1024 * 1024, // 5MB
            max_crawl_pages: 50,
            timeout_secs: 30,
            crawl_same_origin: false,
            user_agent: "AssistSupport/1.0 (Knowledge Base Indexer)".into(),
        }
    }
}

/// Fetched web page content
#[derive(Debug)]
pub struct FetchedPage {
    pub url: String,
    pub canonical_url: String,
    pub content: String,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub title: Option<String>,
}

/// Web page ingester with DNS pinning for SSRF protection
pub struct WebIngester {
    config: WebIngestConfig,
    client: Client,
    resolver: Arc<PinnedDnsResolver>,
}

impl WebIngester {
    /// Create a new web ingester with DNS pinning
    ///
    /// The resolver is created async and shared across all requests.
    pub async fn new(config: WebIngestConfig) -> IngestResult<Self> {
        // Create pinned DNS resolver
        let resolver = PinnedDnsResolver::new(config.ssrf.clone())
            .await
            .map_err(|e| IngestError::Network(NetworkError::RequestFailed(e.to_string())))?;

        // Create client without automatic redirect following
        // We handle redirects manually to validate each hop with DNS pinning
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent(&config.user_agent)
            .redirect(reqwest::redirect::Policy::none()) // Manual redirect handling
            .build()
            .map_err(|e| IngestError::Network(NetworkError::RequestFailed(e.to_string())))?;

        Ok(Self {
            config,
            client,
            resolver: Arc::new(resolver),
        })
    }

    /// Create a new web ingester (sync wrapper for backward compatibility)
    pub fn new_sync(config: WebIngestConfig) -> IngestResult<Self> {
        // For sync contexts, create a basic resolver
        // This is a fallback - prefer new() when possible
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent(&config.user_agent)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| IngestError::Network(NetworkError::RequestFailed(e.to_string())))?;

        // Create resolver in a blocking context
        let resolver = futures::executor::block_on(async {
            PinnedDnsResolver::new(config.ssrf.clone()).await
        })
        .map_err(|e| IngestError::Network(NetworkError::RequestFailed(e.to_string())))?;

        Ok(Self {
            config,
            client,
            resolver: Arc::new(resolver),
        })
    }

    /// Fetch a single web page with DNS pinning
    ///
    /// Security: DNS is resolved once and pinned IPs are used for the connection,
    /// preventing DNS rebinding attacks.
    pub async fn fetch_page(&self, url: &str) -> IngestResult<FetchedPage> {
        // Validate URL and get pinned IPs
        let validated = validate_url_for_ssrf_with_pinning(url, &self.resolver).await?;

        // Fetch with redirect handling
        self.fetch_with_redirects(validated, 0).await
    }

    /// Fetch a page following redirects with DNS pinning on each hop
    ///
    /// Uses Box::pin for recursive async call to avoid infinitely sized future.
    fn fetch_with_redirects<'a>(
        &'a self,
        validated: ValidatedUrl,
        redirect_count: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = IngestResult<FetchedPage>> + Send + 'a>>
    {
        Box::pin(async move {
            const MAX_REDIRECTS: usize = 10;

            if redirect_count >= MAX_REDIRECTS {
                return Err(IngestError::Network(NetworkError::RequestFailed(
                    "Too many redirects".into(),
                )));
            }

            // Build request URL - use IP directly to bypass DNS
            let (request_url, host_header) = if !validated.pinned_ips.is_empty() {
                build_ip_url(&validated)
                    .map_err(|e| IngestError::Network(NetworkError::RequestFailed(e.to_string())))?
            } else {
                // Allowlisted host - use original URL
                (validated.url.to_string(), validated.host.clone())
            };

            // Make request with proper Host header
            let response = self
                .client
                .get(&request_url)
                .header("Host", &host_header)
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() {
                        IngestError::Timeout(format!("Request to {} timed out", validated.url))
                    } else {
                        IngestError::Network(NetworkError::RequestFailed(e.to_string()))
                    }
                })?;

            // Check status - handle redirects manually with DNS pinning
            let status = response.status();

            // Handle redirects with DNS validation
            if status.is_redirection() {
                if let Some(location) = response.headers().get("location") {
                    let location_str = location.to_str().map_err(|_| {
                        IngestError::Network(NetworkError::RequestFailed(
                            "Invalid redirect location header".into(),
                        ))
                    })?;

                    // Resolve relative URLs against the current URL
                    let redirect_url = validated.url.join(location_str).map_err(|e| {
                        IngestError::Network(NetworkError::InvalidUrl(e.to_string()))
                    })?;

                    // Validate redirect target with DNS pinning
                    let redirect_validated =
                        validate_url_for_ssrf_with_pinning(redirect_url.as_str(), &self.resolver)
                            .await?;

                    // Follow redirect recursively
                    return self
                        .fetch_with_redirects(redirect_validated, redirect_count + 1)
                        .await;
                }
            }

            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(IngestError::NotFound(validated.url.to_string()));
            }
            if status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN
            {
                return Err(IngestError::AuthRequired(format!(
                    "Authentication required for {}",
                    validated.url
                )));
            }
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Err(IngestError::RateLimited(format!(
                    "Rate limited for {}",
                    validated.url
                )));
            }
            if !status.is_success() {
                return Err(IngestError::Network(NetworkError::RequestFailed(format!(
                    "HTTP {} for {}",
                    status, validated.url
                ))));
            }

            // Extract headers
            let content_type = response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            let etag = response
                .headers()
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            let last_modified = response
                .headers()
                .get("last-modified")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            // Check content type is HTML
            if let Some(ref ct) = content_type {
                if !ct.contains("text/html") && !ct.contains("application/xhtml") {
                    return Err(IngestError::InvalidSource(format!(
                        "URL {} is not HTML (content-type: {})",
                        validated.url, ct
                    )));
                }
            }

            // Get content length if available
            if let Some(content_length) = response.content_length() {
                if content_length as usize > self.config.max_page_size {
                    return Err(IngestError::ContentTooLarge {
                        size: content_length as usize,
                        max: self.config.max_page_size,
                    });
                }
            }

            // Read body with streaming size limit
            // This ensures we stop reading early if content exceeds max size
            let bytes = read_body_with_limit(response, self.config.max_page_size).await?;

            // Convert to string
            let content = String::from_utf8_lossy(&bytes).to_string();

            // Use the original URL for canonical (not the IP-based request URL)
            let canonical = canonicalize_url(validated.url.as_str())?;

            // Check for login page
            if is_login_page(&validated.url, Some(&content)) {
                return Err(IngestError::AuthRequired(format!(
                "URL {} appears to be a login page. Please download the content manually after authenticating.",
                validated.url
            )));
            }

            // Extract title from HTML
            let title = extract_html_title(&content);

            Ok(FetchedPage {
                url: validated.url.to_string(),
                canonical_url: canonical,
                content,
                content_type,
                etag,
                last_modified,
                title,
            })
        }) // Close Box::pin(async move {
    }

    /// Check if a page needs refresh based on ETag/Last-Modified
    ///
    /// Uses DNS pinning for SSRF protection.
    pub async fn check_needs_refresh(
        &self,
        url: &str,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> IngestResult<bool> {
        // Validate URL with DNS pinning
        let validated = validate_url_for_ssrf_with_pinning(url, &self.resolver).await?;

        // Build request URL using pinned IP
        let (request_url, host_header) = if !validated.pinned_ips.is_empty() {
            build_ip_url(&validated)
                .map_err(|e| IngestError::Network(NetworkError::RequestFailed(e.to_string())))?
        } else {
            (validated.url.to_string(), validated.host.clone())
        };

        // Build conditional request
        let mut request = self.client.head(&request_url).header("Host", &host_header);

        if let Some(etag) = etag {
            request = request.header("If-None-Match", etag);
        }
        if let Some(lm) = last_modified {
            request = request.header("If-Modified-Since", lm);
        }

        let response = request
            .send()
            .await
            .map_err(|e| IngestError::Network(NetworkError::RequestFailed(e.to_string())))?;

        // 304 Not Modified means no refresh needed
        Ok(response.status() != reqwest::StatusCode::NOT_MODIFIED)
    }

    /// Crawl pages from a starting URL (same-origin only)
    pub async fn crawl_same_origin(
        &self,
        start_url: &str,
        cancel_token: &CancellationToken,
        progress: Option<&ProgressCallback>,
    ) -> IngestResult<Vec<FetchedPage>> {
        let mut pages = Vec::new();
        let mut visited = HashSet::new();
        let mut to_visit = vec![start_url.to_string()];

        let start_parsed =
            Url::parse(start_url).map_err(|e| IngestError::InvalidSource(e.to_string()))?;
        let base_host = start_parsed.host_str().unwrap_or("");

        while let Some(url) = to_visit.pop() {
            if cancel_token.is_cancelled() {
                return Err(IngestError::Cancelled);
            }

            if visited.contains(&url) || pages.len() >= self.config.max_crawl_pages {
                continue;
            }
            visited.insert(url.clone());

            if let Some(progress) = progress {
                progress(IngestProgress {
                    phase: IngestPhase::Fetching,
                    current: pages.len(),
                    total: Some(self.config.max_crawl_pages),
                    message: format!("Fetching {}", url),
                });
            }

            match self.fetch_page(&url).await {
                Ok(page) => {
                    // Extract same-origin links
                    if self.config.crawl_same_origin {
                        if let Ok(page_url) = Url::parse(&page.url) {
                            let links = extract_same_origin_links(&page_url, &page.content);
                            for link in links {
                                if let Ok(link_url) = Url::parse(&link) {
                                    if link_url.host_str() == Some(base_host)
                                        && !visited.contains(&link)
                                    {
                                        to_visit.push(link);
                                    }
                                }
                            }
                        }
                    }
                    pages.push(page);
                }
                Err(e) => {
                    // Log error but continue crawling
                    tracing::warn!("Failed to fetch {}: {}", url, e);
                }
            }
        }

        Ok(pages)
    }

    /// Ingest a web page into the knowledge base
    pub async fn ingest_page(
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

        // Report progress
        if let Some(progress) = progress {
            progress(IngestProgress {
                phase: IngestPhase::Fetching,
                current: 0,
                total: None,
                message: format!("Fetching {}", url),
            });
        }

        // Fetch the page
        let page = self.fetch_page(url).await?;

        if cancel_token.is_cancelled() {
            return Err(IngestError::Cancelled);
        }

        // Check if source exists
        let source_uri = format!("url://{}", page.canonical_url);
        let now = chrono::Utc::now().to_rfc3339();

        let source = match db.find_ingest_source("web", &source_uri, namespace_id)? {
            Some(mut existing) => {
                // Update existing source
                existing.etag = page.etag.clone();
                existing.last_modified = page.last_modified.clone();
                existing.title = page.title.clone();
                existing.last_ingested_at = Some(now.clone());
                existing.status = "active".to_string();
                existing.updated_at = now.clone();
                db.save_ingest_source(&existing)?;
                existing
            }
            None => {
                // Create new source
                let source = IngestSource {
                    id: uuid::Uuid::new_v4().to_string(),
                    source_type: "web".to_string(),
                    source_uri: source_uri.clone(),
                    namespace_id: namespace_id.to_string(),
                    title: page.title.clone(),
                    etag: page.etag.clone(),
                    last_modified: page.last_modified.clone(),
                    content_hash: None,
                    last_ingested_at: Some(now.clone()),
                    status: "active".to_string(),
                    error_message: None,
                    metadata_json: None,
                    created_at: now.clone(),
                    updated_at: now.clone(),
                };
                db.save_ingest_source(&source)?;
                source
            }
        };

        // Create ingest run
        let run_id = db.create_ingest_run(&source.id)?;

        // Report progress
        if let Some(progress) = progress {
            progress(IngestProgress {
                phase: IngestPhase::Parsing,
                current: 0,
                total: None,
                message: "Parsing HTML content".to_string(),
            });
        }

        // Parse HTML to text
        let text_content = html_to_text(&page.content);
        let title = page.title.clone().unwrap_or_else(|| page.url.clone());

        // Compute content hash for incremental ingestion
        let content_hash = sha256_hash(&text_content);

        // Check if content is unchanged (incremental ingestion)
        let existing_doc: Option<(String, String, i32)> = db
            .conn()
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
                    title,
                    source_uri,
                    chunk_count: chunk_count as usize,
                    word_count: text_content.split_whitespace().count(),
                });
            }
        }

        // Extract headings and build sections
        let headings = extract_headings(&page.content);
        let sections = build_sections_from_headings(&text_content, &headings);

        // Create parsed document
        let parsed = ParsedDocument {
            title: Some(title.clone()),
            sections,
        };

        if cancel_token.is_cancelled() {
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

        // Report progress
        if let Some(progress) = progress {
            progress(IngestProgress {
                phase: IngestPhase::Chunking,
                current: 0,
                total: None,
                message: "Chunking content".to_string(),
            });
        }

        // Chunk the document using KbIndexer
        let indexer = KbIndexer::new();
        let chunks = indexer.chunk_document(&parsed);
        let chunk_count = chunks.len();
        let word_count = text_content.split_whitespace().count();

        // Delete existing document for this source if any
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
                "web",
                source.id,
            ],
        )?;

        // Insert chunks
        for (i, chunk) in chunks.iter().enumerate() {
            if cancel_token.is_cancelled() {
                // Rollback by deleting the document (cascades to chunks)
                db.conn()
                    .execute("DELETE FROM kb_documents WHERE id = ?", [&doc_id])?;
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

// Note: validate_redirect_target removed - redirect validation now uses DNS pinning
// directly in fetch_with_redirects()

/// Read response body with streaming size limit.
///
/// Stops reading early if the content exceeds max_size, avoiding memory exhaustion.
async fn read_body_with_limit(
    response: reqwest::Response,
    max_size: usize,
) -> IngestResult<Vec<u8>> {
    let mut stream = response.bytes_stream();
    let mut buffer = Vec::new();
    let mut total_bytes = 0usize;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result
            .map_err(|e| IngestError::Network(NetworkError::RequestFailed(e.to_string())))?;

        total_bytes += chunk.len();

        // Check size limit before adding to buffer
        if total_bytes > max_size {
            return Err(IngestError::ContentTooLarge {
                size: total_bytes,
                max: max_size,
            });
        }

        buffer.extend_from_slice(&chunk);
    }

    Ok(buffer)
}

/// Extract title from HTML
fn extract_html_title(html: &str) -> Option<String> {
    // Simple regex extraction (avoids full HTML parser)
    let re = regex_lite::Regex::new(r"<title[^>]*>([^<]+)</title>").ok()?;
    re.captures(html)
        .and_then(|cap| cap.get(1))
        .map(|m| html_entities_decode(m.as_str().trim()))
}

/// Extract headings from HTML
fn extract_headings(html: &str) -> Vec<(usize, String)> {
    let mut headings = Vec::new();
    let re = regex_lite::Regex::new(r"<h([1-6])[^>]*>([^<]+)</h[1-6]>").unwrap();

    for cap in re.captures_iter(html) {
        if let (Some(level), Some(text)) = (cap.get(1), cap.get(2)) {
            if let Ok(level) = level.as_str().parse::<usize>() {
                let text = html_entities_decode(text.as_str().trim());
                if !text.is_empty() {
                    headings.push((level, text));
                }
            }
        }
    }

    headings
}

/// Convert HTML to plain text (simple implementation)
fn html_to_text(html: &str) -> String {
    // Remove script and style blocks
    let re_script = regex_lite::Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap();
    let re_style = regex_lite::Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap();
    let re_comments = regex_lite::Regex::new(r"<!--.*?-->").unwrap();

    let text = re_script.replace_all(html, "");
    let text = re_style.replace_all(&text, "");
    let text = re_comments.replace_all(&text, "");

    // Replace block elements with newlines
    let re_blocks = regex_lite::Regex::new(r"<(?:p|div|br|h[1-6]|li|tr)[^>]*>").unwrap();
    let text = re_blocks.replace_all(&text, "\n");

    // Remove all remaining tags
    let re_tags = regex_lite::Regex::new(r"<[^>]+>").unwrap();
    let text = re_tags.replace_all(&text, "");

    // Decode HTML entities
    let text = html_entities_decode(&text);

    // Normalize whitespace
    let re_whitespace = regex_lite::Regex::new(r"\s+").unwrap();
    let text = re_whitespace.replace_all(&text, " ");

    // Normalize newlines
    let re_newlines = regex_lite::Regex::new(r"\n\s*\n").unwrap();
    let text = re_newlines.replace_all(&text, "\n\n");

    text.trim().to_string()
}

/// Decode common HTML entities
fn html_entities_decode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&copy;", "©")
        .replace("&reg;", "®")
        .replace("&trade;", "™")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
        .replace("&hellip;", "…")
}

/// Build sections from text content and extracted headings
fn build_sections_from_headings(content: &str, headings: &[(usize, String)]) -> Vec<Section> {
    if headings.is_empty() {
        // No headings - treat entire content as single section
        return vec![Section {
            heading: None,
            level: 0,
            content: content.to_string(),
        }];
    }

    // For web pages, we typically have the full text already extracted
    // Create sections based on headings (simplified approach)
    let sections = vec![Section {
        heading: None,
        level: 0,
        content: content.to_string(),
    }];

    // Note: A more sophisticated approach would split the content by heading positions
    // but that requires tracking positions in the HTML which is complex.
    // For now, we use the full content as a single section since
    // the chunker will split it appropriately.

    sections
}

/// Compute SHA256 hash of content
fn sha256_hash(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_html_title() {
        assert_eq!(
            extract_html_title("<html><head><title>Test Page</title></head></html>"),
            Some("Test Page".to_string())
        );
        assert_eq!(
            extract_html_title("<html><head><title>  Spaces  </title></head></html>"),
            Some("Spaces".to_string())
        );
        assert_eq!(extract_html_title("<html><head></head></html>"), None);
    }

    #[test]
    fn test_html_to_text() {
        let html = r#"
            <html>
            <head><title>Test</title></head>
            <body>
                <script>alert('xss')</script>
                <style>.hidden { display: none; }</style>
                <p>Hello <b>World</b>!</p>
                <p>Second paragraph.</p>
            </body>
            </html>
        "#;
        let text = html_to_text(html);
        assert!(text.contains("Hello World!"));
        assert!(text.contains("Second paragraph"));
        assert!(!text.contains("alert"));
        assert!(!text.contains("display: none"));
    }

    #[test]
    fn test_html_entities_decode() {
        assert_eq!(html_entities_decode("&amp;"), "&");
        assert_eq!(html_entities_decode("&lt;test&gt;"), "<test>");
        assert_eq!(html_entities_decode("&quot;quoted&quot;"), "\"quoted\"");
    }

    #[test]
    fn test_extract_headings() {
        let html = "<h1>Title</h1><p>Text</p><h2>Subtitle</h2>";
        let headings = extract_headings(html);
        assert_eq!(headings.len(), 2);
        assert_eq!(headings[0], (1, "Title".to_string()));
        assert_eq!(headings[1], (2, "Subtitle".to_string()));
    }

    #[tokio::test]
    async fn test_dns_pinning_blocks_private_ip() {
        let config = SsrfConfig::default();
        let resolver = PinnedDnsResolver::new(config).await.unwrap();

        // Direct IP should be blocked
        let result = validate_url_for_ssrf_with_pinning("http://127.0.0.1/", &resolver).await;
        assert!(result.is_err());

        let result = validate_url_for_ssrf_with_pinning("http://192.168.1.1/", &resolver).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dns_pinning_allows_public_ip() {
        let config = SsrfConfig::default();
        let resolver = PinnedDnsResolver::new(config).await.unwrap();

        // Public IP should be allowed
        let result = validate_url_for_ssrf_with_pinning("http://8.8.8.8/", &resolver).await;
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert_eq!(validated.pinned_ips.len(), 1);
    }

    #[tokio::test]
    async fn test_dns_pinning_returns_pinned_ips() {
        let config = SsrfConfig::default();
        let resolver = PinnedDnsResolver::new(config).await.unwrap();

        let result =
            validate_url_for_ssrf_with_pinning("http://1.1.1.1/path?query=1", &resolver).await;
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert!(!validated.pinned_ips.is_empty());
        assert_eq!(validated.host, "1.1.1.1");
        assert_eq!(validated.port, 80);
    }

    #[test]
    fn test_build_ip_url() {
        let validated = ValidatedUrl {
            url: Url::parse("https://example.com/path").unwrap(),
            host: "example.com".to_string(),
            port: 443,
            pinned_ips: vec![std::net::IpAddr::V4(std::net::Ipv4Addr::new(
                93, 184, 216, 34,
            ))],
        };

        let (ip_url, host_header) = build_ip_url(&validated).unwrap();
        assert_eq!(ip_url, "https://93.184.216.34:443/path");
        assert_eq!(host_header, "example.com");
    }
}
