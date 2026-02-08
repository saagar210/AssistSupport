use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::audit::{self, AuditAction};
use crate::error::AppError;
use crate::ollama;
use crate::state::{get_conn, AppState};
use crate::utils::{bytes_to_f64_vec, cosine_similarity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryEntry {
    pub id: String,
    pub collection_id: String,
    pub query: String,
    pub result_count: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk_id: String,
    pub document_id: String,
    pub document_title: String,
    pub section_title: Option<String>,
    pub page_number: Option<i32>,
    pub content: String,
    pub score: f64,
}

/// Perform vector (semantic) search: embed the query, search vector store,
/// then enrich results with chunk/document metadata.
#[tauri::command]
pub async fn vector_search(
    state: State<'_, AppState>,
    collection_id: String,
    query: String,
    top_k: usize,
) -> Result<Vec<SearchResult>, AppError> {
    // 1. Read ollama settings inside connection scope, then drop
    let (host, port, embedding_model) = {
        let conn = get_conn(state.inner())?;
        let host: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_host'",
            [],
            |row| row.get(0),
        )?;
        let port: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_port'",
            [],
            |row| row.get(0),
        )?;
        let embedding_model: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'embedding_model'",
            [],
            |row| row.get(0),
        )?;
        (host, port, embedding_model)
    };

    // 2. Generate query embedding (async, no lock held)
    let query_embedding =
        ollama::generate_embedding(&host, &port, &embedding_model, &query).await?;

    // 3. Load all chunk embeddings for the collection from the DB,
    //    compute cosine similarity in Rust, and return top_k.
    //    NOTE: The other agent is building a vector_store module. For now we do
    //    an in-process brute-force search so this module compiles independently.
    let results = {
        let conn = get_conn(state.inner())?;
        vector_search_in_db(&conn, &collection_id, &query_embedding, top_k)?
    };

    Ok(results)
}

/// In-DB vector search using brute-force cosine similarity.
/// Reads chunk_embeddings rows, computes similarity, joins chunk + document metadata.
/// This will be replaced by vector_store::search_vectors once that module lands.
/// Alias for chat module to call with a pre-computed embedding.
pub(crate) fn vector_search_in_db_with_embedding(
    conn: &rusqlite::Connection,
    collection_id: &str,
    query_embedding: &[f64],
    top_k: usize,
) -> Result<Vec<SearchResult>, AppError> {
    vector_search_in_db(conn, collection_id, query_embedding, top_k)
}

fn vector_search_in_db(
    conn: &rusqlite::Connection,
    collection_id: &str,
    query_embedding: &[f64],
    top_k: usize,
) -> Result<Vec<SearchResult>, AppError> {
    // Check if chunk_embeddings table exists (other agent may not have created it yet)
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='chunk_embeddings'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)?;

    if !table_exists {
        return Ok(Vec::new());
    }

    let mut stmt = conn.prepare(
        "SELECT ce.chunk_id, ce.embedding
         FROM chunk_embeddings ce
         JOIN chunks c ON c.id = ce.chunk_id
         WHERE c.collection_id = ?1",
    )?;

    let rows = stmt.query_map(rusqlite::params![collection_id], |row| {
        let chunk_id: String = row.get(0)?;
        let embedding_blob: Vec<u8> = row.get(1)?;
        Ok((chunk_id, embedding_blob))
    })?;

    // Compute cosine similarity for each chunk
    let mut scored: Vec<(String, f64)> = Vec::new();
    for row_result in rows {
        let (chunk_id, blob) = row_result?;
        if blob.len() % 8 != 0 {
            tracing::warn!("Invalid embedding blob length {} for chunk {}, skipping", blob.len(), chunk_id);
            continue;
        }
        let embedding = bytes_to_f64_vec(&blob);
        let sim = cosine_similarity(query_embedding, &embedding);
        scored.push((chunk_id, sim));
    }

    // Sort descending by score
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);

    // Enrich with chunk/document metadata
    let mut results = Vec::with_capacity(scored.len());
    for (chunk_id, score) in scored {
        let result = conn.query_row(
            "SELECT c.id, c.document_id, c.content, c.section_title, c.page_number,
                    d.title
             FROM chunks c
             JOIN documents d ON d.id = c.document_id
             WHERE c.id = ?1",
            rusqlite::params![chunk_id],
            |row| {
                Ok(SearchResult {
                    chunk_id: row.get(0)?,
                    document_id: row.get(1)?,
                    content: row.get(2)?,
                    section_title: row.get(3)?,
                    page_number: row.get(4)?,
                    document_title: row.get(5)?,
                    score,
                })
            },
        );

        match result {
            Ok(sr) => results.push(sr),
            Err(rusqlite::Error::QueryReturnedNoRows) => continue,
            Err(e) => return Err(AppError::Database(e)),
        }
    }

    Ok(results)
}

