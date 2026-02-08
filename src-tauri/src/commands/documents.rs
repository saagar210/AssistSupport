use std::path::Path;

use sha2::{Digest, Sha256};
use tauri::Emitter;

use crate::chunker;
use crate::embedder;
use crate::error::AppError;
use crate::models::{Chunk, Document};
use crate::parsers;
use crate::state::AppState;
use crate::vector_store;

fn lock_db<'a>(
    state: &'a tauri::State<'a, AppState>,
) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, AppError> {
    state.db.lock().map_err(|e| {
        AppError::Database(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
            Some(format!("Mutex lock failed: {}", e)),
        ))
    })
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

#[tauri::command]
pub async fn ingest_files(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    collection_id: String,
    file_paths: Vec<String>,
) -> Result<Vec<String>, AppError> {
    let mut created_ids: Vec<String> = Vec::new();

    for file_path_str in &file_paths {
        let path = Path::new(file_path_str);
        let doc_id = uuid::Uuid::new_v4().to_string();

        // Detect file type
        let file_type = match detect_file_type(path) {
            Some(ft) => ft,
            None => {
                tracing::warn!("Skipping file with unknown extension: {}", file_path_str);
                continue;
            }
        };

        // Compute hash
        let file_hash = match compute_sha256(path) {
            Ok(h) => h,
            Err(e) => {
                tracing::error!("Failed to hash file {}: {}", file_path_str, e);
                continue;
            }
        };

        // Get file metadata
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

        let now = chrono::Utc::now().to_rfc3339();

        // Check for duplicates and insert document record inside lock scope
        {
            let db = lock_db(&state)?;

            // Check duplicate hash
            let existing: Option<String> = db
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

            // Insert document with status "processing"
            db.execute(
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
        } // db lock dropped here

        // Parse document (sync, no lock needed)
        let parsed = match parsers::parse_document(path, &file_type) {
            Ok(p) => p,
            Err(e) => {
                let db = lock_db(&state)?;
                let now = chrono::Utc::now().to_rfc3339();
                let _ = db.execute(
                    "UPDATE documents SET status = 'failed', error_message = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![e.to_string(), now, doc_id],
                );
                let _ = app_handle.emit(
                    "ingestion-progress",
                    serde_json::json!({"document_id": doc_id, "status": "failed", "error": e.to_string()}),
                );
                continue;
            }
        };

        // Read settings inside lock, then drop
        let (chunk_size, chunk_overlap, ollama_host, ollama_port, embedding_model) = {
            let db = lock_db(&state)?;

            let chunk_size = get_setting(&db, "chunk_size", "512");
            let chunk_overlap = get_setting(&db, "chunk_overlap", "64");
            let host = get_setting(&db, "ollama_host", "localhost");
            let port = get_setting(&db, "ollama_port", "11434");
            let model = get_setting(&db, "embedding_model", "nomic-embed-text");

            (
                chunk_size.parse::<usize>().unwrap_or(512),
                chunk_overlap.parse::<usize>().unwrap_or(64),
                host,
                port,
                model,
            )
        }; // db lock dropped

        // Chunk text (sync)
        let chunks = chunker::chunk_text(
            &parsed.text,
            &parsed.sections,
            chunk_size,
            chunk_overlap,
        );

        // Insert chunks into DB
        {
            let db = lock_db(&state)?;
            let now = chrono::Utc::now().to_rfc3339();

            for chunk in &chunks {
                let chunk_id = uuid::Uuid::new_v4().to_string();

                db.execute(
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

                // Insert into FTS table
                db.execute(
                    "INSERT INTO chunks_fts (content, chunk_id, document_id, collection_id) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![chunk.content, chunk_id, doc_id, collection_id],
                )?;
            }
        } // db lock dropped

        // Generate embeddings (async, no lock held)
        let embeddings = match embedder::embed_chunks(
            &ollama_host,
            &ollama_port,
            &embedding_model,
            &chunks,
        )
        .await
        {
            Ok(e) => e,
            Err(e) => {
                let db = lock_db(&state)?;
                let now = chrono::Utc::now().to_rfc3339();
                let _ = db.execute(
                    "UPDATE documents SET status = 'failed', error_message = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![e.to_string(), now, doc_id],
                );
                let _ = app_handle.emit(
                    "ingestion-progress",
                    serde_json::json!({"document_id": doc_id, "status": "failed", "error": e.to_string()}),
                );
                continue;
            }
        };

        // Store embeddings and update document status
        {
            let db = lock_db(&state)?;

            // Re-query chunk IDs in order to match with embeddings
            let mut stmt = db.prepare(
                "SELECT id, content FROM chunks WHERE document_id = ?1 ORDER BY chunk_index ASC",
            )?;
            let chunk_rows: Vec<(String, String)> = stmt
                .query_map(rusqlite::params![doc_id], |row: &rusqlite::Row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?
                .collect::<Result<Vec<_>, _>>()?;

            // Build embedding tuples
            let mut embedding_data: Vec<(String, String, String, Vec<f64>, String)> = Vec::new();
            for (i, (chunk_id, content)) in chunk_rows.iter().enumerate() {
                if let Some(embedding) = embeddings.get(i) {
                    let preview = if content.len() > 200 {
                        format!("{}...", &content[..200])
                    } else {
                        content.clone()
                    };
                    embedding_data.push((
                        chunk_id.clone(),
                        collection_id.clone(),
                        doc_id.clone(),
                        embedding.clone(),
                        preview,
                    ));
                }
            }

            vector_store::store_embeddings(&db, &embedding_data)?;

            // Update document status to completed
            let now = chrono::Utc::now().to_rfc3339();
            db.execute(
                "UPDATE documents SET status = 'completed', word_count = ?1, chunk_count = ?2, title = ?3, author = ?4, page_count = ?5, updated_at = ?6 WHERE id = ?7",
                rusqlite::params![
                    parsed.metadata.word_count,
                    chunks.len() as i32,
                    parsed.metadata.title.as_deref().unwrap_or(&filename),
                    parsed.metadata.author,
                    parsed.metadata.page_count,
                    now,
                    doc_id,
                ],
            )?;
        } // db lock dropped

        let _ = app_handle.emit(
            "ingestion-progress",
            serde_json::json!({"document_id": doc_id, "status": "completed"}),
        );

        created_ids.push(doc_id);
    }

    Ok(created_ids)
}

#[tauri::command]
pub fn list_documents(
    state: tauri::State<'_, AppState>,
    collection_id: String,
) -> Result<Vec<Document>, AppError> {
    let db = lock_db(&state)?;

    let mut stmt = db.prepare(
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
    let db = lock_db(&state)?;

    let document = db
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
    let db = lock_db(&state)?;

    // Delete embeddings
    vector_store::delete_document_vectors(&db, &id)?;

    // Delete FTS entries for this document's chunks
    db.execute(
        "DELETE FROM chunks_fts WHERE document_id = ?1",
        rusqlite::params![id],
    )?;

    // Delete graph edges referencing this document's chunks
    db.execute(
        "DELETE FROM graph_edges WHERE source_chunk_id IN (SELECT id FROM chunks WHERE document_id = ?1) OR target_chunk_id IN (SELECT id FROM chunks WHERE document_id = ?1)",
        rusqlite::params![id],
    )?;

    // Delete chunks (cascade should handle this, but be explicit)
    db.execute(
        "DELETE FROM chunks WHERE document_id = ?1",
        rusqlite::params![id],
    )?;

    // Delete the document itself
    let rows = db.execute(
        "DELETE FROM documents WHERE id = ?1",
        rusqlite::params![id],
    )?;

    if rows == 0 {
        return Err(AppError::NotFound(format!("Document '{}' not found", id)));
    }

    Ok(())
}

#[tauri::command]
pub fn get_document_chunks(
    state: tauri::State<'_, AppState>,
    document_id: String,
) -> Result<Vec<Chunk>, AppError> {
    let db = lock_db(&state)?;

    let mut stmt = db.prepare(
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
    let db = lock_db(&state)?;

    let doc_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM documents WHERE collection_id = ?1 AND status = 'completed'",
        rusqlite::params![collection_id],
        |row: &rusqlite::Row| row.get(0),
    )?;

    let chunk_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM chunks WHERE collection_id = ?1",
        rusqlite::params![collection_id],
        |row: &rusqlite::Row| row.get(0),
    )?;

    Ok((doc_count, chunk_count))
}
