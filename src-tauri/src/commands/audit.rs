use serde::{Deserialize, Serialize};
use tauri::State;

use crate::audit::{self, AuditEntry};
use crate::error::AppError;
use crate::state::{get_conn, AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogResponse {
    pub entries: Vec<AuditEntry>,
    pub total: i64,
    pub page: usize,
    pub page_size: usize,
    pub has_more: bool,
}

#[tauri::command]
pub fn get_audit_log(
    state: State<'_, AppState>,
    action_filter: Option<String>,
    entity_type_filter: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> Result<AuditLogResponse, AppError> {
    let conn = get_conn(state.inner())?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(50);

    let (entries, total) = audit::query_audit_log(
        &conn,
        action_filter.as_deref(),
        entity_type_filter.as_deref(),
        start_date.as_deref(),
        end_date.as_deref(),
        page,
        page_size,
    )?;

    let has_more = (page * page_size) < total as usize;

    Ok(AuditLogResponse {
        entries,
        total,
        page,
        page_size,
        has_more,
    })
}

#[tauri::command]
pub fn export_audit_log(
    state: State<'_, AppState>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<String, AppError> {
    let conn = get_conn(state.inner())?;

    // Export all matching entries as JSON
    let (entries, _) = audit::query_audit_log(
        &conn,
        None,
        None,
        start_date.as_deref(),
        end_date.as_deref(),
        1,
        100_000, // Large page to get all
    )?;

    serde_json::to_string_pretty(&entries)
        .map_err(|e| AppError::Validation(format!("Failed to serialize audit log: {}", e)))
}
