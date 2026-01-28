//! AssistSupport - Self-contained local KB + LLM app for IT support

pub mod audit;
pub mod backup;
pub mod commands;
pub mod db;
pub mod diagnostics;
pub mod downloads;
pub mod error;
pub mod exports;
pub mod jira;
pub mod jobs;
pub mod kb;
pub mod llm;
pub mod migration;
pub mod model_integrity;
pub mod prompts;
pub mod security;
pub mod sources;
pub mod validation;

use crate::db::Database;
use crate::jobs::JobManager;
use crate::kb::embeddings::EmbeddingEngine;
use crate::kb::vectors::VectorStore;
use crate::llm::LlmEngine;
use llama_cpp_2::llama_backend::LlamaBackend;
use parking_lot::RwLock;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock as TokioRwLock;

/// Application state
pub struct AppState {
    /// Shared llama.cpp backend — initialized once, shared by LLM and embedding engines
    pub backend: Arc<LlamaBackend>,
    pub db: Mutex<Option<Database>>,
    pub llm: Arc<RwLock<Option<LlmEngine>>>,
    pub embeddings: Arc<RwLock<Option<EmbeddingEngine>>>,
    pub vectors: Arc<TokioRwLock<Option<VectorStore>>>,
    pub jobs: Arc<JobManager>,
}

