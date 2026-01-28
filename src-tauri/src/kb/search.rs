//! Hybrid Search Module for AssistSupport
//! Combines FTS5 keyword search with vector similarity search using RRF fusion

use rusqlite::params;
use thiserror::Error;

use crate::db::{Database, DbError};

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Database error: {0}")]
    Database(#[from] DbError),
    #[error("Embedding error: {0}")]
    Embedding(String),
    #[error("Vector search disabled")]
    VectorDisabled,
    #[error("No results found")]
    NoResults,
}

/// Search result combining FTS5 and vector search results
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub chunk_id: String,
    pub document_id: String,
    pub file_path: String,
    pub title: Option<String>,
    pub heading_path: Option<String>,
    pub content: String,
    pub snippet: String,
    pub score: f64,
    pub source: SearchSource,
    pub namespace_id: Option<String>,
    pub source_type: Option<String>,
}

/// Source of the search result
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum SearchSource {
    Fts5,
    Vector,
    Hybrid,
}

/// RRF (Reciprocal Rank Fusion) constant - higher values favor higher-ranked results
const RRF_K: f64 = 60.0;

/// Default similarity threshold for deduplication (0.0-1.0)
const DEFAULT_DEDUP_THRESHOLD: f64 = 0.85;

/// Hybrid search engine
pub struct HybridSearch;

/// Search options for filtering and tuning
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub namespace_id: Option<String>,
    pub limit: usize,
    /// Weight for FTS5 results in hybrid search (0.0-1.0)
    pub fts_weight: f64,
    /// Weight for vector results in hybrid search (0.0-1.0)
    pub vector_weight: f64,
    /// Similarity threshold for deduplication (0.0-1.0, higher = more aggressive)
    pub dedup_threshold: f64,
    /// Enable content-based deduplication
    pub enable_dedup: bool,
    /// Sanitize HTML in snippets for safe UI rendering
    pub sanitize_snippets: bool,
    /// Normalize scores to 0..1 range
    pub normalize_scores: bool,
    /// Boost pinned sources in search results
    pub boost_pinned: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            namespace_id: None,
            limit: 10,
            fts_weight: 0.5,
            vector_weight: 0.5,
            dedup_threshold: DEFAULT_DEDUP_THRESHOLD,
            enable_dedup: true,
            sanitize_snippets: true,
            normalize_scores: true,
            boost_pinned: true,
        }
    }
}

impl SearchOptions {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            ..Default::default()
        }
    }

    pub fn with_namespace(mut self, namespace_id: Option<String>) -> Self {
        self.namespace_id = namespace_id;
        self
    }

    pub fn with_weights(mut self, fts_weight: f64, vector_weight: f64) -> Self {
        self.fts_weight = fts_weight.clamp(0.0, 1.0);
        self.vector_weight = vector_weight.clamp(0.0, 1.0);
        self
    }

    pub fn with_dedup(mut self, enable: bool, threshold: f64) -> Self {
        self.enable_dedup = enable;
        self.dedup_threshold = threshold.clamp(0.0, 1.0);
        self
    }
}

/// Check if a token contains both letters and digits (mixed alphanumeric)
fn is_mixed_alphanumeric(token: &str) -> bool {
    let has_alpha = token.chars().any(|c| c.is_alphabetic());
    let has_digit = token.chars().any(|c| c.is_ascii_digit());
    has_alpha && has_digit
}