/// Full-text keyword search using FTS5 with BM25 ranking.
#[tauri::command]
pub fn keyword_search(
    state: State<'_, AppState>,
    collection_id: String,
    query: String,
    top_k: usize,
) -> Result<Vec<SearchResult>, AppError> {
    let conn = get_conn(state.inner())?;
    keyword_search_in_db(&conn, &collection_id, &query, top_k)
}

pub(crate) fn keyword_search_in_db(
    conn: &rusqlite::Connection,
    collection_id: &str,
    query: &str,
    top_k: usize,
) -> Result<Vec<SearchResult>, AppError> {
    // FTS5 query — filter by collection_id, rank by BM25
    let mut stmt = conn.prepare(
        "SELECT f.chunk_id, f.document_id, f.content, bm25(chunks_fts) AS rank
         FROM chunks_fts f
         WHERE chunks_fts MATCH ?1
           AND f.collection_id = ?2
         ORDER BY rank ASC
         LIMIT ?3",
    )?;

    let rows = stmt.query_map(rusqlite::params![query, collection_id, top_k as i64], |row| {
        let chunk_id: String = row.get(0)?;
        let document_id: String = row.get(1)?;
        let content: String = row.get(2)?;
        let bm25_rank: f64 = row.get(3)?;
        Ok((chunk_id, document_id, content, bm25_rank))
    })?;

    let mut results = Vec::new();
    for row_result in rows {
        let (chunk_id, document_id, content, bm25_rank) = row_result?;

        // BM25 returns negative values where more negative = better match.
        // Convert to a positive score (negate it).
        let score = -bm25_rank;

        // Get section_title, page_number from chunks table and title from documents
        let meta = conn.query_row(
            "SELECT c.section_title, c.page_number, d.title
             FROM chunks c
             JOIN documents d ON d.id = c.document_id
             WHERE c.id = ?1",
            rusqlite::params![chunk_id],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<i32>>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        );

        match meta {
            Ok((section_title, page_number, document_title)) => {
                results.push(SearchResult {
                    chunk_id,
                    document_id,
                    document_title,
                    section_title,
                    page_number,
                    content,
                    score,
                });
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => continue,
            Err(e) => return Err(AppError::Database(e)),
        }
    }

    Ok(results)
}

/// Hybrid search combining vector and keyword search with Reciprocal Rank Fusion (RRF).
#[tauri::command]
pub async fn hybrid_search(
    state: State<'_, AppState>,
    collection_id: String,
    query: String,
    top_k: usize,
) -> Result<Vec<SearchResult>, AppError> {
    // Read settings
    let (host, port, embedding_model, rrf_k, vector_top_k, keyword_top_k) = {
        let conn = get_conn(state.inner())?;
        let host: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_host'",
            [],
            |row| row.get(0),
        )?;
        let port: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_port'",
            [],
            |row| row.get(0),
        )?;
        let embedding_model: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'embedding_model'",
            [],
            |row| row.get(0),
        )?;
        let rrf_k: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'rrf_k'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "60".to_string());
        let vector_top_k: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'vector_top_k'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "20".to_string());
        let keyword_top_k: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'keyword_top_k'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "20".to_string());
        (host, port, embedding_model, rrf_k, vector_top_k, keyword_top_k)
    };

    let rrf_k_val: f64 = rrf_k.parse().unwrap_or(60.0);
    let vec_top_k: usize = vector_top_k.parse().unwrap_or(20);
    let kw_top_k: usize = keyword_top_k.parse().unwrap_or(20);

    // Generate query embedding (async, no lock held)
    let query_embedding =
        ollama::generate_embedding(&host, &port, &embedding_model, &query).await?;

    // Run both searches under a single connection acquisition
    let (vector_results, keyword_results) = {
        let conn = get_conn(state.inner())?;
        let vr = vector_search_in_db(&conn, &collection_id, &query_embedding, vec_top_k)?;
        let kr = keyword_search_in_db(&conn, &collection_id, &query, kw_top_k)?;
        (vr, kr)
    };

    // Apply Reciprocal Rank Fusion
    let fused = reciprocal_rank_fusion(vector_results, keyword_results, rrf_k_val, top_k);
    Ok(fused)
}

