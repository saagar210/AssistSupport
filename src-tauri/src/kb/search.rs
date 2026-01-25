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
}

/// Source of the search result
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum SearchSource {
    Fts5,
    Vector,
    Hybrid,
}

/// RRF (Reciprocal Rank Fusion) constant
const RRF_K: f64 = 60.0;

/// Hybrid search engine
pub struct HybridSearch;

impl HybridSearch {
    /// Perform FTS5-only search (sync version)
    pub fn search(
        db: &Database,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let fts_results = Self::fts_search(db, query, limit)?;
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
        // Get FTS5 results (fetch more for fusion)
        let fts_results = Self::fts_search(db, query, limit * 2)?;

        // If no vector results, just return FTS5
        let vector_results = match vector_results {
            Some(vr) if !vr.is_empty() => vr,
            _ => return Ok(fts_results.into_iter().take(limit).collect()),
        };

        // Convert vector results to SearchResults by looking up chunk metadata
        let mut vector_search_results = Vec::with_capacity(vector_results.len());
        for vr in vector_results {
            if let Ok(sr) = Self::get_chunk_as_search_result(db, &vr.chunk_id, vr.distance) {
                vector_search_results.push(sr);
            }
        }

        // Use RRF fusion
        Ok(Self::hybrid_search_with_vectors(fts_results, vector_search_results, limit))
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
                kb_chunks.content
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
                })
            },
        ).map_err(|e| SearchError::Database(DbError::Sqlite(e)))
    }

    /// Perform FTS5 keyword search
    pub fn fts_search(
        db: &Database,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let conn = db.conn();

        let mut stmt = conn.prepare(
            r#"
            SELECT
                kb_chunks.id,
                kb_chunks.document_id,
                kb_documents.file_path,
                kb_documents.title,
                kb_chunks.heading_path,
                kb_chunks.content,
                snippet(kb_fts, 0, '<mark>', '</mark>', '...', 32) as snippet,
                bm25(kb_fts) as rank
            FROM kb_fts
            JOIN kb_chunks ON kb_fts.rowid = kb_chunks.rowid
            JOIN kb_documents ON kb_chunks.document_id = kb_documents.id
            WHERE kb_fts MATCH ?
            ORDER BY rank
            LIMIT ?
            "#,
        ).map_err(|e| SearchError::Database(DbError::Sqlite(e)))?;

        let results = stmt
            .query_map(params![query, limit as i64], |row| {
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
                })
            })
            .map_err(|e| SearchError::Database(DbError::Sqlite(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SearchError::Database(DbError::Sqlite(e)))?;

        Ok(results)
    }

    /// Perform hybrid search with RRF fusion
    ///
    /// This combines FTS5 and vector search results using Reciprocal Rank Fusion.
    /// RRF score = sum(1 / (k + rank)) for each result list
    pub fn hybrid_search_with_vectors(
        fts_results: Vec<SearchResult>,
        vector_results: Vec<SearchResult>,
        limit: usize,
    ) -> Vec<SearchResult> {
        use std::collections::HashMap;

        // Build RRF scores
        let mut rrf_scores: HashMap<String, (f64, Option<SearchResult>)> = HashMap::new();

        // Add FTS5 scores
        for (rank, result) in fts_results.into_iter().enumerate() {
            let score = 1.0 / (RRF_K + rank as f64 + 1.0);
            rrf_scores.entry(result.chunk_id.clone())
                .and_modify(|(s, _)| *s += score)
                .or_insert((score, Some(result)));
        }

        // Add vector scores
        for (rank, result) in vector_results.into_iter().enumerate() {
            let score = 1.0 / (RRF_K + rank as f64 + 1.0);
            rrf_scores.entry(result.chunk_id.clone())
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

    /// Get chunk content by ID
    pub fn get_chunk_content(db: &Database, chunk_id: &str) -> Result<String, SearchError> {
        let content: String = db.conn()
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

        let placeholders: String = chunk_ids.iter()
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

        let mut stmt = db.conn().prepare(&query)
            .map_err(|e| SearchError::Database(DbError::Sqlite(e)))?;

        let results = stmt
            .query_map(
                rusqlite::params_from_iter(chunk_ids.iter()),
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
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
        let results = vec![
            SearchResult {
                chunk_id: "1".to_string(),
                document_id: "d1".to_string(),
                file_path: "/docs/vpn.md".to_string(),
                title: Some("VPN Guide".to_string()),
                heading_path: Some("Connection Issues".to_string()),
                content: "If VPN fails, restart the client.".to_string(),
                snippet: "".to_string(),
                score: 1.0,
                source: SearchSource::Fts5,
            },
        ];

        let context = HybridSearch::format_context(&results);
        assert!(context.contains("VPN Guide"));
        assert!(context.contains("Connection Issues"));
        assert!(context.contains("If VPN fails"));
    }
}
