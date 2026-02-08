use std::path::Path;

use sha2::{Digest, Sha256};
use tauri::{Emitter, Manager};

use crate::audit::{self, AuditAction};
use crate::chunker;
use crate::embedder;
use crate::error::AppError;
use crate::metrics::MetricCounter;
use crate::models::{Chunk, Document};
use crate::parsers;
use crate::state::{get_conn, AppState};
use crate::vector_store;

fn get_conn_arc(
    state: &AppState,
) -> Result<r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>, AppError> {
    crate::state::get_conn(state)
}

fn detect_file_type(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

fn compute_sha256(path: &Path) -> Result<String, AppError> {
    let bytes = std::fs::read(path)?;
    let hash = Sha256::digest(&bytes);
    Ok(format!("{:x}", hash))
}

fn get_setting(db: &rusqlite::Connection, key: &str, default: &str) -> String {
    db.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        rusqlite::params![key],
        |row: &rusqlite::Row| row.get(0),
    )
    .unwrap_or_else(|_| default.to_string())
}

fn emit_progress(
    app: &tauri::AppHandle,
    doc_id: &str,
    filename: &str,
    stage: &str,
    chunks_done: usize,
    chunks_total: usize,
    error: Option<&str>,
) {
    let mut payload = serde_json::json!({
        "document_id": doc_id,
        "filename": filename,
        "stage": stage,
        "chunks_done": chunks_done,
        "chunks_total": chunks_total,
    });
    if let Some(err) = error {
        payload["error"] = serde_json::Value::String(err.to_string());
    }
    let _ = app.emit("ingestion-progress", payload);
}

/// Internal ingestion logic for a single file. Used by both ingest_files and reingest.
async fn ingest_single_file(
    app: &tauri::AppHandle,
    app_state: &AppState,
    collection_id: &str,
    doc_id: &str,
    file_path_str: &str,
    filename: &str,
    file_type: &str,
) -> Result<(), AppError> {
    let path = Path::new(file_path_str);

    // Stage: parsing
    emit_progress(app, doc_id, filename, "parsing", 0, 0, None);
    let parsed = match parsers::parse_document(path, file_type) {
        Ok(p) => p,
        Err(e) => {
            let conn = get_conn_arc(app_state)?;
            let now = chrono::Utc::now().to_rfc3339();
            let _ = conn.execute(
                "UPDATE documents SET status = 'failed', error_message = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![e.to_string(), now, doc_id],
            );
            emit_progress(app, doc_id, filename, "failed", 0, 0, Some(&e.to_string()));
            return Err(e);
        }
    };

    // Read settings
    let (chunk_size, chunk_overlap, ollama_host, ollama_port, embedding_model) = {
        let conn = get_conn_arc(app_state)?;
        (
            get_setting(&conn, "chunk_size", "512").parse::<usize>().unwrap_or(512),
            get_setting(&conn, "chunk_overlap", "64").parse::<usize>().unwrap_or(64),
            get_setting(&conn, "ollama_host", "localhost"),
            get_setting(&conn, "ollama_port", "11434"),
            get_setting(&conn, "embedding_model", "nomic-embed-text"),
        )
    };

    // Stage: chunking
    emit_progress(app, doc_id, filename, "chunking", 0, 0, None);
    let chunks = chunker::chunk_text(
        &parsed.text,
        &parsed.sections,
        chunk_size,
        chunk_overlap,
    );
    let chunks_total = chunks.len();

    // Insert chunks into DB
    {
        let conn = get_conn_arc(app_state)?;
        let now = chrono::Utc::now().to_rfc3339();

        for chunk in &chunks {
            let chunk_id = uuid::Uuid::new_v4().to_string();

            conn.execute(
                "INSERT INTO chunks (id, document_id, collection_id, content, chunk_index, start_offset, end_offset, page_number, section_title, token_count, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, ?8, ?9, ?10)",
                rusqlite::params![
                    chunk_id,
                    doc_id,
                    collection_id,
                    chunk.content,
                    chunk.chunk_index,
                    chunk.start_offset,
                    chunk.end_offset,
                    chunk.section_title,
                    chunk.token_count,
                    now,
                ],
            )?;

            conn.execute(
                "INSERT INTO chunks_fts (content, chunk_id, document_id, collection_id) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![chunk.content, chunk_id, doc_id, collection_id],
            )?;
        }
    }

    // Stage: embedding
    emit_progress(app, doc_id, filename, "embedding", 0, chunks_total, None);
    let progress_ctx = embedder::ProgressCtx {
        app_handle: app.clone(),
        document_id: doc_id.to_string(),
        filename: filename.to_string(),
    };

    let embeddings = match embedder::embed_chunks(
        &ollama_host,
        &ollama_port,
        &embedding_model,
        &chunks,
        Some(progress_ctx),
    )
    .await
    {
        Ok(e) => e,
        Err(e) => {
            let conn = get_conn_arc(app_state)?;
            let now = chrono::Utc::now().to_rfc3339();
            let _ = conn.execute(
                "UPDATE documents SET status = 'failed', error_message = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![e.to_string(), now, doc_id],
            );
            emit_progress(app, doc_id, filename, "failed", 0, chunks_total, Some(&e.to_string()));
            return Err(e);
        }
    };

    // Stage: indexing
    emit_progress(app, doc_id, filename, "indexing", chunks_total, chunks_total, None);
    {
        let conn = get_conn_arc(app_state)?;

        let mut stmt = conn.prepare(
            "SELECT id, content FROM chunks WHERE document_id = ?1 ORDER BY chunk_index ASC",
        )?;
        let chunk_rows: Vec<(String, String)> = stmt
            .query_map(rusqlite::params![doc_id], |row: &rusqlite::Row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut embedding_data: Vec<(String, String, String, Vec<f64>, String)> = Vec::new();
        for (i, (chunk_id, content)) in chunk_rows.iter().enumerate() {
            if let Some(embedding) = embeddings.get(i) {
                let preview = if content.chars().count() > 200 {
                    format!("{}...", content.chars().take(200).collect::<String>())
                } else {
                    content.clone()
                };
                embedding_data.push((
                    chunk_id.clone(),
                    collection_id.to_string(),
                    doc_id.to_string(),
                    embedding.clone(),
                    preview,
                ));
            }
        }

        vector_store::store_embeddings(&conn, &embedding_data)?;

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE documents SET status = 'completed', word_count = ?1, chunk_count = ?2, title = ?3, author = ?4, page_count = ?5, updated_at = ?6 WHERE id = ?7",
            rusqlite::params![
                parsed.metadata.word_count,
                chunks.len() as i32,
                parsed.metadata.title.as_deref().unwrap_or(filename),
                parsed.metadata.author,
                parsed.metadata.page_count,
                now,
                doc_id,
            ],
        )?;
    }

    // Stage: complete
    emit_progress(app, doc_id, filename, "complete", chunks_total, chunks_total, None);

    // Track metrics
    app_state.metrics.increment(MetricCounter::DocumentsIngested);
    app_state.metrics.increment_by(MetricCounter::ChunksCreated, chunks_total as u64);

    Ok(())
}