/// Public wrapper for chat module.
pub(crate) fn reciprocal_rank_fusion_pub(
    vector_results: Vec<SearchResult>,
    keyword_results: Vec<SearchResult>,
    k: f64,
    top_k: usize,
) -> Vec<SearchResult> {
    reciprocal_rank_fusion(vector_results, keyword_results, k, top_k)
}

// --- Search History Commands ---

#[tauri::command]
pub fn save_search_query(
    state: State<'_, AppState>,
    collection_id: String,
    query: String,
    result_count: i32,
) -> Result<(), AppError> {
    let conn = get_conn(state.inner())?;
    let now = chrono::Utc::now().to_rfc3339();

    // Upsert: if same query exists for collection, update timestamp + result_count
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM search_history WHERE collection_id = ?1 AND query = ?2",
            rusqlite::params![collection_id, query],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing {
        conn.execute(
            "UPDATE search_history SET result_count = ?1, created_at = ?2 WHERE id = ?3",
            rusqlite::params![result_count, now, id],
        )?;
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO search_history (id, collection_id, query, result_count, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, collection_id, query, result_count, now],
        )?;
    }

    // Add audit logging
    let _ = audit::log_audit(
        &conn,
        AuditAction::SearchExecute,
        Some("search"),
        None,
        &serde_json::json!({
            "query": query,
            "result_count": result_count
        }),
    );

    Ok(())
}