impl Default for AppState {
    fn default() -> Self {
        let backend = Arc::new(LlamaBackend::init().expect("Failed to initialize llama backend"));
        Self {
            backend,
            db: Mutex::new(None),
            llm: Arc::new(RwLock::new(None)),
            embeddings: Arc::new(RwLock::new(None)),
            vectors: Arc::new(TokioRwLock::new(None)),
            jobs: Arc::new(JobManager::new()),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::initialize_app,
            commands::check_fts5_enabled,
            commands::check_db_integrity,
            commands::get_vector_consent,
            commands::set_vector_consent,
            commands::check_keychain_available,
            commands::search_kb,
            commands::search_kb_with_options,
            commands::get_search_context,
            // LLM commands
            commands::init_llm_engine,
            commands::load_model,
            commands::load_custom_model,
            commands::validate_gguf_file,
            commands::unload_model,
            commands::get_model_info,
            commands::is_model_loaded,
            commands::generate_text,
            commands::generate_with_context,
            commands::generate_streaming,
            commands::generate_first_response,
            commands::generate_troubleshooting_checklist,
            commands::update_troubleshooting_checklist,
            commands::test_model,
            commands::cancel_generation,
            commands::get_context_window,
            commands::set_context_window,
            // Download commands
            commands::get_recommended_models,
            commands::list_downloaded_models,
            commands::get_models_dir,
            commands::delete_downloaded_model,
            commands::has_hf_token,
            commands::set_hf_token,
            commands::clear_hf_token,
            commands::download_model,
            commands::cancel_download,
            // KB Indexer commands
            commands::set_kb_folder,
            commands::get_kb_folder,
            commands::index_kb,
            commands::get_kb_stats,
            commands::list_kb_documents,
            commands::remove_kb_document,
            commands::generate_kb_embeddings,
            // KB Watcher commands
            commands::start_kb_watcher,
            commands::stop_kb_watcher,
            commands::is_kb_watcher_running,
            // Embedding commands
            commands::init_embedding_engine,
            commands::load_embedding_model,
            commands::unload_embedding_model,
            commands::get_embedding_model_info,
            commands::is_embedding_model_loaded,
            commands::get_embedding_model_path,
            commands::is_embedding_model_downloaded,
            // Vector store commands
            commands::init_vector_store,
            commands::set_vector_enabled,
            commands::is_vector_enabled,
            commands::get_vector_stats,
            // OCR commands
            commands::process_ocr,
            commands::process_ocr_bytes,
            commands::is_ocr_available,
            // Decision Tree commands
            commands::list_decision_trees,
            commands::get_decision_tree,
            // Jira commands
            commands::is_jira_configured,
            commands::get_jira_config,
            commands::configure_jira,
            commands::clear_jira_config,
            commands::get_jira_ticket,
            commands::add_jira_comment,
            commands::push_draft_to_jira,
            // Export commands (Phase 18)
            commands::export_draft_formatted,
            commands::format_draft_for_clipboard,
            // Draft & Template commands
            commands::list_drafts,
            commands::search_drafts,
            commands::get_draft,
            commands::save_draft,
            commands::delete_draft,
            commands::list_autosaves,
            commands::cleanup_autosaves,
            commands::get_draft_versions,
            // Draft versioning commands (Phase 17)
            commands::create_draft_version,
            commands::list_draft_versions,
            commands::finalize_draft,
            commands::archive_draft,
            commands::update_draft_handoff,
            // Playbook commands (Phase 17)
            commands::list_playbooks,
            commands::get_playbook,
            commands::save_playbook,
            commands::use_playbook,
            commands::delete_playbook,
            // Action shortcut commands (Phase 17)
            commands::list_action_shortcuts,
            commands::get_action_shortcut,
            commands::save_action_shortcut,
            commands::delete_action_shortcut,
            commands::list_templates,
            commands::get_template,
            commands::save_template,
            commands::delete_template,
            // Custom variable commands
            commands::list_custom_variables,
            commands::get_custom_variable,
            commands::save_custom_variable,
            commands::delete_custom_variable,
            // Export commands
            commands::backup::export_draft,
            // Backup/Restore commands
            commands::backup::export_backup,
            commands::backup::preview_backup_import,
            commands::backup::import_backup,
            // Ingestion commands
            commands::ingest_url,
            commands::ingest_youtube,
            commands::ingest_github,
            commands::ingest_github_remote,
            commands::process_source_file,
            commands::set_github_token,
            commands::clear_github_token,
            commands::has_github_token,
            commands::get_audit_entries,
            commands::export_audit_log,
            // Namespace commands
            commands::list_namespaces,
            commands::list_namespaces_with_counts,
            commands::create_namespace,
            commands::rename_namespace,
            commands::delete_namespace,
            // Ingest source management commands
            commands::list_ingest_sources,
            commands::delete_ingest_source,
            commands::get_source_health,
            commands::retry_source,
            commands::mark_stale_sources,
            commands::get_document_chunks,
            commands::delete_kb_document,
            commands::clear_knowledge_data,
            commands::check_ytdlp_available,
            // Job commands
            commands::create_job,
            commands::list_jobs,
            commands::get_job,
            commands::cancel_job,
            commands::get_job_logs,
            commands::get_job_counts,
            commands::cleanup_old_jobs,
            // Document versioning commands (Phase 14)
            commands::list_document_versions,
            commands::rollback_document,
            // Source trust commands (Phase 14)
            commands::update_source_trust,
            commands::set_source_pinned,
            commands::set_source_review_status,
            commands::get_stale_sources,
            // Namespace rules commands (Phase 14)
            commands::add_namespace_rule,
            commands::delete_namespace_rule,
            commands::list_namespace_rules,
            // Diagnostics commands
            commands::diagnostics::get_system_health,
            commands::diagnostics::repair_database_cmd,
            commands::diagnostics::rebuild_vector_store,
            commands::diagnostics::get_failure_modes_cmd,
            commands::diagnostics::run_quick_health_check,
            commands::diagnostics::get_database_stats_cmd,
            commands::diagnostics::run_database_maintenance_cmd,
            commands::diagnostics::get_resource_metrics_cmd,
            commands::diagnostics::get_llm_resource_limits,
            commands::diagnostics::set_llm_resource_limits,
            commands::diagnostics::get_vector_maintenance_info_cmd,
            // Phase 4: Response Rating commands
            commands::rate_response,
            commands::get_draft_rating,
            commands::get_rating_stats,
            // Phase 2: Analytics commands
            commands::log_analytics_event,
            commands::get_analytics_summary,
            commands::get_kb_usage_stats,
            commands::get_low_rating_analysis,
            // Phase 10: KB Management commands
            commands::update_chunk_content,
            commands::get_kb_health_stats,
            // Phase 6: Draft Version Restore
            commands::restore_draft_version,
            // Phase 9: Batch Processing commands
            commands::batch_generate,
            commands::get_batch_status,
            commands::export_batch_results,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use crate::security::KeychainManager;

    #[test]
    fn test_keychain_available() {
        // This will vary by environment
        let available = KeychainManager::is_available();
        println!("Keychain available: {}", available);
    }
}
