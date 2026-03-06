//! Tauri commands for AssistSupport
//!
//! Commands are organized into domain-specific submodules:
//! - backup: Export, backup, and restore operations
//! - diagnostics: Health checks and repair operations
//!
//! This file contains the remaining commands that are being gradually migrated.

// Domain-specific command modules
pub mod backup;
pub mod diagnostics;
pub mod draft_commands;
pub mod generation_commands;
pub mod ingestion_commands;
pub mod jira_commands;
pub mod kb_commands;
pub mod lifecycle_commands;
pub mod memory_kernel;
pub mod model_commands;
pub mod ops_commands;
pub mod ops_facade_commands;
pub mod platform_commands;
pub mod search_api;
pub mod security_commands;
pub mod workflow_commands;

// Re-export commands from submodules
pub use backup::{export_backup, export_draft, import_backup, preview_backup_import, ExportFormat};
pub use diagnostics::{
    get_database_stats_cmd, get_failure_modes_cmd, get_llm_resource_limits,
    get_resource_metrics_cmd, get_system_health, get_vector_maintenance_info_cmd,
    rebuild_vector_store, repair_database_cmd, run_database_maintenance_cmd,
    run_quick_health_check, set_llm_resource_limits, QuickHealthResult,
};
pub use generation_commands::{
    cancel_generation, generate_first_response, generate_streaming, generate_text,
    generate_troubleshooting_checklist, generate_with_context, get_model_info, get_search_context,
    greet, init_llm_engine, is_model_loaded, load_custom_model, load_model, search_kb,
    search_kb_with_options, test_model, unload_model, update_troubleshooting_checklist,
    validate_gguf_file, ChecklistGenerateParams, ChecklistItem, ChecklistResult, ChecklistState,
    ChecklistUpdateParams, ConfidenceAssessment, ConfidenceMode, ContextSource,
    FirstResponseParams, FirstResponseResult, FirstResponseTone, GenerateParams,
    GenerateWithContextParams, GenerateWithContextResult, GenerationMetrics, GenerationResult,
    GgufFileInfo, GroundedClaim, InitResult, SearchOptionsParam, StreamToken, TestModelResult,
};
pub use lifecycle_commands::{
    check_db_integrity, check_fts5_enabled, check_keychain_available, export_pilot_data,
    get_pilot_logging_policy, get_pilot_query_logs, get_pilot_stats, get_vector_consent,
    initialize_app, log_pilot_query, set_vector_consent, submit_pilot_feedback, PilotLoggingPolicy,
};
pub use memory_kernel::{
    get_memory_kernel_integration_pin, get_memory_kernel_preflight_status, memory_kernel_query_ask,
    MemoryKernelEnrichmentResult, MemoryKernelIntegrationPin, MemoryKernelPreflightStatus,
};
pub use ops_commands::{
    BatchInput, BatchResult, BatchSource, BatchStatus, DeploymentPreflightResult, EvalHarnessCase,
    EvalHarnessResult, JobSummary, TriageClusterOutput, TriageTicketInput,
};
pub use ops_facade_commands::{
    add_namespace_rule, advance_runbook_session, batch_generate, choose_alternative,
    cleanup_old_jobs, cluster_tickets_for_triage, configure_integration, create_job,
    delete_namespace_rule, export_batch_results, find_similar_saved_responses,
    get_alternatives_for_draft, get_analytics_for_article, get_analytics_summary, get_batch_status,
    get_deployment_health_summary, get_documents_needing_review, get_draft_rating,
    get_jira_transitions, get_job, get_job_counts, get_job_logs, get_kb_gap_candidates,
    get_kb_health_stats, get_kb_usage_stats, get_low_rating_analysis, get_rating_stats,
    get_response_quality_drilldown_examples, get_response_quality_summary, get_stale_sources,
    increment_saved_template_usage, list_deployment_artifacts, list_eval_runs, list_integrations,
    list_jobs, list_namespace_rules, list_recent_triage_clusters, list_runbook_sessions,
    list_saved_response_templates, mark_document_reviewed, post_and_transition, rate_response,
    record_deployment_artifact, restore_draft_version, rollback_deployment_run, rollback_document,
    run_deployment_preflight, run_eval_harness, save_response_alternative,
    save_response_as_template, set_source_pinned, set_source_review_status, set_workspace_role,
    transition_jira_ticket, update_chunk_content, update_kb_gap_status, update_source_trust,
    verify_signed_artifact,
};
pub use platform_commands::{
    audit_response_copy_override, cancel_download, clear_github_token, clear_hf_token,
    clear_memorykernel_service_token, clear_search_api_token, delete_downloaded_model,
    download_model, export_audit_log, get_audit_entries, get_context_window,
    get_embedding_model_path, get_model_state, get_models_dir, get_recommended_models,
    get_startup_metrics, has_github_token, has_hf_token, has_memorykernel_service_token,
    has_search_api_token, is_embedding_model_downloaded, list_downloaded_models,
    set_context_window, set_github_token, set_hf_token, set_memorykernel_service_token,
    set_search_api_token, ModelStateResult, StartupMetricsResult,
};
pub use search_api::{
    check_search_api_health, get_search_api_health_status, get_search_api_stats, hybrid_search,
    submit_search_feedback, HybridSearchResponse, SearchApiHealthStatus, SearchApiStatsData,
};
pub use workflow_commands::{
    add_jira_comment, archive_draft, check_ytdlp_available, cleanup_autosaves, clear_jira_config,
    clear_knowledge_data, create_draft_version, create_namespace, delete_action_shortcut,
    delete_custom_variable, delete_draft, delete_ingest_source, delete_kb_document,
    delete_namespace, delete_playbook, delete_template, export_draft_formatted, finalize_draft,
    format_draft_for_clipboard, generate_kb_embeddings, get_action_shortcut, get_custom_variable,
    get_decision_tree, get_document_chunks, get_draft, get_draft_versions,
    get_embedding_model_info, get_jira_config, get_jira_ticket, get_kb_folder, get_kb_stats,
    get_playbook, get_source_health, get_template, get_vector_stats, index_kb, ingest_github,
    ingest_github_remote, ingest_kb_from_disk, ingest_url, ingest_youtube, init_embedding_engine,
    init_vector_store, is_embedding_model_loaded, is_jira_configured, is_kb_watcher_running,
    is_ocr_available, is_vector_enabled, list_action_shortcuts, list_autosaves,
    list_custom_variables, list_decision_trees, list_draft_versions, list_drafts,
    list_ingest_sources, list_kb_documents, list_namespaces, list_namespaces_with_counts,
    list_playbooks, list_templates, load_embedding_model, mark_stale_sources, process_ocr,
    process_ocr_bytes, process_source_file, push_draft_to_jira, remove_kb_document,
    rename_namespace, retry_source, save_action_shortcut, save_custom_variable, save_draft,
    save_playbook, save_template, search_drafts, set_kb_folder, set_vector_enabled,
    start_kb_watcher, stop_kb_watcher, unload_embedding_model, update_draft_handoff, use_playbook,
    BatchIngestResult, DiskIngestResultResponse, DocumentChunk, EmbeddingGenerationResult,
    FailedSource, IngestResult, KbDocumentInfo, OcrResult, SourceHealth, SourceHealthSummary,
    VectorStats,
};

