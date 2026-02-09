use std::collections::HashMap;

use crate::audit::{log_audit, AuditAction};
use crate::error::AppError;
use crate::gdpr::{self, ConsentRecord, RetentionPolicy};
use crate::state::{get_conn, AppState};

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

    let _ = log_audit(
        &conn,
        AuditAction::DataErase,
        Some("document"),
        Some(&document_id),
        &serde_json::json!({"scope": "document", "document_id": document_id}),
    );

    gdpr::erase_document(&conn, &document_id)
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

    gdpr::erase_collection(&conn, &collection_id)
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