/// Clear all chunks, FTS entries, embeddings, and graph edges for a document.
fn clear_document_data(db: &rusqlite::Connection, doc_id: &str) -> Result<(), AppError> {
    vector_store::delete_document_vectors(db, doc_id)?;
    db.execute(
        "DELETE FROM chunks_fts WHERE document_id = ?1",
        rusqlite::params![doc_id],
    )?;
    db.execute(
        "DELETE FROM graph_edges WHERE source_chunk_id IN (SELECT id FROM chunks WHERE document_id = ?1) OR target_chunk_id IN (SELECT id FROM chunks WHERE document_id = ?1)",
        rusqlite::params![doc_id],
    )?;
    db.execute(
        "DELETE FROM chunks WHERE document_id = ?1",
        rusqlite::params![doc_id],
    )?;
    Ok(())
}

#[tauri::command]
pub async fn ingest_files(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    collection_id: String,
    file_paths: Vec<String>,
) -> Result<Vec<String>, AppError> {
    // Gather document IDs synchronously, then spawn background work
    let mut doc_entries: Vec<(String, String, String, String)> = Vec::new(); // (doc_id, path, filename, file_type)

    for file_path_str in &file_paths {
        let path = Path::new(file_path_str);

        let file_type = match detect_file_type(path) {
            Some(ft) => ft,
            None => {
                tracing::warn!("Skipping file with unknown extension: {}", file_path_str);
                continue;
            }
        };

        let file_hash = match compute_sha256(path) {
            Ok(h) => h,
            Err(e) => {
                tracing::error!("Failed to hash file {}: {}", file_path_str, e);
                continue;
            }
        };

        let file_metadata = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Failed to read metadata for {}: {}", file_path_str, e);
                continue;
            }
        };

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let doc_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        {
            let conn = get_conn(state.inner())?;

            // Check duplicate hash
            let existing: Option<String> = conn
                .query_row(
                    "SELECT id FROM documents WHERE file_hash = ?1 AND collection_id = ?2",
                    rusqlite::params![file_hash, collection_id],
                    |row: &rusqlite::Row| row.get(0),
                )
                .ok();

            if existing.is_some() {
                tracing::info!("Skipping duplicate file: {}", file_path_str);
                continue;
            }

            conn.execute(
                "INSERT INTO documents (id, collection_id, filename, file_path, file_type, file_size, file_hash, title, author, page_count, word_count, chunk_count, status, error_message, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, NULL, 0, 0, 'processing', NULL, ?9, ?10)",
                rusqlite::params![
                    doc_id,
                    collection_id,
                    filename,
                    file_path_str,
                    file_type,
                    file_metadata.len() as i64,
                    file_hash,
                    filename,
                    now,
                    now,
                ],
            )?;

            let _ = audit::log_audit(&conn, AuditAction::DocumentIngest, Some("document"), Some(&doc_id), &serde_json::json!({"filename": filename}));
        }

        doc_entries.push((doc_id, file_path_str.clone(), filename, file_type));
    }

    let created_ids: Vec<String> = doc_entries.iter().map(|(id, _, _, _)| id.clone()).collect();

    // Fire-and-forget: spawn background processing for all docs
    let app = app_handle.clone();
    let cid = collection_id.clone();
    tauri::async_runtime::spawn(async move {
        let app_state: tauri::State<'_, AppState> = app.state();
        for (doc_id, file_path_str, filename, file_type) in &doc_entries {
            if let Err(e) = ingest_single_file(
                &app,
                app_state.inner(),
                &cid,
                doc_id,
                file_path_str,
                filename,
                file_type,
            )
            .await
            {
                tracing::error!("Ingestion failed for {}: {}", filename, e);
            }
        }
        // Rebuild HNSW index after ingestion
        if let Ok(conn) = crate::state::get_conn(app_state.inner()) {
            if let Ok(mut index) = app_state.inner().vector_index.write() {
                let _ = index.rebuild_collection_index(&conn, &cid);
            }
        }
        // Signal all files done
        let _ = app.emit("ingestion-all-complete", serde_json::json!({ "collection_id": cid }));
    });

    Ok(created_ids)
}