use crate::audit::{self, AuditLogger};
use crate::db::{get_app_data_dir, get_db_path, get_vectors_dir, Database, GenerationQualityEvent};
use crate::kb::vectors::{VectorStore, VectorStoreConfig};
use crate::llm::{GenerationParams, LlmEngine, ModelInfo};
use crate::security::{
    FileKeyStore, KeyStorageMode, TOKEN_HUGGINGFACE, TOKEN_JIRA, TOKEN_MEMORYKERNEL_SERVICE,
    TOKEN_SEARCH_API,
};
use crate::validation::{
    is_http_url, normalize_and_validate_namespace_id, validate_non_empty,
    validate_output_file_within_home, validate_text_size, validate_ticket_id, validate_within_home,
    ValidationError, MAX_QUERY_BYTES, MAX_TEXT_INPUT_BYTES,
};
use crate::AppState;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Emitter, State};
use tokio::sync::mpsc;

/// Global cancel flag for generation - shared between generate and cancel commands
static GENERATION_CANCEL_FLAG: Lazy<Arc<AtomicBool>> =
    Lazy::new(|| Arc::new(AtomicBool::new(false)));
static DOWNLOAD_CANCEL_FLAG: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));
const GITHUB_TOKEN_PREFIX: &str = "github_token:";

fn normalize_github_host(host: &str) -> Result<String, String> {
    let trimmed = host.trim();
    if trimmed.is_empty() {
        return Err("GitHub host cannot be empty".to_string());
    }
    if trimmed.contains("://") || trimmed.contains('/') {
        return Err("GitHub host must be a hostname (no scheme or path)".to_string());
    }

    let re =
        regex_lite::Regex::new(r"^[A-Za-z0-9.-]+(:[0-9]{1,5})?$").map_err(|e| e.to_string())?;
    if !re.is_match(trimmed) {
        return Err("GitHub host contains invalid characters".to_string());
    }

    Ok(trimmed.to_lowercase())
}

// Search and generation commands moved to commands/generation_commands.rs
// Model/download/security facade commands moved to commands/platform_commands.rs
// KB/Jira/Draft/Ingestion facade commands moved to commands/workflow_commands.rs
// Job Commands
// ============================================================================

// Job/ops/jira-transition facade commands moved to commands/ops_facade_commands.rs