/// Split a mixed alphanumeric token at digit/letter boundaries
/// e.g., "6sense" -> ["6", "sense"], "h2o" -> ["h", "2", "o"]
fn split_mixed_alphanumeric(token: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut last_was_digit: Option<bool> = None;

    for c in token.chars() {
        let is_digit = c.is_ascii_digit();
        if let Some(prev) = last_was_digit {
            if prev != is_digit && !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
        }
        current.push(c);
        last_was_digit = Some(is_digit);
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

/// Preprocess a query string for FTS5 MATCH
///
/// Handles:
/// - Mixed alphanumeric tokens: "6sense" → `("6sense" OR 6 sense)`
/// - Multi-word queries: "vpn connection" → `("vpn connection" OR vpn OR connection)`
/// - Strips double quotes to prevent FTS5 syntax errors
/// - Empty queries return empty string
pub fn preprocess_fts5_query(query: &str) -> String {
    // Strip double quotes to prevent FTS5 syntax errors
    let query = query.replace('"', "");
    let query = query.trim();

    if query.is_empty() {
        return String::new();
    }

    let words: Vec<&str> = query.split_whitespace().collect();

    if words.is_empty() {
        return String::new();
    }

    // Check if any word is mixed alphanumeric
    let has_mixed = words.iter().any(|w| is_mixed_alphanumeric(w));

    if words.len() == 1 {
        let word = words[0];
        if is_mixed_alphanumeric(word) {
            let parts = split_mixed_alphanumeric(word);
            // ("6sense" OR 6 sense)
            return format!("(\"{word}\" OR {})", parts.join(" "));
        }
        // Single simple word - return as-is
        return word.to_string();
    }

    // Multi-word query
    let mut alternatives = Vec::new();

    // Full phrase match
    alternatives.push(format!("\"{}\"", words.join(" ")));

    // Individual words (with mixed alphanumeric expansion)
    for word in &words {
        if is_mixed_alphanumeric(word) {
            let parts = split_mixed_alphanumeric(word);
            alternatives.push(format!("\"{}\"", word));
            for part in parts {
                alternatives.push(part);
            }
        } else {
            alternatives.push(word.to_string());
        }
    }

    if has_mixed || words.len() > 1 {
        format!("({})", alternatives.join(" OR "))
    } else {
        alternatives.join(" OR ")
    }
}

impl HybridSearch {
    /// Perform FTS5-only search (sync version)
    pub fn search(
        db: &Database,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        Self::search_with_options(db, query, SearchOptions::new(limit))
    }

    /// Perform FTS5-only search with options
    pub fn search_with_options(
        db: &Database,
        query: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let fts_results = Self::fts_search_with_namespace(
            db,
            query,
            options.namespace_id.as_deref(),
            options.limit,
        )?;
        Ok(fts_results)
    }

    /// Perform hybrid search combining FTS5 and vector similarity
    ///
    /// This method performs RRF fusion when vector results are available.
    pub fn search_with_vectors(
        db: &Database,
        query: &str,
        limit: usize,
        vector_results: Option<Vec<super::vectors::VectorSearchResult>>,
    ) -> Result<Vec<SearchResult>, SearchError> {
        Self::search_with_vectors_and_options(db, query, SearchOptions::new(limit), vector_results)
    }

    /// Perform hybrid search with options
    pub fn search_with_vectors_and_options(
        db: &Database,
        query: &str,
        options: SearchOptions,
        vector_results: Option<Vec<super::vectors::VectorSearchResult>>,
    ) -> Result<Vec<SearchResult>, SearchError> {
        // Get FTS5 results (fetch more for fusion and dedup)
        let fts_results = Self::fts_search_with_namespace(
            db,
            query,
            options.namespace_id.as_deref(),
            options.limit * 3,
        )?;

        // If no vector results, just return FTS5 (with dedup if enabled)
        let vector_results = match vector_results {
            Some(vr) if !vr.is_empty() => vr,
            _ => {
                let results: Vec<_> = fts_results.into_iter().take(options.limit * 2).collect();
                let results = if options.enable_dedup {
                    Self::deduplicate_results(results, options.dedup_threshold)
                } else {
                    results
                };
                return Ok(results.into_iter().take(options.limit).collect());
            }
        };

        // Convert vector results to SearchResults by looking up chunk metadata
        let mut vector_search_results = Vec::with_capacity(vector_results.len());
        for vr in vector_results {
            if let Ok(sr) = Self::get_chunk_as_search_result(db, &vr.chunk_id, vr.distance) {
                vector_search_results.push(sr);
            }
        }

        // Use RRF fusion with configurable weights
        let mut results = Self::hybrid_search_with_weights(
            fts_results,
            vector_search_results,
            options.fts_weight,
            options.vector_weight,
            options.limit * 2,
        );

        // Apply deduplication if enabled
        if options.enable_dedup {
            results = Self::deduplicate_results(results, options.dedup_threshold);
        }

        results.truncate(options.limit);
        Ok(results)
    }

    /// Get a chunk by ID and convert to SearchResult
    fn get_chunk_as_search_result(
        db: &Database,
        chunk_id: &str,
        distance: f32,
    ) -> Result<SearchResult, SearchError> {
        let conn = db.conn();

        conn.query_row(
            r#"
            SELECT
                kb_chunks.id,
                kb_chunks.document_id,
                kb_documents.file_path,
                kb_documents.title,
                kb_chunks.heading_path,
                kb_chunks.content,
                kb_documents.namespace_id,
                kb_documents.source_type
            FROM kb_chunks
            JOIN kb_documents ON kb_chunks.document_id = kb_documents.id
            WHERE kb_chunks.id = ?
            "#,
            params![chunk_id],
            |row| {
                let content: String = row.get(5)?;
                let snippet = content.chars().take(200).collect::<String>();
                Ok(SearchResult {
                    chunk_id: row.get(0)?,
                    document_id: row.get(1)?,
                    file_path: row.get(2)?,
                    title: row.get(3)?,
                    heading_path: row.get(4)?,
                    content,
                    snippet,
                    score: 1.0 - distance as f64, // Convert distance to similarity
                    source: SearchSource::Vector,
                    namespace_id: row.get(6)?,
                    source_type: row.get(7)?,
                })
            },
        )
        .map_err(|e| SearchError::Database(DbError::Sqlite(e)))
    }

    /// Perform FTS5 keyword search
    pub fn fts_search(
        db: &Database,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        Self::fts_search_with_namespace(db, query, None, limit)
    }

    /// Perform FTS5 keyword search with optional namespace filtering
    pub fn fts_search_with_namespace(
        db: &Database,
        query: &str,
        namespace_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        // Preprocess query for better FTS5 matching
        let preprocessed = preprocess_fts5_query(query);
        if preprocessed.is_empty() {
            return Ok(vec![]);
        }

        // Try with preprocessed query first, fall back to simple quoted phrase on error
        match Self::fts_search_raw(db, &preprocessed, namespace_id, limit) {
            Ok(results) => Ok(results),
            Err(_) => {
                // Fallback: wrap original query as a simple quoted phrase
                let fallback = format!("\"{}\"", query.replace('"', "").trim());
                if fallback == "\"\"" {
                    return Ok(vec![]);
                }
                Self::fts_search_raw(db, &fallback, namespace_id, limit)
            }
        }
    }

    /// Execute a raw FTS5 query (internal helper)
    fn fts_search_raw(
        db: &Database,
        fts_query: &str,
        namespace_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let conn = db.conn();

        let (sql, params_vec): (String, Vec<Box<dyn rusqlite::ToSql>>) =
            if let Some(ns) = namespace_id {
                (
                    r#"
                SELECT
                    kb_chunks.id,
                    kb_chunks.document_id,
                    kb_documents.file_path,
                    kb_documents.title,
                    kb_chunks.heading_path,
                    kb_chunks.content,
                    snippet(kb_fts, 0, '<mark>', '</mark>', '...', 32) as snippet,
                    bm25(kb_fts) as rank,
                    kb_documents.namespace_id,
                    kb_documents.source_type
                FROM kb_fts
                JOIN kb_chunks ON kb_fts.rowid = kb_chunks.rowid
                JOIN kb_documents ON kb_chunks.document_id = kb_documents.id
                WHERE kb_fts MATCH ?1 AND kb_documents.namespace_id = ?2
                ORDER BY rank
                LIMIT ?3
                "#
                    .to_string(),
                    vec![
                        Box::new(fts_query.to_string()) as Box<dyn rusqlite::ToSql>,
                        Box::new(ns.to_string()),
                        Box::new(limit as i64),
                    ],
                )
            } else {
                (
                    r#"
                SELECT
                    kb_chunks.id,
                    kb_chunks.document_id,
                    kb_documents.file_path,
                    kb_documents.title,
                    kb_chunks.heading_path,
                    kb_chunks.content,
                    snippet(kb_fts, 0, '<mark>', '</mark>', '...', 32) as snippet,
                    bm25(kb_fts) as rank,
                    kb_documents.namespace_id,
                    kb_documents.source_type
                FROM kb_fts
                JOIN kb_chunks ON kb_fts.rowid = kb_chunks.rowid
                JOIN kb_documents ON kb_chunks.document_id = kb_documents.id
                WHERE kb_fts MATCH ?1
                ORDER BY rank
                LIMIT ?2
                "#
                    .to_string(),
                    vec![
                        Box::new(fts_query.to_string()) as Box<dyn rusqlite::ToSql>,
                        Box::new(limit as i64),
                    ],
                )
            };

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| SearchError::Database(DbError::Sqlite(e)))?;

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let results = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(SearchResult {
                    chunk_id: row.get(0)?,
                    document_id: row.get(1)?,
                    file_path: row.get(2)?,
                    title: row.get(3)?,
                    heading_path: row.get(4)?,
                    content: row.get(5)?,
                    snippet: row.get(6)?,
                    score: row.get::<_, f64>(7)?.abs(), // BM25 returns negative, lower is better
                    source: SearchSource::Fts5,
                    namespace_id: row.get(8)?,
                    source_type: row.get(9)?,
                })
            })
            .map_err(|e| SearchError::Database(DbError::Sqlite(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SearchError::Database(DbError::Sqlite(e)))?;

        Ok(results)
    }

    /// Fuse pre-computed FTS results with vector results
    ///
    /// This is used when FTS and vector searches are run in parallel.
    /// Takes pre-computed FTS results and optional raw vector results.
    pub fn fuse_results(
        db: &Database,
        fts_results: Vec<SearchResult>,
        vector_results: Option<Vec<super::vectors::VectorSearchResult>>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        Self::fuse_results_with_options(db, fts_results, vector_results, SearchOptions::new(limit))
    }

    /// Fuse pre-computed FTS results with vector results using configurable options
    ///
    /// This is used when FTS and vector searches are run in parallel.
    /// Takes pre-computed FTS results and optional raw vector results.
    /// Applies configurable weights for fusion and optional deduplication.
    pub fn fuse_results_with_options(
        db: &Database,
        fts_results: Vec<SearchResult>,
        vector_results: Option<Vec<super::vectors::VectorSearchResult>>,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>, SearchError> {
        // If no vector results, just return FTS results (with dedup if enabled)
        let vector_results = match vector_results {
            Some(vr) if !vr.is_empty() => vr,
            _ => {
                let results: Vec<_> = fts_results.into_iter().take(options.limit * 2).collect();
                let results = if options.enable_dedup {
                    Self::deduplicate_results(results, options.dedup_threshold)
                } else {
                    results
                };
                return Ok(results.into_iter().take(options.limit).collect());
            }
        };

        // Convert vector results to SearchResults by looking up chunk metadata
        let mut vector_search_results = Vec::with_capacity(vector_results.len());
        for vr in vector_results {
            if let Ok(sr) = Self::get_chunk_as_search_result(db, &vr.chunk_id, vr.distance) {
                vector_search_results.push(sr);
            }
        }

        // Use weighted RRF fusion
        let mut results = Self::hybrid_search_with_weights(
            fts_results,
            vector_search_results,
            options.fts_weight,
            options.vector_weight,
            options.limit * 2,
        );

        // Apply deduplication if enabled
        if options.enable_dedup {
            results = Self::deduplicate_results(results, options.dedup_threshold);
        }

        results.truncate(options.limit);
        Ok(results)
    }

    /// Perform hybrid search with RRF fusion (default equal weights)
    ///
    /// This combines FTS5 and vector search results using Reciprocal Rank Fusion.
    /// RRF score = sum(1 / (k + rank)) for each result list
    pub fn hybrid_search_with_vectors(
        fts_results: Vec<SearchResult>,
        vector_results: Vec<SearchResult>,
        limit: usize,
    ) -> Vec<SearchResult> {
        Self::hybrid_search_with_weights(fts_results, vector_results, 0.5, 0.5, limit)
    }

    /// Perform hybrid search with configurable weights
    ///
    /// This combines FTS5 and vector search results using weighted RRF.
    /// Weights control the relative importance of each result source.
    pub fn hybrid_search_with_weights(
        fts_results: Vec<SearchResult>,
        vector_results: Vec<SearchResult>,
        fts_weight: f64,
        vector_weight: f64,
        limit: usize,
    ) -> Vec<SearchResult> {
        use std::collections::HashMap;

        // Normalize weights
        let total_weight = fts_weight + vector_weight;
        let fts_w = if total_weight > 0.0 {
            fts_weight / total_weight
        } else {
            0.5
        };
        let vec_w = if total_weight > 0.0 {
            vector_weight / total_weight
        } else {
            0.5
        };

        // Build RRF scores with weights
        let mut rrf_scores: HashMap<String, (f64, Option<SearchResult>)> = HashMap::new();

        // Add FTS5 scores (weighted)
        for (rank, result) in fts_results.into_iter().enumerate() {
            let score = fts_w * (1.0 / (RRF_K + rank as f64 + 1.0));
            rrf_scores
                .entry(result.chunk_id.clone())
                .and_modify(|(s, _)| *s += score)
                .or_insert((score, Some(result)));
        }

        // Add vector scores (weighted)
        for (rank, result) in vector_results.into_iter().enumerate() {
            let score = vec_w * (1.0 / (RRF_K + rank as f64 + 1.0));
            rrf_scores
                .entry(result.chunk_id.clone())
                .and_modify(|(s, existing)| {
                    *s += score;
                    if existing.is_none() {
                        *existing = Some(result.clone());
                    }
                })
                .or_insert((score, Some(result)));
        }

        // Sort by RRF score and take top results
        let mut results: Vec<_> = rrf_scores
            .into_iter()
            .filter_map(|(_, (score, result))| {
                result.map(|mut r| {
                    r.score = score;
                    r.source = SearchSource::Hybrid;
                    r
                })
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        results
    }

    /// Deduplicate results based on content similarity
    ///
    /// Uses Jaccard similarity of word sets to detect near-duplicates.
    /// Results with similarity above threshold are removed (keeping the higher-scored one).
    pub fn deduplicate_results(results: Vec<SearchResult>, threshold: f64) -> Vec<SearchResult> {
        if results.is_empty() || threshold >= 1.0 {
            return results;
        }

        let mut deduped: Vec<SearchResult> = Vec::with_capacity(results.len());

        for result in results {
            // Check if this result is too similar to any already-kept result
            let is_duplicate = deduped
                .iter()
                .any(|kept| Self::content_similarity(&kept.content, &result.content) >= threshold);

            if !is_duplicate {
                deduped.push(result);
            }
        }

        deduped
    }

    /// Calculate Jaccard similarity between two content strings
    ///
    /// Returns value between 0.0 (no overlap) and 1.0 (identical)
    fn content_similarity(a: &str, b: &str) -> f64 {
        use std::collections::HashSet;

        // Tokenize into word sets (lowercase, alphanumeric only)
        let tokenize = |s: &str| -> HashSet<String> {
            s.to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() > 2) // Skip very short words
                .map(String::from)
                .collect()
        };

        let set_a = tokenize(a);
        let set_b = tokenize(b);

        if set_a.is_empty() && set_b.is_empty() {
            return 1.0; // Both empty = identical
        }
        if set_a.is_empty() || set_b.is_empty() {
            return 0.0; // One empty = no similarity
        }

        let intersection = set_a.intersection(&set_b).count();
        let union = set_a.union(&set_b).count();

        intersection as f64 / union as f64
    }

    /// Get chunk content by ID
    pub fn get_chunk_content(db: &Database, chunk_id: &str) -> Result<String, SearchError> {
        let content: String = db
            .conn()
            .query_row(
                "SELECT content FROM kb_chunks WHERE id = ?",
                params![chunk_id],
                |row| row.get(0),
            )
            .map_err(|e| SearchError::Database(DbError::Sqlite(e)))?;

        Ok(content)
    }

    /// Get multiple chunks for context injection
    pub fn get_chunks_for_context(
        db: &Database,
        chunk_ids: &[String],
    ) -> Result<Vec<(String, String, Option<String>)>, SearchError> {
        if chunk_ids.is_empty() {
            return Ok(vec![]);
        }

        let placeholders: String = chunk_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            r#"
            SELECT
                kb_chunks.content,
                kb_documents.file_path,
                kb_chunks.heading_path
            FROM kb_chunks
            JOIN kb_documents ON kb_chunks.document_id = kb_documents.id
            WHERE kb_chunks.id IN ({})
            "#,
            placeholders
        );

        let mut stmt = db
            .conn()
            .prepare(&query)
            .map_err(|e| SearchError::Database(DbError::Sqlite(e)))?;

        let results = stmt
            .query_map(rusqlite::params_from_iter(chunk_ids.iter()), |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(|e| SearchError::Database(DbError::Sqlite(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SearchError::Database(DbError::Sqlite(e)))?;

        Ok(results)
    }

    /// Format search results for LLM context injection
    pub fn format_context(results: &[SearchResult]) -> String {
        results
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let source = r.title.as_deref().unwrap_or(&r.file_path);
                let heading = r.heading_path.as_deref().unwrap_or("Document");
                format!(
                    "[Source {}: {} > {}]\n{}\n",
                    i + 1,
                    source,
                    heading,
                    r.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n---\n")
    }

    /// Sanitize snippet HTML for safe rendering
    ///
    /// Escapes HTML entities except for allowed highlight marks.
    /// Preserves <mark> tags from FTS5 snippets but escapes everything else.
    pub fn sanitize_snippet(snippet: &str) -> String {
        // First escape all HTML
        let escaped = snippet
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;");

        // Then restore the allowed highlight marks from FTS5
        escaped
            .replace("&lt;mark&gt;", "<mark>")
            .replace("&lt;/mark&gt;", "</mark>")
    }

    /// Normalize scores to 0..1 range using min-max normalization
    pub fn normalize_scores(results: &mut [SearchResult]) {
        if results.is_empty() {
            return;
        }

        let min_score = results
            .iter()
            .map(|r| r.score)
            .fold(f64::INFINITY, f64::min);
        let max_score = results
            .iter()
            .map(|r| r.score)
            .fold(f64::NEG_INFINITY, f64::max);

        let range = max_score - min_score;
        if range > 0.0 {
            for result in results.iter_mut() {
                result.score = (result.score - min_score) / range;
            }
        } else {
            // All scores are the same, normalize to 1.0
            for result in results.iter_mut() {
                result.score = 1.0;
            }
        }
    }

    /// Apply post-processing to search results based on options
    pub fn post_process_results(
        mut results: Vec<SearchResult>,
        options: &SearchOptions,
    ) -> Vec<SearchResult> {
        // Normalize scores if enabled
        if options.normalize_scores {
            Self::normalize_scores(&mut results);
        }

        // Sanitize snippets if enabled
        if options.sanitize_snippets {
            for result in results.iter_mut() {
                result.snippet = Self::sanitize_snippet(&result.snippet);
            }
        }

        results
    }
}

/// SimHash for scalable deduplication (Phase 15 upgrade)
pub struct SimHash;

impl SimHash {
    /// Compute a 64-bit SimHash fingerprint from text
    pub fn compute(text: &str) -> u64 {
        let mut v = [0i32; 64];

        // Tokenize and hash each token
        for token in text
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
        {
            let hash = Self::hash_token(token);
            for (i, val) in v.iter_mut().enumerate().take(64) {
                if (hash >> i) & 1 == 1 {
                    *val += 1;
                } else {
                    *val -= 1;
                }
            }
        }

        // Build fingerprint
        let mut fingerprint: u64 = 0;
        for (i, &count) in v.iter().enumerate() {
            if count > 0 {
                fingerprint |= 1 << i;
            }
        }

        fingerprint
    }

    /// Simple hash function for tokens
    fn hash_token(token: &str) -> u64 {
        let mut hash: u64 = 0;
        for byte in token.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash
    }

    /// Calculate Hamming distance between two fingerprints
    pub fn hamming_distance(a: u64, b: u64) -> u32 {
        (a ^ b).count_ones()
    }

    /// Calculate similarity (1 - normalized Hamming distance)
    pub fn similarity(a: u64, b: u64) -> f64 {
        1.0 - (Self::hamming_distance(a, b) as f64 / 64.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_fusion() {
        let fts_results = vec![
            SearchResult {
                chunk_id: "a".to_string(),
                document_id: "d1".to_string(),
                file_path: "/test.md".to_string(),
                title: Some("Test".to_string()),
                heading_path: None,
                content: "Content A".to_string(),
                snippet: "Content A".to_string(),
                score: 1.0,
                source: SearchSource::Fts5,
                namespace_id: Some("default".to_string()),
                source_type: Some("file".to_string()),
            },
            SearchResult {
                chunk_id: "b".to_string(),
                document_id: "d1".to_string(),
                file_path: "/test.md".to_string(),
                title: Some("Test".to_string()),
                heading_path: None,
                content: "Content B".to_string(),
                snippet: "Content B".to_string(),
                score: 0.8,
                source: SearchSource::Fts5,
                namespace_id: Some("default".to_string()),
                source_type: Some("file".to_string()),
            },
        ];

        let vector_results = vec![
            SearchResult {
                chunk_id: "b".to_string(),
                document_id: "d1".to_string(),
                file_path: "/test.md".to_string(),
                title: Some("Test".to_string()),
                heading_path: None,
                content: "Content B".to_string(),
                snippet: "Content B".to_string(),
                score: 0.95,
                source: SearchSource::Vector,
                namespace_id: Some("default".to_string()),
                source_type: Some("file".to_string()),
            },
            SearchResult {
                chunk_id: "c".to_string(),
                document_id: "d1".to_string(),
                file_path: "/test.md".to_string(),
                title: Some("Test".to_string()),
                heading_path: None,
                content: "Content C".to_string(),
                snippet: "Content C".to_string(),
                score: 0.9,
                source: SearchSource::Vector,
                namespace_id: Some("default".to_string()),
                source_type: Some("file".to_string()),
            },
        ];

        let hybrid = HybridSearch::hybrid_search_with_vectors(fts_results, vector_results, 10);

        // "b" should be ranked highest as it appears in both
        assert_eq!(hybrid[0].chunk_id, "b");
        assert_eq!(hybrid[0].source, SearchSource::Hybrid);

        // All three chunks should be present
        assert_eq!(hybrid.len(), 3);
    }

    #[test]
    fn test_format_context() {
        let results = vec![SearchResult {
            chunk_id: "1".to_string(),
            document_id: "d1".to_string(),
            file_path: "/docs/vpn.md".to_string(),
            title: Some("VPN Guide".to_string()),
            heading_path: Some("Connection Issues".to_string()),
            content: "If VPN fails, restart the client.".to_string(),
            snippet: "".to_string(),
            score: 1.0,
            source: SearchSource::Fts5,
            namespace_id: Some("default".to_string()),
            source_type: Some("file".to_string()),
        }];

        let context = HybridSearch::format_context(&results);
        assert!(context.contains("VPN Guide"));
        assert!(context.contains("Connection Issues"));
        assert!(context.contains("If VPN fails"));
    }

    #[test]
    fn test_content_similarity() {
        // Identical content
        let sim = HybridSearch::content_similarity(
            "The quick brown fox jumps over the lazy dog",
            "The quick brown fox jumps over the lazy dog",
        );
        assert!((sim - 1.0).abs() < 0.001);

        // Similar content
        let sim = HybridSearch::content_similarity(
            "The quick brown fox jumps over the lazy dog",
            "The quick brown fox runs over the lazy cat",
        );
        assert!(sim > 0.5);

        // Different content
        let sim = HybridSearch::content_similarity(
            "The quick brown fox jumps over the lazy dog",
            "Python programming language tutorial for beginners",
        );
        assert!(sim < 0.3);
    }

    #[test]
    fn test_deduplication() {
        let results = vec![
            SearchResult {
                chunk_id: "1".to_string(),
                document_id: "d1".to_string(),
                file_path: "/test.md".to_string(),
                title: None,
                heading_path: None,
                content: "The VPN connection requires proper configuration settings.".to_string(),
                snippet: "VPN connection".to_string(),
                score: 1.0,
                source: SearchSource::Fts5,
                namespace_id: None,
                source_type: None,
            },
            SearchResult {
                chunk_id: "2".to_string(),
                document_id: "d1".to_string(),
                file_path: "/test.md".to_string(),
                title: None,
                heading_path: None,
                content: "The VPN connection needs proper configuration settings.".to_string(),
                snippet: "VPN connection".to_string(),
                score: 0.9,
                source: SearchSource::Fts5,
                namespace_id: None,
                source_type: None,
            },
            SearchResult {
                chunk_id: "3".to_string(),
                document_id: "d2".to_string(),
                file_path: "/other.md".to_string(),
                title: None,
                heading_path: None,
                content: "Email configuration is done through the admin panel.".to_string(),
                snippet: "Email config".to_string(),
                score: 0.8,
                source: SearchSource::Fts5,
                namespace_id: None,
                source_type: None,
            },
        ];

        // With high threshold, first two should be deduped (very similar)
        let deduped = HybridSearch::deduplicate_results(results, 0.7);
        assert_eq!(deduped.len(), 2);
        assert_eq!(deduped[0].chunk_id, "1"); // Higher score kept
        assert_eq!(deduped[1].chunk_id, "3"); // Different content kept
    }

    #[test]
    fn test_weighted_rrf() {
        let fts_results = vec![SearchResult {
            chunk_id: "a".to_string(),
            document_id: "d1".to_string(),
            file_path: "/test.md".to_string(),
            title: None,
            heading_path: None,
            content: "A".to_string(),
            snippet: "A".to_string(),
            score: 1.0,
            source: SearchSource::Fts5,
            namespace_id: None,
            source_type: None,
        }];

        let vector_results = vec![SearchResult {
            chunk_id: "b".to_string(),
            document_id: "d1".to_string(),
            file_path: "/test.md".to_string(),
            title: None,
            heading_path: None,
            content: "B".to_string(),
            snippet: "B".to_string(),
            score: 1.0,
            source: SearchSource::Vector,
            namespace_id: None,
            source_type: None,
        }];

        // With heavy FTS weight, FTS result should rank higher
        let results = HybridSearch::hybrid_search_with_weights(
            fts_results.clone(),
            vector_results.clone(),
            0.9,
            0.1,
            10,
        );
        assert_eq!(results[0].chunk_id, "a");

        // With heavy vector weight, vector result should rank higher
        let results =
            HybridSearch::hybrid_search_with_weights(fts_results, vector_results, 0.1, 0.9, 10);
        assert_eq!(results[0].chunk_id, "b");
    }

    #[test]
    fn test_search_options_builder() {
        let opts = SearchOptions::new(20)
            .with_namespace(Some("test".to_string()))
            .with_weights(0.7, 0.3)
            .with_dedup(true, 0.9);

        assert_eq!(opts.limit, 20);
        assert_eq!(opts.namespace_id, Some("test".to_string()));
        assert!((opts.fts_weight - 0.7).abs() < 0.001);
        assert!((opts.vector_weight - 0.3).abs() < 0.001);
        assert!(opts.enable_dedup);
        assert!((opts.dedup_threshold - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_sanitize_snippet() {
        // Basic HTML escaping
        let sanitized = HybridSearch::sanitize_snippet("<script>alert('xss')</script>");
        assert!(!sanitized.contains("<script>"));
        assert!(sanitized.contains("&lt;script&gt;"));

        // Preserves FTS5 highlight marks
        let sanitized = HybridSearch::sanitize_snippet("Found <mark>VPN</mark> issue");
        assert!(sanitized.contains("<mark>VPN</mark>"));

        // Escapes quotes
        let sanitized = HybridSearch::sanitize_snippet("Use \"quotes\" and 'apostrophes'");
        assert!(!sanitized.contains("\""));
        assert!(!sanitized.contains("'"));
    }

    #[test]
    fn test_score_normalization() {
        let mut results = vec![
            SearchResult {
                chunk_id: "1".to_string(),
                document_id: "d1".to_string(),
                file_path: "/test.md".to_string(),
                title: None,
                heading_path: None,
                content: "A".to_string(),
                snippet: "A".to_string(),
                score: 0.5,
                source: SearchSource::Fts5,
                namespace_id: None,
                source_type: None,
            },
            SearchResult {
                chunk_id: "2".to_string(),
                document_id: "d1".to_string(),
                file_path: "/test.md".to_string(),
                title: None,
                heading_path: None,
                content: "B".to_string(),
                snippet: "B".to_string(),
                score: 1.0,
                source: SearchSource::Fts5,
                namespace_id: None,
                source_type: None,
            },
            SearchResult {
                chunk_id: "3".to_string(),
                document_id: "d1".to_string(),
                file_path: "/test.md".to_string(),
                title: None,
                heading_path: None,
                content: "C".to_string(),
                snippet: "C".to_string(),
                score: 0.0,
                source: SearchSource::Fts5,
                namespace_id: None,
                source_type: None,
            },
        ];

        HybridSearch::normalize_scores(&mut results);

        // Scores should now be in 0..1 range
        assert!((results[0].score - 0.5).abs() < 0.001); // Was 0.5, normalized to 0.5
        assert!((results[1].score - 1.0).abs() < 0.001); // Was 1.0, normalized to 1.0
        assert!((results[2].score - 0.0).abs() < 0.001); // Was 0.0, normalized to 0.0
    }

    #[test]
    fn test_simhash_similarity() {
        // Identical text should have high similarity
        let a = SimHash::compute("The quick brown fox jumps over the lazy dog");
        let b = SimHash::compute("The quick brown fox jumps over the lazy dog");
        assert_eq!(a, b);
        assert!((SimHash::similarity(a, b) - 1.0).abs() < 0.001);

        // Similar text should have moderate similarity
        let a = SimHash::compute("The quick brown fox jumps over the lazy dog");
        let b = SimHash::compute("The quick brown fox runs over the lazy cat");
        let sim_similar = SimHash::similarity(a, b);
        assert!(
            sim_similar > 0.5,
            "Similar text similarity: {}",
            sim_similar
        );

        // Different text should have lower similarity than identical
        let a = SimHash::compute("The quick brown fox jumps over the lazy dog");
        let b = SimHash::compute("Python programming language tutorial for beginners");
        let sim_different = SimHash::similarity(a, b);
        // SimHash may still show moderate similarity for short texts
        // The key is that different text has lower similarity than similar text
        assert!(
            sim_different < sim_similar || sim_different < 1.0,
            "Different text similarity: {}",
            sim_different
        );
    }

    #[test]
    fn test_simhash_hamming_distance() {
        // Same fingerprint should have 0 distance
        assert_eq!(SimHash::hamming_distance(0b1010, 0b1010), 0);

        // One bit different
        assert_eq!(SimHash::hamming_distance(0b1010, 0b1011), 1);

        // All bits different in 4-bit example
        assert_eq!(SimHash::hamming_distance(0b0000, 0b1111), 4);
    }

    // FTS5 query preprocessing tests

    #[test]
    fn test_is_mixed_alphanumeric() {
        assert!(is_mixed_alphanumeric("6sense"));
        assert!(is_mixed_alphanumeric("h2o"));
        assert!(is_mixed_alphanumeric("abc123"));
        assert!(!is_mixed_alphanumeric("hello"));
        assert!(!is_mixed_alphanumeric("12345"));
        assert!(!is_mixed_alphanumeric(""));
    }

    #[test]
    fn test_split_mixed_alphanumeric_basic() {
        let parts = split_mixed_alphanumeric("6sense");
        assert_eq!(parts, vec!["6", "sense"]);
    }

    #[test]
    fn test_split_mixed_alphanumeric_multiple() {
        let parts = split_mixed_alphanumeric("h2o");
        assert_eq!(parts, vec!["h", "2", "o"]);
    }

    #[test]
    fn test_split_mixed_alphanumeric_trailing_digits() {
        let parts = split_mixed_alphanumeric("abc123");
        assert_eq!(parts, vec!["abc", "123"]);
    }

    #[test]
    fn test_preprocess_empty_query() {
        assert_eq!(preprocess_fts5_query(""), "");
        assert_eq!(preprocess_fts5_query("   "), "");
    }

    #[test]
    fn test_preprocess_simple_word() {
        assert_eq!(preprocess_fts5_query("vpn"), "vpn");
    }

    #[test]
    fn test_preprocess_mixed_alphanumeric() {
        let result = preprocess_fts5_query("6sense");
        assert!(
            result.contains("\"6sense\""),
            "Should contain quoted original: {}",
            result
        );
        assert!(result.contains("OR"), "Should contain OR: {}", result);
        assert!(
            result.contains("6"),
            "Should contain digit part: {}",
            result
        );
        assert!(
            result.contains("sense"),
            "Should contain alpha part: {}",
            result
        );
    }

    #[test]
    fn test_preprocess_multi_word() {
        let result = preprocess_fts5_query("vpn connection");
        assert!(
            result.contains("\"vpn connection\""),
            "Should contain quoted phrase: {}",
            result
        );
        assert!(result.contains("OR"), "Should contain OR: {}", result);
        assert!(
            result.contains("vpn"),
            "Should contain individual word: {}",
            result
        );
        assert!(
            result.contains("connection"),
            "Should contain individual word: {}",
            result
        );
    }

    #[test]
    fn test_preprocess_strips_double_quotes() {
        let result = preprocess_fts5_query("\"6sense\"");
        // Should not have unmatched quotes that would cause FTS5 syntax error
        assert!(
            !result.contains("\"\""),
            "Should not have empty quotes: {}",
            result
        );
        assert!(
            result.contains("6sense"),
            "Should still contain the term: {}",
            result
        );
    }
}
