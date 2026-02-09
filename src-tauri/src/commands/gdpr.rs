use std::collections::HashMap;

use crate::audit::{log_audit, AuditAction};
use crate::error::AppError;
use crate::gdpr::{self, ConsentRecord, RetentionPolicy};
use crate::state::{get_conn, AppState};

/// Rebuild the HNSW index for a specific collection after data erasure.
fn rebuild_index_for_collection(state: &AppState, collection_id: &str) {
    let conn = match get_conn(state) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to get connection for index rebuild: {}", e);
            return;
        }
    };
    let mut index = match state.vector_index.write() {
        Ok(i) => i,
        Err(e) => {
            tracing::warn!("Failed to acquire write lock for index rebuild: {}", e);
            return;
        }
    };
    if let Err(e) = index.rebuild_collection_index(&conn, collection_id) {
        tracing::warn!("Failed to rebuild HNSW index for collection {}: {}", collection_id, e);
    }
}

/// Drop the HNSW index for a specific collection after full erasure.
fn drop_index_for_collection(state: &AppState, collection_id: &str) {
    if let Ok(mut index) = state.vector_index.write() {
        index.drop_collection(collection_id);
    }
}

/// Drop all HNSW indices after full data erasure.
fn drop_all_indices(state: &AppState) {
    if let Ok(mut index) = state.vector_index.write() {
        index.clear();
    }
}

/// Export all user data as JSON (GDPR data portability).
#[tauri::command]
pub fn export_user_data(
    state: tauri::State<'_, AppState>,
) -> Result<HashMap<String, String>, AppError> {
    let conn = get_conn(state.inner())?;

    let _ = log_audit(
        &conn,
        AuditAction::DataExport,
        None,
        None,
        &serde_json::json!({"action": "full_export"}),
    );

    gdpr::export_all_data(&conn)
}

/// Erase a specific document and all related data (GDPR right to erasure).
#[tauri::command]
pub fn erase_document_data(
    state: tauri::State<'_, AppState>,
    document_id: String,
) -> Result<(), AppError> {
    let conn = get_conn(state.inner())?;

    // Get the collection_id before erasing so we can rebuild the index
    let collection_id: Option<String> = conn
        .query_row(
            "SELECT collection_id FROM documents WHERE id = ?1",
            rusqlite::params![document_id],
            |row| row.get(0),
        )
        .ok();

    let _ = log_audit(
        &conn,
        AuditAction::DataErase,
        Some("document"),
        Some(&document_id),
        &serde_json::json!({"scope": "document", "document_id": document_id}),
    );

    gdpr::erase_document(&conn, &document_id)?;

    // Rebuild HNSW index for the affected collection
    if let Some(cid) = collection_id {
        rebuild_index_for_collection(state.inner(), &cid);
    }

    Ok(())
}

/// Erase an entire collection and all related data (GDPR right to erasure).
#[tauri::command]
pub fn erase_collection_data(
    state: tauri::State<'_, AppState>,
    collection_id: String,
) -> Result<(), AppError> {
    let conn = get_conn(state.inner())?;

    let _ = log_audit(
        &conn,
        AuditAction::DataErase,
        Some("collection"),
        Some(&collection_id),
        &serde_json::json!({"scope": "collection", "collection_id": collection_id}),
    );

    gdpr::erase_collection(&conn, &collection_id)?;

    // Drop HNSW index for the erased collection
    drop_index_for_collection(state.inner(), &collection_id);

    Ok(())
}

/// Erase ALL user data. Nuclear option (GDPR right to erasure).
#[tauri::command]
pub fn erase_all_user_data(
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    let conn = get_conn(state.inner())?;

    let _ = log_audit(
        &conn,
        AuditAction::DataErase,
        None,
        None,
        &serde_json::json!({"scope": "all_data"}),
    );

    gdpr::erase_all_data(&conn)?;

    // Drop all HNSW indices
    drop_all_indices(state.inner());

    // Also delete the encryption key from the OS keychain
    crate::crypto::delete_db_key()?;

    Ok(())
}

/// Get all data retention policies.
#[tauri::command]
pub fn get_retention_policies(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<RetentionPolicy>, AppError> {
    let conn = get_conn(state.inner())?;
    gdpr::get_retention_policies(&conn)
}

/// Update a data retention policy.
#[tauri::command]
pub fn update_retention_policy(
    state: tauri::State<'_, AppState>,
    id: String,
    retention_days: i64,
    auto_delete: bool,
) -> Result<(), AppError> {
    let conn = get_conn(state.inner())?;
    gdpr::update_retention_policy(&conn, &id, retention_days, auto_delete)
}

/// Run retention cleanup - delete data older than retention policies.
#[tauri::command]
pub fn run_retention_cleanup(
    state: tauri::State<'_, AppState>,
) -> Result<usize, AppError> {
    let conn = get_conn(state.inner())?;

    let deleted = gdpr::enforce_retention_policies(&conn)?;

    let _ = log_audit(
        &conn,
        AuditAction::RetentionEnforce,
        None,
        None,
        &serde_json::json!({"rows_deleted": deleted}),
    );

    Ok(deleted)
}

/// Record user consent (GDPR consent management).
#[tauri::command]
pub fn record_consent(
    state: tauri::State<'_, AppState>,
    consent_type: String,
    granted: bool,
) -> Result<(), AppError> {
    let conn = get_conn(state.inner())?;
    gdpr::record_consent(&conn, &consent_type, granted)
}

/// Get all consent records.
#[tauri::command]
pub fn get_consent_records(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ConsentRecord>, AppError> {
    let conn = get_conn(state.inner())?;
    gdpr::get_consent_records(&conn)
}
