use super::*;
use std::path::PathBuf;
use std::sync::Mutex as StdMutex;
use tauri::Emitter;

use crate::db::get_app_data_dir;
use crate::kb::indexer::{IndexResult, IndexStats, KbIndexer};
use crate::kb::ocr::OcrManager;
use crate::kb::vectors::{VectorStore, VectorStoreConfig};
use crate::kb::watcher::KbWatcher;

/// Global watcher instance
static KB_WATCHER: Lazy<StdMutex<Option<KbWatcher>>> = Lazy::new(|| StdMutex::new(None));
const MAX_OCR_BASE64_BYTES: usize = 10 * 1024 * 1024;
const KB_FOLDER_SETTING: &str = "kb_folder";

fn validate_stored_kb_path(folder_path: &str) -> Result<std::path::PathBuf, String> {
    let path = std::path::Path::new(folder_path);
    validate_within_home(path).map_err(|e| match e {
        ValidationError::PathTraversal => {
            "KB folder must be within your home directory".to_string()
        }
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This directory cannot be used as it contains sensitive data".to_string()
        }
        _ => format!("Invalid KB folder: {}", e),
    })
}

pub(crate) fn set_kb_folder_impl(
    state: State<'_, AppState>,
    folder_path: String,
) -> Result<(), String> {
    // Validate path is within home directory (auto-creates if needed)
    let validated_path = validate_stored_kb_path(&folder_path)?;

    // Verify it's a directory
    if !validated_path.is_dir() {
        return Err("Path is not a directory".into());
    }

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    // Store in settings
    db.conn()
        .execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            rusqlite::params![KB_FOLDER_SETTING, validated_path.to_string_lossy().as_ref()],
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub(crate) fn get_kb_folder_impl(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let result: Result<String, _> = db.conn().query_row(
        "SELECT value FROM settings WHERE key = ?",
        rusqlite::params![KB_FOLDER_SETTING],
        |row| row.get(0),
    );

    match result {
        Ok(path) => Ok(Some(path)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub(crate) async fn index_kb_impl(
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<IndexResult, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    // Get KB folder
    let folder_path: String = db
        .conn()
        .query_row(
            "SELECT value FROM settings WHERE key = ?",
            rusqlite::params![KB_FOLDER_SETTING],
            |row| row.get(0),
        )
        .map_err(|_| "KB folder not configured")?;

    let validated_path = validate_stored_kb_path(&folder_path)?;
    if !validated_path.exists() {
        return Err("KB folder does not exist".into());
    }

    // Run indexing with progress events
    let indexer = KbIndexer::new();
    let result = indexer
        .index_folder(db, &validated_path, |progress| {
            // Emit progress event to frontend
            let _ = window.emit("kb:indexing:progress", &progress);
        })
        .map_err(|e| e.to_string())?;

    Ok(result)
}

pub(crate) fn get_kb_stats_impl(state: State<'_, AppState>) -> Result<IndexStats, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let indexer = KbIndexer::new();
    indexer.get_stats(db).map_err(|e| e.to_string())
}

pub(crate) fn list_kb_documents_impl(
    state: State<'_, AppState>,
    namespace_id: Option<String>,
    source_id: Option<String>,
) -> Result<Vec<KbDocumentInfo>, String> {
    // Validate and normalize namespace_id if provided
    let namespace_id = namespace_id
        .map(|ns| normalize_and_validate_namespace_id(&ns))
        .transpose()
        .map_err(|e| e.to_string())?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let docs = db
        .list_kb_documents(namespace_id.as_deref(), source_id.as_deref())
        .map_err(|e| e.to_string())?;

    Ok(docs
        .into_iter()
        .map(|d| KbDocumentInfo {
            id: d.id,
            file_path: d.file_path,
            title: d.title,
            indexed_at: d.indexed_at,
            chunk_count: d.chunk_count.map(|c| c as i64),
            namespace_id: d.namespace_id,
            source_type: d.source_type,
            source_id: d.source_id,
        })
        .collect())
}

pub(crate) fn remove_kb_document_impl(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let indexer = KbIndexer::new();
    indexer
        .remove_document(db, &file_path)
        .map_err(|e| e.to_string())
}

pub(crate) async fn start_kb_watcher_impl(
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    // Get KB folder path
    let folder_path = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;
        db.conn()
            .query_row(
                "SELECT value FROM settings WHERE key = ?",
                rusqlite::params![KB_FOLDER_SETTING],
                |row| row.get::<_, String>(0),
            )
            .map_err(|_| "KB folder not configured")?
    };

    let validated_path = validate_stored_kb_path(&folder_path)?;

    // Create and start watcher
    let mut watcher = KbWatcher::new(&validated_path).map_err(|e| e.to_string())?;
    let mut rx = watcher.start().map_err(|e| e.to_string())?;

    // Store watcher instance
    {
        let mut guard = KB_WATCHER.lock().map_err(|e| e.to_string())?;
        *guard = Some(watcher);
    }

    // Spawn event handler
    let window_clone = window.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            // Emit event to frontend
            let _ = window_clone.emit("kb:file:changed", &event);
        }
    });

    Ok(true)
}

