use super::*;

pub(crate) fn list_drafts_impl(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<SavedDraft>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_drafts(limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

pub(crate) fn search_drafts_impl(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<SavedDraft>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.search_drafts(&query, limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

pub(crate) fn get_draft_impl(
    state: State<'_, AppState>,
    draft_id: String,
) -> Result<SavedDraft, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.get_draft(&draft_id).map_err(|e| e.to_string())
}

pub(crate) fn save_draft_impl(
    state: State<'_, AppState>,
    draft: SavedDraft,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.save_draft(&draft).map_err(|e| e.to_string())
}

pub(crate) fn delete_draft_impl(
    state: State<'_, AppState>,
    draft_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.delete_draft(&draft_id).map_err(|e| e.to_string())
}

pub(crate) fn list_autosaves_impl(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<SavedDraft>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_autosaves(limit.unwrap_or(10))
        .map_err(|e| e.to_string())
}

pub(crate) fn cleanup_autosaves_impl(
    state: State<'_, AppState>,
    keep_count: Option<usize>,
) -> Result<usize, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.cleanup_autosaves(keep_count.unwrap_or(10))
        .map_err(|e| e.to_string())
}
