use tauri::State;

use crate::audit::{self, AuditAction};
use crate::error::AppError;
use crate::graph::{self, Community, GraphData, GraphTraversalNode};
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

/// BFS traversal from a start node, up to max_depth, filtering edges by min_weight.
#[tauri::command]
pub fn traverse_graph_cmd(
    state: State<'_, AppState>,
    collection_id: String,
    start_chunk_id: String,
    max_depth: Option<usize>,
    min_weight: Option<f64>,
) -> Result<Vec<GraphTraversalNode>, AppError> {
    let conn = get_conn(state.inner())?;
    let depth = max_depth.unwrap_or(3);
    let weight = min_weight.unwrap_or(0.3);
    graph::traverse_graph(&conn, &collection_id, &start_chunk_id, depth, weight)
}

/// Find shortest path between two chunks using BFS.
#[tauri::command]
pub fn find_graph_path(
    state: State<'_, AppState>,
    collection_id: String,
    from_chunk_id: String,
    to_chunk_id: String,
) -> Result<Vec<String>, AppError> {
    let conn = get_conn(state.inner())?;
    graph::find_path(&conn, &collection_id, &from_chunk_id, &to_chunk_id)
}

/// Detect communities using label propagation algorithm.
#[tauri::command]
pub fn detect_graph_communities(
    state: State<'_, AppState>,
    collection_id: String,
    min_weight: Option<f64>,
) -> Result<Vec<Community>, AppError> {
    let conn = get_conn(state.inner())?;
    let weight = min_weight.unwrap_or(0.3);
    graph::detect_communities(&conn, &collection_id, weight)
}