pub(crate) fn stop_kb_watcher_impl() -> Result<bool, String> {
    let mut guard = KB_WATCHER.lock().map_err(|e| e.to_string())?;
    if let Some(mut watcher) = guard.take() {
        watcher.stop();
        Ok(true)
    } else {
        Ok(false)
    }
}

pub(crate) fn is_kb_watcher_running_impl() -> Result<bool, String> {
    let guard = KB_WATCHER.lock().map_err(|e| e.to_string())?;
    Ok(guard.as_ref().map(|w| w.is_running()).unwrap_or(false))
}

pub(crate) async fn generate_kb_embeddings_impl(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<EmbeddingGenerationResult, String> {
    {
        let vectors_lock = state.vectors.read().await;
        let embeddings_lock = state.embeddings.read();

        let vectors = vectors_lock
            .as_ref()
            .ok_or("Vector store not initialized")?;
        if !vectors.is_enabled() {
            return Err("Vector search is disabled".into());
        }

        let embeddings = embeddings_lock
            .as_ref()
            .ok_or("Embedding engine not initialized")?;
        if !embeddings.is_model_loaded() {
            return Err("Embedding model not loaded".into());
        }
    }

    let chunks: Vec<(String, String)> = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;
        db.get_all_chunks_for_embedding()
            .map_err(|e| e.to_string())?
    };

    if chunks.is_empty() {
        return Ok(EmbeddingGenerationResult {
            chunks_processed: 0,
            vectors_created: 0,
        });
    }

    let total_chunks = chunks.len();
    let batch_size = 32usize;
    let mut vectors_created = 0usize;

    let _ = app_handle.emit(
        "kb:embeddings:start",
        serde_json::json!({
            "total_chunks": total_chunks
        }),
    );

    for (batch_idx, batch) in chunks.chunks(batch_size).enumerate() {
        let chunk_ids: Vec<String> = batch.iter().map(|(id, _)| id.clone()).collect();
        let chunk_texts: Vec<String> = batch.iter().map(|(_, text)| text.clone()).collect();

        let embeddings: Vec<Vec<f32>> = {
            let embeddings_lock = state.embeddings.read();
            let engine = embeddings_lock
                .as_ref()
                .ok_or("Embedding engine not available")?;
            engine
                .embed_batch(&chunk_texts)
                .map_err(|e| e.to_string())?
        };

        {
            let vectors_lock = state.vectors.read().await;
            let vectors = vectors_lock.as_ref().ok_or("Vector store not available")?;
            vectors
                .insert_embeddings(&chunk_ids, &embeddings)
                .await
                .map_err(|e| e.to_string())?;
        }

        vectors_created += embeddings.len();

        let progress = ((batch_idx + 1) * batch_size).min(total_chunks);
        let _ = app_handle.emit(
            "kb:embeddings:progress",
            serde_json::json!({
                "processed": progress,
                "total": total_chunks,
                "percentage": (progress * 100) / total_chunks
            }),
        );
    }

    let _ = app_handle.emit(
        "kb:embeddings:complete",
        serde_json::json!({
            "vectors_created": vectors_created
        }),
    );

    Ok(EmbeddingGenerationResult {
        chunks_processed: total_chunks,
        vectors_created,
    })
}

