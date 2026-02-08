use tauri::State;

use crate::audit::{self, AuditAction};
use crate::error::AppError;
use crate::graph::{self, GraphData};
use crate::state::{get_conn, AppState};

/// Build knowledge graph edges for a collection using the similarity_threshold from settings.
#[tauri::command]
pub fn build_graph(
    state: State<'_, AppState>,
    collection_id: String,
) -> Result<(), AppError> {
    let conn = get_conn(state.inner())?;

    let threshold: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'similarity_threshold'",
            [],
            |row: &rusqlite::Row| row.get(0),
        )
        .unwrap_or_else(|_| "0.75".to_string());

    let threshold_val: f64 = threshold.parse().unwrap_or(0.75);

    // Use HNSW-based rebuild if index is available (O(n * k log n)),
    // otherwise fall back to the original O(n^2) approach
    let used_hnsw = if let Ok(vi) = state.inner().vector_index.read() {
        if vi.has_index(&collection_id) {
            graph::rebuild_graph_edges(&conn, &vi, &collection_id, threshold_val)?;
            true
        } else {
            false
        }
    } else {
        false
    };
    if !used_hnsw {
        graph::build_graph_edges(&conn, &collection_id, threshold_val)?;
    }

    let _ = audit::log_audit(
        &conn,
        AuditAction::GraphBuild,
        Some("collection"),
        Some(&collection_id),
        &serde_json::json!({}),
    );

    Ok(())
}

/// Get graph visualization data (nodes = documents, links = semantic edges).
#[tauri::command]
pub fn get_graph(
    state: State<'_, AppState>,
    collection_id: String,
) -> Result<GraphData, AppError> {
    let conn = get_conn(state.inner())?;
    graph::get_graph_data(&conn, &collection_id)
}
