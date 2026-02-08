use tauri::State;

use crate::error::AppError;
use crate::graph::{self, GraphData};
use crate::state::AppState;

/// Helper to lock the DB mutex.
fn lock_db<'a>(
    state: &'a State<'a, AppState>,
) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, AppError> {
    crate::state::lock_db(state.inner())
}

/// Build knowledge graph edges for a collection using the similarity_threshold from settings.
#[tauri::command]
pub fn build_graph(
    state: State<'_, AppState>,
    collection_id: String,
) -> Result<(), AppError> {
    let db = lock_db(&state)?;

    let threshold: String = db
        .query_row(
            "SELECT value FROM settings WHERE key = 'similarity_threshold'",
            [],
            |row: &rusqlite::Row| row.get(0),
        )
        .unwrap_or_else(|_| "0.75".to_string());

    let threshold_val: f64 = threshold.parse().unwrap_or(0.75);

    graph::build_graph_edges(&db, &collection_id, threshold_val)?;
    Ok(())
}

/// Get graph visualization data (nodes = documents, links = semantic edges).
#[tauri::command]
pub fn get_graph(
    state: State<'_, AppState>,
    collection_id: String,
) -> Result<GraphData, AppError> {
    let db = lock_db(&state)?;
    graph::get_graph_data(&db, &collection_id)
}
