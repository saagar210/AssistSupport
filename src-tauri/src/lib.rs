//! AssistSupport - Self-contained local KB + LLM app for IT support

pub mod audit;
pub mod backup;
pub mod commands;
pub mod db;
pub mod diagnostics;
pub mod downloads;
pub mod error;
pub mod exports;
pub mod feedback;
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

#[cfg(target_os = "macos")]
fn configure_ggml_metal_env() {
    // Work around a class of macOS Metal crashes/aborts observed in ggml's
    // residency-set teardown (ggml_metal_rsets_free). If the user explicitly set
    // the env var we respect it; otherwise default to the safer setting.
    if std::env::var_os("GGML_METAL_NO_RESIDENCY").is_none() {
        std::env::set_var("GGML_METAL_NO_RESIDENCY", "1");
    }
}

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
    #[cfg(target_os = "macos")]
    configure_ggml_metal_env();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::generation_commands::greet,
            commands::lifecycle_commands::initialize_app,
            commands::lifecycle_commands::check_fts5_enabled,
            commands::lifecycle_commands::check_db_integrity,
            commands::lifecycle_commands::get_vector_consent,
            commands::lifecycle_commands::set_vector_consent,
            commands::lifecycle_commands::check_keychain_available,
            commands::generation_commands::search_kb,
            commands::generation_commands::search_kb_with_options,
            commands::generation_commands::get_search_context,
            // LLM commands
            commands::generation_commands::init_llm_engine,
            commands::generation_commands::load_model,
            commands::generation_commands::load_custom_model,
            commands::generation_commands::validate_gguf_file,
            commands::generation_commands::unload_model,
            commands::generation_commands::get_model_info,
            commands::generation_commands::is_model_loaded,
            commands::generation_commands::generate_text,
            commands::generation_commands::generate_with_context,
            commands::generation_commands::generate_streaming,
            commands::generation_commands::generate_first_response,
            commands::generation_commands::generate_troubleshooting_checklist,
            commands::generation_commands::update_troubleshooting_checklist,
            commands::generation_commands::test_model,
            commands::generation_commands::cancel_generation,
            commands::platform_commands::get_context_window,
            commands::platform_commands::set_context_window,
            // Download commands
            commands::platform_commands::get_recommended_models,
            commands::platform_commands::list_downloaded_models,
            commands::platform_commands::get_models_dir,
            commands::platform_commands::delete_downloaded_model,
            commands::platform_commands::has_hf_token,
            commands::platform_commands::set_hf_token,
            commands::platform_commands::clear_hf_token,
            commands::platform_commands::has_search_api_token,
            commands::platform_commands::set_search_api_token,
            commands::platform_commands::clear_search_api_token,
            commands::platform_commands::has_memorykernel_service_token,
            commands::platform_commands::set_memorykernel_service_token,
            commands::platform_commands::clear_memorykernel_service_token,
            commands::platform_commands::download_model,
            commands::platform_commands::cancel_download,
            // KB Indexer commands
            commands::workflow_commands::set_kb_folder,
            commands::workflow_commands::get_kb_folder,
            commands::workflow_commands::index_kb,
            commands::workflow_commands::get_kb_stats,
            commands::workflow_commands::list_kb_documents,
            commands::workflow_commands::remove_kb_document,
            commands::workflow_commands::generate_kb_embeddings,
            // KB Watcher commands
            commands::workflow_commands::start_kb_watcher,
            commands::workflow_commands::stop_kb_watcher,
            commands::workflow_commands::is_kb_watcher_running,
            // Embedding commands
            commands::workflow_commands::init_embedding_engine,
            commands::workflow_commands::load_embedding_model,
            commands::workflow_commands::unload_embedding_model,
            commands::workflow_commands::get_embedding_model_info,
            commands::workflow_commands::is_embedding_model_loaded,
            commands::platform_commands::get_embedding_model_path,
            commands::platform_commands::is_embedding_model_downloaded,
            // Vector store commands
            commands::workflow_commands::init_vector_store,
            commands::workflow_commands::set_vector_enabled,
            commands::workflow_commands::is_vector_enabled,
            commands::workflow_commands::get_vector_stats,
            // OCR commands
            commands::workflow_commands::process_ocr,
            commands::workflow_commands::process_ocr_bytes,
            commands::workflow_commands::is_ocr_available,
            // Decision Tree commands
            commands::workflow_commands::list_decision_trees,
            commands::workflow_commands::get_decision_tree,
            // Jira commands
            commands::workflow_commands::is_jira_configured,
            commands::workflow_commands::get_jira_config,
            commands::workflow_commands::configure_jira,
            commands::workflow_commands::clear_jira_config,
            commands::workflow_commands::get_jira_ticket,
            commands::workflow_commands::add_jira_comment,
            commands::workflow_commands::push_draft_to_jira,
            // Export commands (Phase 18)
            commands::workflow_commands::export_draft_formatted,
            commands::workflow_commands::format_draft_for_clipboard,
            // Draft & Template commands
            commands::workflow_commands::list_drafts,
            commands::workflow_commands::search_drafts,
            commands::workflow_commands::get_draft,
            commands::workflow_commands::save_draft,
            commands::workflow_commands::delete_draft,
            commands::workflow_commands::list_autosaves,
            commands::workflow_commands::cleanup_autosaves,
            commands::workflow_commands::get_draft_versions,
            // Draft versioning commands (Phase 17)
            commands::workflow_commands::create_draft_version,
            commands::workflow_commands::list_draft_versions,
            commands::workflow_commands::finalize_draft,
            commands::workflow_commands::archive_draft,
            commands::workflow_commands::update_draft_handoff,
            // Playbook commands (Phase 17)
            commands::workflow_commands::list_playbooks,
            commands::workflow_commands::get_playbook,
            commands::workflow_commands::save_playbook,
            commands::workflow_commands::use_playbook,
            commands::workflow_commands::delete_playbook,
            // Action shortcut commands (Phase 17)
            commands::workflow_commands::list_action_shortcuts,
            commands::workflow_commands::get_action_shortcut,
            commands::workflow_commands::save_action_shortcut,
            commands::workflow_commands::delete_action_shortcut,
            commands::workflow_commands::list_templates,
            commands::workflow_commands::get_template,
            commands::workflow_commands::save_template,
            commands::workflow_commands::delete_template,
            // Custom variable commands
            commands::workflow_commands::list_custom_variables,
            commands::workflow_commands::get_custom_variable,
            commands::workflow_commands::save_custom_variable,
            commands::workflow_commands::delete_custom_variable,
            // Export commands
            commands::backup::export_draft,
            // Backup/Restore commands
            commands::backup::export_backup,
            commands::backup::preview_backup_import,
            commands::backup::import_backup,
            // Ingestion commands
            commands::workflow_commands::ingest_kb_from_disk,
            commands::workflow_commands::ingest_url,
            commands::workflow_commands::ingest_youtube,
            commands::workflow_commands::ingest_github,
            commands::workflow_commands::ingest_github_remote,
            commands::workflow_commands::process_source_file,
            commands::platform_commands::set_github_token,
            commands::platform_commands::clear_github_token,
            commands::platform_commands::has_github_token,
            commands::platform_commands::get_audit_entries,
            commands::platform_commands::export_audit_log,
            commands::platform_commands::audit_response_copy_override,
            // Namespace commands
            commands::workflow_commands::list_namespaces,
            commands::workflow_commands::list_namespaces_with_counts,
            commands::workflow_commands::create_namespace,
            commands::workflow_commands::rename_namespace,
            commands::workflow_commands::delete_namespace,
            // Ingest source management commands
            commands::workflow_commands::list_ingest_sources,
            commands::workflow_commands::delete_ingest_source,
            commands::workflow_commands::get_source_health,
            commands::workflow_commands::retry_source,
            commands::workflow_commands::mark_stale_sources,
            commands::workflow_commands::get_document_chunks,
            commands::workflow_commands::delete_kb_document,
            commands::workflow_commands::clear_knowledge_data,
            commands::workflow_commands::check_ytdlp_available,
            // Job commands
            commands::ops_facade_commands::create_job,
            commands::ops_facade_commands::list_jobs,
            commands::ops_facade_commands::get_job,
            commands::ops_facade_commands::cancel_job,
            commands::ops_facade_commands::get_job_logs,
            commands::ops_facade_commands::get_job_counts,
            commands::ops_facade_commands::cleanup_old_jobs,
            // Document versioning commands (Phase 14)
            commands::ops_facade_commands::list_document_versions,
            commands::ops_facade_commands::rollback_document,
            // Source trust commands (Phase 14)
            commands::ops_facade_commands::update_source_trust,
            commands::ops_facade_commands::set_source_pinned,
            commands::ops_facade_commands::set_source_review_status,
            commands::ops_facade_commands::get_stale_sources,
            // Namespace rules commands (Phase 14)
            commands::ops_facade_commands::add_namespace_rule,
            commands::ops_facade_commands::delete_namespace_rule,
            commands::ops_facade_commands::list_namespace_rules,
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
            commands::ops_facade_commands::rate_response,
            commands::ops_facade_commands::get_draft_rating,
            commands::ops_facade_commands::get_rating_stats,
            // Phase 2: Analytics commands
            commands::ops_facade_commands::log_analytics_event,
            commands::ops_facade_commands::get_analytics_summary,
            commands::ops_facade_commands::get_response_quality_summary,
            commands::ops_facade_commands::get_response_quality_drilldown_examples,
            commands::ops_facade_commands::get_kb_usage_stats,
            commands::ops_facade_commands::get_low_rating_analysis,
            commands::ops_facade_commands::get_kb_gap_candidates,
            commands::ops_facade_commands::update_kb_gap_status,
            commands::ops_facade_commands::run_deployment_preflight,
            commands::ops_facade_commands::record_deployment_artifact,
            commands::ops_facade_commands::get_deployment_health_summary,
            commands::ops_facade_commands::list_deployment_artifacts,
            commands::ops_facade_commands::verify_signed_artifact,
            commands::ops_facade_commands::rollback_deployment_run,
            commands::ops_facade_commands::run_eval_harness,
            commands::ops_facade_commands::list_eval_runs,
            commands::ops_facade_commands::cluster_tickets_for_triage,
            commands::ops_facade_commands::list_recent_triage_clusters,
            commands::ops_facade_commands::start_runbook_session,
            commands::ops_facade_commands::advance_runbook_session,
            commands::ops_facade_commands::list_runbook_sessions,
            commands::ops_facade_commands::configure_integration,
            commands::ops_facade_commands::list_integrations,
            commands::ops_facade_commands::set_workspace_role,
            commands::ops_facade_commands::list_workspace_roles,
            // Phase 10: KB Management commands
            commands::ops_facade_commands::update_chunk_content,
            commands::ops_facade_commands::get_kb_health_stats,
            // Phase 6: Draft Version Restore
            commands::ops_facade_commands::restore_draft_version,
            // Phase 9: Batch Processing commands
            commands::ops_facade_commands::batch_generate,
            commands::ops_facade_commands::get_batch_status,
            commands::ops_facade_commands::export_batch_results,
            // Phase 2 v0.4.0: KB Staleness
            commands::ops_facade_commands::mark_document_reviewed,
            commands::ops_facade_commands::get_documents_needing_review,
            // Phase 2 v0.4.0: Actionable Analytics
            commands::ops_facade_commands::get_analytics_for_article,
            // Phase 2 v0.4.0: Saved Response Templates
            commands::ops_facade_commands::save_response_as_template,
            commands::ops_facade_commands::list_saved_response_templates,
            commands::ops_facade_commands::increment_saved_template_usage,
            commands::ops_facade_commands::find_similar_saved_responses,
            // Phase 2 v0.4.0: Response Alternatives
            commands::ops_facade_commands::save_response_alternative,
            commands::ops_facade_commands::get_alternatives_for_draft,
            commands::ops_facade_commands::choose_alternative,
            // Phase 2 v0.4.0: Jira Transitions
            commands::ops_facade_commands::get_jira_transitions,
            commands::ops_facade_commands::transition_jira_ticket,
            commands::ops_facade_commands::post_and_transition,
            // v0.4.1: Startup & Model State
            commands::platform_commands::get_model_state,
            commands::platform_commands::get_startup_metrics,
            // v0.6.0: Pilot Feedback
            commands::lifecycle_commands::get_pilot_logging_policy,
            commands::lifecycle_commands::log_pilot_query,
            commands::lifecycle_commands::submit_pilot_feedback,
            commands::lifecycle_commands::get_pilot_stats,
            commands::lifecycle_commands::get_pilot_query_logs,
            commands::lifecycle_commands::export_pilot_data,
            // PostgreSQL Hybrid Search API (Week 4)
            commands::search_api::hybrid_search,
            commands::search_api::submit_search_feedback,
            commands::search_api::get_search_api_stats,
            commands::search_api::get_search_api_health_status,
            commands::search_api::check_search_api_health,
            // MemoryKernel service integration
            commands::memory_kernel::get_memory_kernel_integration_pin,
            commands::memory_kernel::get_memory_kernel_preflight_status,
            commands::memory_kernel::memory_kernel_query_ask,
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