#[tauri::command]
pub fn get_search_history(
    state: State<'_, AppState>,
    collection_id: String,
    limit: Option<i32>,
) -> Result<Vec<SearchHistoryEntry>, AppError> {
    let conn = get_conn(state.inner())?;
    let limit = limit.unwrap_or(10);

    let mut stmt = conn.prepare(
        "SELECT id, collection_id, query, result_count, created_at FROM search_history WHERE collection_id = ?1 ORDER BY created_at DESC LIMIT ?2",
    )?;

    let entries = stmt
        .query_map(rusqlite::params![collection_id, limit], |row| {
            Ok(SearchHistoryEntry {
                id: row.get(0)?,
                collection_id: row.get(1)?,
                query: row.get(2)?,
                result_count: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(entries)
}

#[tauri::command]
pub fn clear_search_history(
    state: State<'_, AppState>,
    collection_id: String,
) -> Result<(), AppError> {
    let conn = get_conn(state.inner())?;
    conn.execute(
        "DELETE FROM search_history WHERE collection_id = ?1",
        rusqlite::params![collection_id],
    )?;
    Ok(())
}

// --- Find Similar Chunks ---

#[tauri::command]
pub fn find_similar_chunks(
    state: State<'_, AppState>,
    chunk_id: String,
    collection_id: String,
    top_k: Option<usize>,
) -> Result<Vec<SearchResult>, AppError> {
    let conn = get_conn(state.inner())?;
    let top_k = top_k.unwrap_or(10);

    // Load the source chunk's embedding
    let source_blob: Vec<u8> = conn
        .query_row(
            "SELECT embedding FROM chunk_embeddings WHERE chunk_id = ?1",
            rusqlite::params![chunk_id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("Embedding not found for chunk '{}'", chunk_id))
            }
            other => AppError::Database(other),
        })?;

    let source_embedding = bytes_to_f64_vec(&source_blob);

    // Get the source chunk's document_id to exclude it
    let source_doc_id: String = conn.query_row(
        "SELECT document_id FROM chunks WHERE id = ?1",
        rusqlite::params![chunk_id],
        |row| row.get(0),
    )?;

    // Search all embeddings in collection, excluding the source document
    let mut stmt = conn.prepare(
        "SELECT ce.chunk_id, ce.embedding
         FROM chunk_embeddings ce
         JOIN chunks c ON c.id = ce.chunk_id
         WHERE c.collection_id = ?1 AND c.document_id != ?2",
    )?;

    let rows = stmt.query_map(rusqlite::params![collection_id, source_doc_id], |row| {
        let cid: String = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        Ok((cid, blob))
    })?;

    let mut scored: Vec<(String, f64)> = Vec::new();
    for row_result in rows {
        let (cid, blob) = row_result?;
        if blob.len() % 8 != 0 {
            continue;
        }
        let embedding = bytes_to_f64_vec(&blob);
        let sim = cosine_similarity(&source_embedding, &embedding);
        scored.push((cid, sim));
    }

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);

    // Enrich results
    let mut results = Vec::with_capacity(scored.len());
    for (cid, score) in scored {
        let result = conn.query_row(
            "SELECT c.id, c.document_id, c.content, c.section_title, c.page_number, d.title
             FROM chunks c JOIN documents d ON d.id = c.document_id WHERE c.id = ?1",
            rusqlite::params![cid],
            |row| {
                Ok(SearchResult {
                    chunk_id: row.get(0)?,
                    document_id: row.get(1)?,
                    content: row.get(2)?,
                    section_title: row.get(3)?,
                    page_number: row.get(4)?,
                    document_title: row.get(5)?,
                    score,
                })
            },
        );

        match result {
            Ok(sr) => results.push(sr),
            Err(rusqlite::Error::QueryReturnedNoRows) => continue,
            Err(e) => return Err(AppError::Database(e)),
        }
    }

    Ok(results)
}

/// Reciprocal Rank Fusion: merge two ranked result lists.
fn reciprocal_rank_fusion(
    vector_results: Vec<SearchResult>,
    keyword_results: Vec<SearchResult>,
    k: f64,
    top_k: usize,
) -> Vec<SearchResult> {
    // Map chunk_id -> (rrf_score, SearchResult)
    let mut scores: HashMap<String, (f64, SearchResult)> = HashMap::new();

    for (rank, result) in vector_results.into_iter().enumerate() {
        let rrf_score = 1.0 / (k + (rank as f64) + 1.0);
        scores
            .entry(result.chunk_id.clone())
            .and_modify(|(s, _)| *s += rrf_score)
            .or_insert((rrf_score, result));
    }

    for (rank, result) in keyword_results.into_iter().enumerate() {
        let rrf_score = 1.0 / (k + (rank as f64) + 1.0);
        scores
            .entry(result.chunk_id.clone())
            .and_modify(|(s, _)| *s += rrf_score)
            .or_insert((rrf_score, result));
    }

    let mut fused: Vec<(f64, SearchResult)> = scores.into_values().collect();
    fused.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    fused.truncate(top_k);

    fused
        .into_iter()
        .map(|(score, mut result)| {
            result.score = score;
            result
        })
        .collect()
}
