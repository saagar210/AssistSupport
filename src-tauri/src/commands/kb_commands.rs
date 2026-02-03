use super::*;
use std::sync::Mutex as StdMutex;

use crate::kb::watcher::KbWatcher;

/// Global watcher instance
static KB_WATCHER: Lazy<StdMutex<Option<KbWatcher>>> = Lazy::new(|| StdMutex::new(None));

pub(crate) fn set_kb_folder_impl(state: State<'_, AppState>, folder_path: String) -> Result<(), String> {
    let path = std::path::Path::new(&folder_path);

    // Validate path is within home directory (auto-creates if needed)
    let validated_path = validate_within_home(path).map_err(|e| match e {
        ValidationError::PathTraversal => {
            "KB folder must be within your home directory".to_string()
        }
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This directory cannot be used as it contains sensitive data".to_string()
        }
        _ => format!("Invalid KB folder: {}", e),
    })?;

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

    let path = std::path::Path::new(&folder_path);
    if !path.exists() {
        return Err("KB folder does not exist".into());
    }

    // Run indexing with progress events
    let indexer = KbIndexer::new();
    let result = indexer
        .index_folder(db, path, |progress| {
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

    let path = std::path::Path::new(&folder_path);

    // Create and start watcher
    let mut watcher = KbWatcher::new(path).map_err(|e| e.to_string())?;
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