pub(crate) async fn init_vector_store_impl(state: State<'_, AppState>) -> Result<(), String> {
    let app_dir = get_app_data_dir();
    let vectors_path = app_dir.join("vectors");

    let embedding_dim = {
        let emb_guard = state.embeddings.read();
        emb_guard
            .as_ref()
            .and_then(|e| e.embedding_dim())
            .unwrap_or(768)
    };

    let config = VectorStoreConfig {
        path: vectors_path,
        embedding_dim,
        encryption_enabled: false,
    };

    let mut store = VectorStore::new(config);
    store.init().await.map_err(|e| e.to_string())?;
    store.create_table().await.map_err(|e| e.to_string())?;

    let consented = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        if let Some(db) = db_lock.as_ref() {
            db.get_vector_consent().map(|c| c.enabled).unwrap_or(false)
        } else {
            false
        }
    };

    if consented {
        store.enable(true).map_err(|e| e.to_string())?;
    }

    *state.vectors.write().await = Some(store);
    Ok(())
}

pub(crate) async fn set_vector_enabled_impl(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    let mut vectors_lock = state.vectors.write().await;
    let store = vectors_lock
        .as_mut()
        .ok_or("Vector store not initialized")?;

    if enabled {
        store.enable(true).map_err(|e| e.to_string())?;
    } else {
        store.disable();
    }

    Ok(())
}

pub(crate) async fn is_vector_enabled_impl(state: State<'_, AppState>) -> Result<bool, String> {
    let vectors_lock = state.vectors.read().await;
    Ok(vectors_lock
        .as_ref()
        .map(|s| s.is_enabled())
        .unwrap_or(false))
}

pub(crate) async fn get_vector_stats_impl(
    state: State<'_, AppState>,
) -> Result<VectorStats, String> {
    let vectors_lock = state.vectors.read().await;
    let store = vectors_lock
        .as_ref()
        .ok_or("Vector store not initialized")?;

    let count = store.count().await.map_err(|e| e.to_string())?;

    Ok(VectorStats {
        enabled: store.is_enabled(),
        vector_count: count,
        embedding_dim: store.embedding_dim(),
        encryption_supported: store.encryption_supported(),
    })
}

pub(crate) fn process_ocr_impl(image_path: String) -> Result<OcrResult, String> {
    let ocr = OcrManager::new();
    let path = PathBuf::from(&image_path);

    if !path.exists() {
        return Err(format!("Image file not found: {}", image_path));
    }

    let validated_path = validate_within_home(&path).map_err(|e| match e {
        ValidationError::PathTraversal => {
            "Image file must be within your home directory".to_string()
        }
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This path is blocked because it contains sensitive data".to_string()
        }
        _ => format!("Invalid image path: {}", e),
    })?;

    if !validated_path.is_file() {
        return Err("Image path is not a file".into());
    }

    let result = ocr.recognize(&validated_path).map_err(|e| e.to_string())?;

    Ok(OcrResult {
        text: result.text,
        confidence: result.confidence.unwrap_or(1.0),
    })
}

pub(crate) fn process_ocr_bytes_impl(image_base64: String) -> Result<OcrResult, String> {
    use base64::{engine::general_purpose, Engine as _};

    if image_base64.len() > MAX_OCR_BASE64_BYTES {
        return Err(format!(
            "Image too large: {} bytes exceeds limit of {} bytes. Please use a smaller image.",
            image_base64.len(),
            MAX_OCR_BASE64_BYTES
        ));
    }

    let image_data = general_purpose::STANDARD
        .decode(&image_base64)
        .map_err(|e| format!("Invalid base64 data: {}", e))?;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("assistsupport_ocr_{}.png", uuid::Uuid::new_v4()));

    std::fs::write(&temp_path, &image_data)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    let ocr = OcrManager::new();
    let result = ocr.recognize(&temp_path).map_err(|e| e.to_string());

    let _ = std::fs::remove_file(&temp_path);

    let ocr_result = result?;
    Ok(OcrResult {
        text: ocr_result.text,
        confidence: ocr_result.confidence.unwrap_or(1.0),
    })
}

pub(crate) fn is_ocr_available_impl() -> bool {
    let ocr = OcrManager::new();
    !ocr.available_providers().is_empty()
}