#[tauri::command]
pub async fn reingest_document(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    document_id: String,
) -> Result<(), AppError> {
    // Load document info
    let (collection_id, file_path, filename, file_type) = {
        let conn = get_conn(state.inner())?;
        let row = conn.query_row(
            "SELECT collection_id, file_path, filename, file_type FROM documents WHERE id = ?1",
            rusqlite::params![document_id],
            |row: &rusqlite::Row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("Document '{}' not found", document_id))
            }
            other => AppError::Database(other),
        })?;

        // Clear old data
        clear_document_data(&conn, &document_id)?;

        // Reset status
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE documents SET status = 'processing', error_message = NULL, word_count = 0, chunk_count = 0, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, document_id],
        )?;

        let _ = audit::log_audit(&conn, AuditAction::DocumentReingest, Some("document"), Some(&document_id), &serde_json::json!({}));

        row
    };

    let app = app_handle.clone();
    let did = document_id.clone();
    let cid_for_rebuild = collection_id.clone();
    tauri::async_runtime::spawn(async move {
        let app_state: tauri::State<'_, AppState> = app.state();
        if let Err(e) = ingest_single_file(
            &app,
            app_state.inner(),
            &collection_id,
            &did,
            &file_path,
            &filename,
            &file_type,
        )
        .await
        {
            tracing::error!("Re-ingestion failed for {}: {}", filename, e);
        }
        // Rebuild HNSW index after re-ingestion
        if let Ok(conn) = crate::state::get_conn(app_state.inner()) {
            if let Ok(mut index) = app_state.inner().vector_index.write() {
                let _ = index.rebuild_collection_index(&conn, &cid_for_rebuild);
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn reingest_collection(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    collection_id: String,
) -> Result<(), AppError> {
    // Load all documents for collection
    let docs: Vec<(String, String, String, String)> = {
        let conn = get_conn(state.inner())?;
        let mut stmt = conn.prepare(
            "SELECT id, file_path, filename, file_type FROM documents WHERE collection_id = ?1 AND status != 'failed'",
        )?;
        let rows = stmt.query_map(rusqlite::params![collection_id], |row: &rusqlite::Row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let total_docs = docs.len();

    // Clear and reset all documents
    {
        let conn = get_conn(state.inner())?;
        let now = chrono::Utc::now().to_rfc3339();
        for (doc_id, _, _, _) in &docs {
            clear_document_data(&conn, doc_id)?;
            conn.execute(
                "UPDATE documents SET status = 'processing', error_message = NULL, word_count = 0, chunk_count = 0, updated_at = ?1 WHERE id = ?2",
                rusqlite::params![now, doc_id],
            )?;
        }
    }

    let app = app_handle.clone();
    let cid = collection_id.clone();
    tauri::async_runtime::spawn(async move {
        let app_state: tauri::State<'_, AppState> = app.state();
        for (i, (doc_id, file_path, filename, file_type)) in docs.iter().enumerate() {
            let _ = app.emit(
                "reingest-collection-progress",
                serde_json::json!({
                    "collection_id": cid,
                    "documents_done": i,
                    "documents_total": total_docs,
                }),
            );
            if let Err(e) = ingest_single_file(
                &app,
                app_state.inner(),
                &cid,
                doc_id,
                file_path,
                filename,
                file_type,
            )
            .await
            {
                tracing::error!("Re-ingestion failed for {}: {}", filename, e);
            }
        }
        let _ = app.emit(
            "reingest-collection-progress",
            serde_json::json!({
                "collection_id": cid,
                "documents_done": total_docs,
                "documents_total": total_docs,
            }),
        );
        // Rebuild HNSW index after collection re-ingestion
        if let Ok(conn) = crate::state::get_conn(app_state.inner()) {
            if let Ok(mut index) = app_state.inner().vector_index.write() {
                let _ = index.rebuild_collection_index(&conn, &cid);
            }
        }
        let _ = app.emit("ingestion-all-complete", serde_json::json!({ "collection_id": cid }));
    });

    Ok(())
}

#[tauri::command]
pub fn list_documents(
    state: tauri::State<'_, AppState>,
    collection_id: String,
) -> Result<Vec<Document>, AppError> {
    let conn = get_conn(state.inner())?;

    let mut stmt = conn.prepare(
        "SELECT id, collection_id, filename, file_path, file_type, file_size, file_hash, title, author, page_count, word_count, chunk_count, status, error_message, created_at, updated_at
         FROM documents WHERE collection_id = ?1 ORDER BY created_at DESC",
    )?;

    let documents = stmt
        .query_map(rusqlite::params![collection_id], |row: &rusqlite::Row| {
            Ok(Document {
                id: row.get(0)?,
                collection_id: row.get(1)?,
                filename: row.get(2)?,
                file_path: row.get(3)?,
                file_type: row.get(4)?,
                file_size: row.get(5)?,
                file_hash: row.get(6)?,
                title: row.get(7)?,
                author: row.get(8)?,
                page_count: row.get(9)?,
                word_count: row.get(10)?,
                chunk_count: row.get(11)?,
                status: row.get(12)?,
                error_message: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(documents)
}

#[tauri::command]
pub fn get_document(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Document, AppError> {
    let conn = get_conn(state.inner())?;

    let document = conn
        .query_row(
            "SELECT id, collection_id, filename, file_path, file_type, file_size, file_hash, title, author, page_count, word_count, chunk_count, status, error_message, created_at, updated_at
             FROM documents WHERE id = ?1",
            rusqlite::params![id],
            |row: &rusqlite::Row| {
                Ok(Document {
                    id: row.get(0)?,
                    collection_id: row.get(1)?,
                    filename: row.get(2)?,
                    file_path: row.get(3)?,
                    file_type: row.get(4)?,
                    file_size: row.get(5)?,
                    file_hash: row.get(6)?,
                    title: row.get(7)?,
                    author: row.get(8)?,
                    page_count: row.get(9)?,
                    word_count: row.get(10)?,
                    chunk_count: row.get(11)?,
                    status: row.get(12)?,
                    error_message: row.get(13)?,
                    created_at: row.get(14)?,
                    updated_at: row.get(15)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("Document '{}' not found", id))
            }
            other => AppError::Database(other),
        })?;

    Ok(document)
}

#[tauri::command]
pub fn delete_document(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    let conn = get_conn(state.inner())?;

    // Get collection_id before deleting (needed for HNSW rebuild)
    let collection_id: String = conn
        .query_row(
            "SELECT collection_id FROM documents WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("Document '{}' not found", id))
            }
            other => AppError::Database(other),
        })?;

    let _ = audit::log_audit(&conn, AuditAction::DocumentDelete, Some("document"), Some(&id), &serde_json::json!({}));

    clear_document_data(&conn, &id)?;

    let rows = conn.execute(
        "DELETE FROM documents WHERE id = ?1",
        rusqlite::params![id],
    )?;

    if rows == 0 {
        return Err(AppError::NotFound(format!("Document '{}' not found", id)));
    }

    // Rebuild HNSW index after deletion
    if let Ok(mut index) = state.inner().vector_index.write() {
        let _ = index.rebuild_collection_index(&conn, &collection_id);
    }

    Ok(())
}

#[tauri::command]
pub fn get_document_chunks(
    state: tauri::State<'_, AppState>,
    document_id: String,
) -> Result<Vec<Chunk>, AppError> {
    let conn = get_conn(state.inner())?;

    let mut stmt = conn.prepare(
        "SELECT id, document_id, collection_id, content, chunk_index, start_offset, end_offset, page_number, section_title, token_count, created_at
         FROM chunks WHERE document_id = ?1 ORDER BY chunk_index ASC",
    )?;

    let chunks = stmt
        .query_map(rusqlite::params![document_id], |row: &rusqlite::Row| {
            Ok(Chunk {
                id: row.get(0)?,
                document_id: row.get(1)?,
                collection_id: row.get(2)?,
                content: row.get(3)?,
                chunk_index: row.get(4)?,
                start_offset: row.get(5)?,
                end_offset: row.get(6)?,
                page_number: row.get(7)?,
                section_title: row.get(8)?,
                token_count: row.get(9)?,
                created_at: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(chunks)
}

#[tauri::command]
pub fn get_stats(
    state: tauri::State<'_, AppState>,
    collection_id: String,
) -> Result<(i64, i64), AppError> {
    let conn = get_conn(state.inner())?;

    let doc_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM documents WHERE collection_id = ?1 AND status = 'completed'",
        rusqlite::params![collection_id],
        |row: &rusqlite::Row| row.get(0),
    )?;

    let chunk_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM chunks WHERE collection_id = ?1",
        rusqlite::params![collection_id],
        |row: &rusqlite::Row| row.get(0),
    )?;

    Ok((doc_count, chunk_count))
}

#[tauri::command]
pub fn add_document_tag(
    state: tauri::State<'_, AppState>,
    document_id: String,
    tag: String,
) -> Result<Vec<String>, AppError> {
    let tag = tag.trim().to_string();
    if tag.is_empty() {
        return Err(AppError::Validation("Tag cannot be empty".into()));
    }

    let conn = get_conn(state.inner())?;

    let tags_json: String = conn
        .query_row(
            "SELECT COALESCE(tags, '[]') FROM documents WHERE id = ?1",
            rusqlite::params![document_id],
            |row: &rusqlite::Row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("Document '{}' not found", document_id))
            }
            other => AppError::Database(other),
        })?;

    let mut tags: Vec<String> = serde_json::from_str(&tags_json)
        .unwrap_or_default();

    if !tags.contains(&tag) {
        tags.push(tag);
    }

    let updated_json = serde_json::to_string(&tags)
        .map_err(|e| AppError::Validation(format!("Failed to serialize tags: {}", e)))?;

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE documents SET tags = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![updated_json, now, document_id],
    )?;

    Ok(tags)
}

#[tauri::command]
pub fn remove_document_tag(
    state: tauri::State<'_, AppState>,
    document_id: String,
    tag: String,
) -> Result<Vec<String>, AppError> {
    let conn = get_conn(state.inner())?;

    let tags_json: String = conn
        .query_row(
            "SELECT COALESCE(tags, '[]') FROM documents WHERE id = ?1",
            rusqlite::params![document_id],
            |row: &rusqlite::Row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("Document '{}' not found", document_id))
            }
            other => AppError::Database(other),
        })?;

    let mut tags: Vec<String> = serde_json::from_str(&tags_json)
        .unwrap_or_default();

    tags.retain(|t| t != &tag);

    let updated_json = serde_json::to_string(&tags)
        .map_err(|e| AppError::Validation(format!("Failed to serialize tags: {}", e)))?;

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE documents SET tags = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![updated_json, now, document_id],
    )?;

    Ok(tags)
}

#[tauri::command]
pub fn list_all_tags(
    state: tauri::State<'_, AppState>,
    collection_id: String,
) -> Result<Vec<String>, AppError> {
    let conn = get_conn(state.inner())?;

    let mut stmt = conn.prepare(
        "SELECT COALESCE(tags, '[]') FROM documents WHERE collection_id = ?1",
    )?;

    let rows = stmt
        .query_map(rusqlite::params![collection_id], |row: &rusqlite::Row| {
            row.get::<_, String>(0)
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut all_tags = std::collections::HashSet::new();
    for tags_json in &rows {
        if let Ok(tags) = serde_json::from_str::<Vec<String>>(tags_json) {
            for tag in tags {
                all_tags.insert(tag);
            }
        }
    }

    let mut result: Vec<String> = all_tags.into_iter().collect();
    result.sort();

    Ok(result)
}
