use super::*;

/// Get the configured context window size
#[tauri::command]
pub fn get_context_window(state: State<'_, AppState>) -> Result<Option<u32>, String> {
    model_commands::get_context_window_impl(state)
}

/// Set the context window size (2048-32768, or None for model default)
#[tauri::command]
pub fn set_context_window(state: State<'_, AppState>, size: Option<u32>) -> Result<(), String> {
    model_commands::set_context_window_impl(state, size)
}

// ============================================================================
// Download Commands
// ============================================================================

use crate::downloads::ModelSource;

/// Get recommended models list
#[tauri::command]
pub fn get_recommended_models() -> Vec<ModelSource> {
    model_commands::get_recommended_models_impl()
}

/// List downloaded models
#[tauri::command]
pub fn list_downloaded_models() -> Result<Vec<String>, String> {
    model_commands::list_downloaded_models_impl()
}

/// Check if embedding model is downloaded and return its path if so
#[tauri::command]
pub fn get_embedding_model_path(model_id: String) -> Result<Option<String>, String> {
    model_commands::get_embedding_model_path_impl(model_id)
}

/// Check if the default embedding model is downloaded
#[tauri::command]
pub fn is_embedding_model_downloaded() -> Result<bool, String> {
    model_commands::is_embedding_model_downloaded_impl()
}

/// Get models directory path
#[tauri::command]
pub fn get_models_dir() -> Result<String, String> {
    model_commands::get_models_dir_impl()
}

/// Delete a downloaded model
#[tauri::command]
pub fn delete_downloaded_model(filename: String) -> Result<(), String> {
    model_commands::delete_downloaded_model_impl(filename)
}

/// Get HuggingFace token status (not the actual token for security)
#[tauri::command]
pub fn has_hf_token() -> Result<bool, String> {
    security_commands::has_hf_token_impl()
}

/// Store HuggingFace token
#[tauri::command]
pub fn set_hf_token(token: String) -> Result<(), String> {
    security_commands::set_hf_token_impl(token)
}

/// Delete HuggingFace token
#[tauri::command]
pub fn clear_hf_token() -> Result<(), String> {
    security_commands::clear_hf_token_impl()
}

/// Get Search API bearer token status (not the actual token for security)
#[tauri::command]
pub fn has_search_api_token() -> Result<bool, String> {
    security_commands::has_search_api_token_impl()
}

/// Store Search API bearer token
#[tauri::command]
pub fn set_search_api_token(token: String) -> Result<(), String> {
    security_commands::set_search_api_token_impl(token)
}

/// Delete Search API bearer token
#[tauri::command]
pub fn clear_search_api_token() -> Result<(), String> {
    security_commands::clear_search_api_token_impl()
}

/// Get MemoryKernel service token status (not the actual token for security)
#[tauri::command]
pub fn has_memorykernel_service_token() -> Result<bool, String> {
    security_commands::has_memorykernel_service_token_impl()
}

/// Store MemoryKernel service bearer token
#[tauri::command]
pub fn set_memorykernel_service_token(token: String) -> Result<(), String> {
    security_commands::set_memorykernel_service_token_impl(token)
}

/// Delete MemoryKernel service bearer token
#[tauri::command]
pub fn clear_memorykernel_service_token() -> Result<(), String> {
    security_commands::clear_memorykernel_service_token_impl()
}

/// Store GitHub token for a specific host (HTTPS only)
#[tauri::command]
pub fn set_github_token(host: String, token: String) -> Result<(), String> {
    security_commands::set_github_token_impl(host, token)
}

/// Delete GitHub token for a specific host
#[tauri::command]
pub fn clear_github_token(host: String) -> Result<(), String> {
    security_commands::clear_github_token_impl(host)
}

/// Check if a GitHub token exists for a host (does not return the token)
#[tauri::command]
pub fn has_github_token(host: String) -> Result<bool, String> {
    security_commands::has_github_token_impl(host)
}

/// Read audit log entries (most recent first if limit is set)
#[tauri::command]
pub fn get_audit_entries(limit: Option<usize>) -> Result<Vec<crate::audit::AuditEntry>, String> {
    security_commands::get_audit_entries_impl(limit)
}

/// Export audit log entries to a JSON file
#[tauri::command]
pub fn export_audit_log(export_path: String) -> Result<String, String> {
    security_commands::export_audit_log_impl(export_path)
}

/// Audit: operator overrode copy gating (e.g., copied without citations).
///
/// This is best-effort and intentionally does not include the copied text.
#[tauri::command]
pub fn audit_response_copy_override(
    reason: String,
    confidence_mode: Option<String>,
    sources_count: usize,
) -> Result<(), String> {
    use crate::audit::{AuditEntry, AuditEventType, AuditSeverity};

    let trimmed = reason.trim();
    if trimmed.is_empty() {
        return Err("Reason is required".into());
    }

    crate::audit::log_audit_best_effort(
        AuditEntry::new(
            AuditEventType::Custom("response_copy_override".to_string()),
            AuditSeverity::Warning,
            "Operator overrode copy gating".to_string(),
        )
        .with_context(serde_json::json!({
            "reason": trimmed,
            "confidence_mode": confidence_mode,
            "sources_count": sources_count,
        })),
    );

    Ok(())
}

/// Download a model from HuggingFace with progress events
#[tauri::command]
pub async fn download_model(window: tauri::Window, model_id: String) -> Result<String, String> {
    model_commands::download_model_impl(window, model_id).await
}

/// Cancel an ongoing download
#[tauri::command]
pub fn cancel_download() -> Result<(), String> {
    model_commands::cancel_download_impl()
}

// ============================================================================
// Startup & Model State Commands (v0.4.1)
// ============================================================================

/// Get the last-used model state (for auto-load on startup)
#[tauri::command]
pub fn get_model_state(state: State<'_, AppState>) -> Result<ModelStateResult, String> {
    model_commands::get_model_state_impl(state)
}

#[derive(serde::Serialize, Clone)]
pub struct ModelStateResult {
    pub llm_model_id: Option<String>,
    pub llm_model_path: Option<String>,
    pub llm_loaded: bool,
    pub embeddings_model_path: Option<String>,
    pub embeddings_loaded: bool,
}

/// Get the last startup metrics
#[tauri::command]
pub fn get_startup_metrics(state: State<'_, AppState>) -> Result<StartupMetricsResult, String> {
    model_commands::get_startup_metrics_impl(state)
}

#[derive(serde::Serialize, Clone)]
pub struct StartupMetricsResult {
    pub total_ms: i64,
    pub init_app_ms: i64,
    pub models_cached: bool,
}
