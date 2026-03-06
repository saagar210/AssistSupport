use super::*;

// KB Indexer Commands
// ============================================================================

use crate::kb::indexer::{IndexResult, IndexStats};

/// Set the KB folder path
/// Path must be within user's home directory (auto-creates if needed)
/// Blocks sensitive directories like .ssh, .aws, .gnupg, .config
#[tauri::command]
pub fn set_kb_folder(state: State<'_, AppState>, folder_path: String) -> Result<(), String> {
    kb_commands::set_kb_folder_impl(state, folder_path)
}

/// Get the current KB folder path
#[tauri::command]
pub fn get_kb_folder(state: State<'_, AppState>) -> Result<Option<String>, String> {
    kb_commands::get_kb_folder_impl(state)
}

/// Index the KB folder with progress events
#[tauri::command]
pub async fn index_kb(
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<IndexResult, String> {
    kb_commands::index_kb_impl(window, state).await
}

/// Get KB statistics
#[tauri::command]
pub fn get_kb_stats(state: State<'_, AppState>) -> Result<IndexStats, String> {
    kb_commands::get_kb_stats_impl(state)
}

/// List indexed KB documents, optionally filtered by namespace and/or source
#[tauri::command]
pub fn list_kb_documents(
    state: State<'_, AppState>,
    namespace_id: Option<String>,
    source_id: Option<String>,
) -> Result<Vec<KbDocumentInfo>, String> {
    kb_commands::list_kb_documents_impl(state, namespace_id, source_id)
}

/// KB document info for API responses
#[derive(serde::Serialize)]
pub struct KbDocumentInfo {
    pub id: String,
    pub file_path: String,
    pub title: Option<String>,
    pub indexed_at: Option<String>,
    pub chunk_count: Option<i64>,
    pub namespace_id: String,
    pub source_type: String,
    pub source_id: Option<String>,
}

/// Remove a document from the KB index
#[tauri::command]
pub fn remove_kb_document(file_path: String, state: State<'_, AppState>) -> Result<bool, String> {
    kb_commands::remove_kb_document_impl(file_path, state)
}

/// Start watching KB folder for changes
#[tauri::command]
pub async fn start_kb_watcher(
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    kb_commands::start_kb_watcher_impl(window, state).await
}

/// Stop watching KB folder
#[tauri::command]
pub fn stop_kb_watcher() -> Result<bool, String> {
    kb_commands::stop_kb_watcher_impl()
}

/// Check if KB watcher is running
#[tauri::command]
pub fn is_kb_watcher_running() -> Result<bool, String> {
    kb_commands::is_kb_watcher_running_impl()
}

/// Generate embeddings for all KB chunks
/// This should be called after indexing if vector search is enabled
#[tauri::command]
pub async fn generate_kb_embeddings(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<EmbeddingGenerationResult, String> {
    kb_commands::generate_kb_embeddings_impl(state, app_handle).await
}

/// Result of embedding generation
#[derive(serde::Serialize)]
pub struct EmbeddingGenerationResult {
    pub chunks_processed: usize,
    pub vectors_created: usize,
}

// ============================================================================
// Embedding Commands
// ============================================================================

use crate::kb::embeddings::EmbeddingModelInfo;

/// Initialize the embedding engine (idempotent — skips if already initialized)
#[tauri::command]
pub fn init_embedding_engine(state: State<'_, AppState>) -> Result<(), String> {
    model_commands::init_embedding_engine_impl(state)
}

/// Load an embedding model from file
#[tauri::command]
pub fn load_embedding_model(
    state: State<'_, AppState>,
    path: String,
    n_gpu_layers: Option<u32>,
) -> Result<EmbeddingModelInfo, String> {
    model_commands::load_embedding_model_impl(state, path, n_gpu_layers)
}

/// Unload the current embedding model
#[tauri::command]
pub fn unload_embedding_model(state: State<'_, AppState>) -> Result<(), String> {
    model_commands::unload_embedding_model_impl(state)
}

/// Get current embedding model info
#[tauri::command]
pub fn get_embedding_model_info(
    state: State<'_, AppState>,
) -> Result<Option<EmbeddingModelInfo>, String> {
    model_commands::get_embedding_model_info_impl(state)
}

/// Check if an embedding model is loaded
#[tauri::command]
pub fn is_embedding_model_loaded(state: State<'_, AppState>) -> Result<bool, String> {
    model_commands::is_embedding_model_loaded_impl(state)
}

// ============================================================================
// Vector Store Commands
// ============================================================================

/// Initialize the vector store
#[tauri::command]
pub async fn init_vector_store(state: State<'_, AppState>) -> Result<(), String> {
    kb_commands::init_vector_store_impl(state).await
}

/// Enable or disable vector search
#[tauri::command]
pub async fn set_vector_enabled(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    kb_commands::set_vector_enabled_impl(state, enabled).await
}

/// Check if vector store is enabled
#[tauri::command]
pub async fn is_vector_enabled(state: State<'_, AppState>) -> Result<bool, String> {
    kb_commands::is_vector_enabled_impl(state).await
}

/// Get vector store statistics
#[tauri::command]
pub async fn get_vector_stats(state: State<'_, AppState>) -> Result<VectorStats, String> {
    kb_commands::get_vector_stats_impl(state).await
}

/// Vector store statistics
#[derive(serde::Serialize)]
pub struct VectorStats {
    pub enabled: bool,
    pub vector_count: usize,
    pub embedding_dim: usize,
    pub encryption_supported: bool,
}

// ============================================================================
// OCR Commands
// ============================================================================

/// OCR result
#[derive(serde::Serialize)]
pub struct OcrResult {
    pub text: String,
    pub confidence: f32,
}

/// Process an image with OCR to extract text
#[tauri::command]
pub fn process_ocr(image_path: String) -> Result<OcrResult, String> {
    kb_commands::process_ocr_impl(image_path)
}

/// Process OCR from base64-encoded image data (for clipboard paste)
#[tauri::command]
pub fn process_ocr_bytes(image_base64: String) -> Result<OcrResult, String> {
    kb_commands::process_ocr_bytes_impl(image_base64)
}

/// Check if OCR is available on this system
#[tauri::command]
pub fn is_ocr_available() -> bool {
    kb_commands::is_ocr_available_impl()
}

// ============================================================================
// Decision Tree Commands
// ============================================================================

/// List all decision trees
#[tauri::command]
pub fn list_decision_trees(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::DecisionTree>, String> {
    draft_commands::list_decision_trees_impl(state)
}

/// Get a single decision tree by ID
#[tauri::command]
pub fn get_decision_tree(
    state: State<'_, AppState>,
    tree_id: String,
) -> Result<crate::db::DecisionTree, String> {
    draft_commands::get_decision_tree_impl(state, tree_id)
}

// ============================================================================
// Jira Integration Commands
// ============================================================================

use crate::jira::{JiraConfig, JiraTicket};

/// Check if Jira is configured
#[tauri::command]
pub fn is_jira_configured(state: State<'_, AppState>) -> Result<bool, String> {
    jira_commands::is_jira_configured_impl(state)
}

/// Get Jira configuration (without token)
#[tauri::command]
pub fn get_jira_config(state: State<'_, AppState>) -> Result<Option<JiraConfig>, String> {
    jira_commands::get_jira_config_impl(state)
}

/// Configure Jira (tests connection before saving)
/// HTTPS is required by default. HTTP can only be used with explicit opt-in
/// (allow_http = true), which triggers a security audit log entry.
#[tauri::command]
pub async fn configure_jira(
    state: State<'_, AppState>,
    base_url: String,
    email: String,
    api_token: String,
    allow_http: Option<bool>,
) -> Result<(), String> {
    jira_commands::configure_jira_impl(state, base_url, email, api_token, allow_http).await
}

/// Clear Jira configuration
#[tauri::command]
pub fn clear_jira_config(state: State<'_, AppState>) -> Result<(), String> {
    jira_commands::clear_jira_config_impl(state)
}

/// Get a Jira ticket by key
#[tauri::command]
pub async fn get_jira_ticket(
    state: State<'_, AppState>,
    ticket_key: String,
) -> Result<JiraTicket, String> {
    jira_commands::get_jira_ticket_impl(state, ticket_key).await
}

/// Add a comment to a Jira ticket (Phase 18)
#[tauri::command]
pub async fn add_jira_comment(
    state: State<'_, AppState>,
    ticket_key: String,
    comment_body: String,
    visibility: Option<String>,
) -> Result<String, String> {
    jira_commands::add_jira_comment_impl(state, ticket_key, comment_body, visibility).await
}

/// Push draft to Jira as a comment with KB citations (Phase 18)
#[tauri::command]
pub async fn push_draft_to_jira(
    state: State<'_, AppState>,
    draft_id: String,
    ticket_key: String,
    visibility: Option<String>,
) -> Result<String, String> {
    jira_commands::push_draft_to_jira_impl(state, draft_id, ticket_key, visibility).await
}

// ============================================================================
// Export Commands (Phase 18)
// ============================================================================

/// Export a draft in various formats
#[tauri::command]
pub fn export_draft_formatted(
    state: State<'_, AppState>,
    draft_id: String,
    format: String,
    safe_export: Option<crate::exports::SafeExportOptions>,
) -> Result<String, String> {
    draft_commands::export_draft_formatted_impl(state, draft_id, format, safe_export)
}

/// Format draft for clipboard (optimized for ticket systems)
#[tauri::command]
pub fn format_draft_for_clipboard(
    state: State<'_, AppState>,
    draft_id: String,
    include_sources: bool,
) -> Result<String, String> {
    draft_commands::format_draft_for_clipboard_impl(state, draft_id, include_sources)
}

// ============================================================================
// Draft & Template Commands
// ============================================================================

use crate::db::{ResponseTemplate, SavedDraft};

/// List saved drafts (most recent first)
#[tauri::command]
pub fn list_drafts(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<SavedDraft>, String> {
    draft_commands::list_drafts_impl(state, limit)
}

/// Search drafts by text content
#[tauri::command]
pub fn search_drafts(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<SavedDraft>, String> {
    draft_commands::search_drafts_impl(state, query, limit)
}

/// Get a single draft by ID
#[tauri::command]
pub fn get_draft(state: State<'_, AppState>, draft_id: String) -> Result<SavedDraft, String> {
    draft_commands::get_draft_impl(state, draft_id)
}

/// Save a draft (insert or update)
#[tauri::command]
pub fn save_draft(state: State<'_, AppState>, draft: SavedDraft) -> Result<String, String> {
    draft_commands::save_draft_impl(state, draft)
}

/// Delete a draft by ID
#[tauri::command]
pub fn delete_draft(state: State<'_, AppState>, draft_id: String) -> Result<(), String> {
    draft_commands::delete_draft_impl(state, draft_id)
}

/// List autosave drafts (most recent first)
#[tauri::command]
pub fn list_autosaves(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<SavedDraft>, String> {
    draft_commands::list_autosaves_impl(state, limit)
}

/// Cleanup old autosaves, keeping only the most recent ones
#[tauri::command]
pub fn cleanup_autosaves(
    state: State<'_, AppState>,
    keep_count: Option<usize>,
) -> Result<usize, String> {
    draft_commands::cleanup_autosaves_impl(state, keep_count)
}

/// Get draft versions by input hash (autosaves with matching input_text hash)
/// Used for version history UI
#[tauri::command]
pub fn get_draft_versions(
    state: State<'_, AppState>,
    input_hash: String,
) -> Result<Vec<SavedDraft>, String> {
    draft_commands::get_draft_versions_impl(state, input_hash)
}

// ============================================================================
// Draft Versioning Commands (Phase 17)
// ============================================================================

/// Create a draft version snapshot
#[tauri::command]
pub fn create_draft_version(
    state: State<'_, AppState>,
    draft_id: String,
    change_reason: Option<String>,
) -> Result<String, String> {
    draft_commands::create_draft_version_impl(state, draft_id, change_reason)
}

/// List draft versions for a specific draft
#[tauri::command]
pub fn list_draft_versions(
    state: State<'_, AppState>,
    draft_id: String,
) -> Result<Vec<crate::db::DraftVersion>, String> {
    draft_commands::list_draft_versions_impl(state, draft_id)
}

/// Finalize a draft (lock and mark as read-only)
#[tauri::command]
pub fn finalize_draft(
    state: State<'_, AppState>,
    draft_id: String,
    finalized_by: Option<String>,
) -> Result<(), String> {
    draft_commands::finalize_draft_impl(state, draft_id, finalized_by)
}

/// Archive a draft
#[tauri::command]
pub fn archive_draft(state: State<'_, AppState>, draft_id: String) -> Result<(), String> {
    draft_commands::archive_draft_impl(state, draft_id)
}

/// Update draft handoff summary for escalations
#[tauri::command]
pub fn update_draft_handoff(
    state: State<'_, AppState>,
    draft_id: String,
    handoff_summary: String,
) -> Result<(), String> {
    draft_commands::update_draft_handoff_impl(state, draft_id, handoff_summary)
}

// ============================================================================
// Playbook Commands (Phase 17)
// ============================================================================

/// List all active playbooks
#[tauri::command]
pub fn list_playbooks(
    state: State<'_, AppState>,
    category: Option<String>,
) -> Result<Vec<crate::db::Playbook>, String> {
    draft_commands::list_playbooks_impl(state, category)
}

/// Get a playbook by ID
#[tauri::command]
pub fn get_playbook(
    state: State<'_, AppState>,
    playbook_id: String,
) -> Result<crate::db::Playbook, String> {
    draft_commands::get_playbook_impl(state, playbook_id)
}

/// Save a playbook (insert or update)
#[tauri::command]
pub fn save_playbook(
    state: State<'_, AppState>,
    playbook: crate::db::Playbook,
) -> Result<String, String> {
    draft_commands::save_playbook_impl(state, playbook)
}

/// Record playbook usage
#[tauri::command]
pub fn use_playbook(state: State<'_, AppState>, playbook_id: String) -> Result<(), String> {
    draft_commands::use_playbook_impl(state, playbook_id)
}

/// Delete a playbook
#[tauri::command]
pub fn delete_playbook(state: State<'_, AppState>, playbook_id: String) -> Result<(), String> {
    draft_commands::delete_playbook_impl(state, playbook_id)
}

// ============================================================================
// Action Shortcut Commands (Phase 17)
// ============================================================================

/// List all active action shortcuts
#[tauri::command]
pub fn list_action_shortcuts(
    state: State<'_, AppState>,
    category: Option<String>,
) -> Result<Vec<crate::db::ActionShortcut>, String> {
    draft_commands::list_action_shortcuts_impl(state, category)
}

/// Get an action shortcut by ID
#[tauri::command]
pub fn get_action_shortcut(
    state: State<'_, AppState>,
    shortcut_id: String,
) -> Result<crate::db::ActionShortcut, String> {
    draft_commands::get_action_shortcut_impl(state, shortcut_id)
}

/// Save an action shortcut (insert or update)
#[tauri::command]
pub fn save_action_shortcut(
    state: State<'_, AppState>,
    shortcut: crate::db::ActionShortcut,
) -> Result<String, String> {
    draft_commands::save_action_shortcut_impl(state, shortcut)
}

/// Delete an action shortcut
#[tauri::command]
pub fn delete_action_shortcut(
    state: State<'_, AppState>,
    shortcut_id: String,
) -> Result<(), String> {
    draft_commands::delete_action_shortcut_impl(state, shortcut_id)
}

/// List all response templates
#[tauri::command]
pub fn list_templates(state: State<'_, AppState>) -> Result<Vec<ResponseTemplate>, String> {
    draft_commands::list_templates_impl(state)
}

/// Get a single template by ID
#[tauri::command]
pub fn get_template(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<ResponseTemplate, String> {
    draft_commands::get_template_impl(state, template_id)
}

/// Save a template (insert or update)
#[tauri::command]
pub fn save_template(
    state: State<'_, AppState>,
    template: ResponseTemplate,
) -> Result<String, String> {
    draft_commands::save_template_impl(state, template)
}

/// Delete a template by ID
#[tauri::command]
pub fn delete_template(state: State<'_, AppState>, template_id: String) -> Result<(), String> {
    draft_commands::delete_template_impl(state, template_id)
}

// ============================================================================
// Custom Variable Commands
// ============================================================================

/// List all custom template variables
#[tauri::command]
pub fn list_custom_variables(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::CustomVariable>, String> {
    draft_commands::list_custom_variables_impl(state)
}

/// Get a custom variable by ID
#[tauri::command]
pub fn get_custom_variable(
    state: State<'_, AppState>,
    variable_id: String,
) -> Result<crate::db::CustomVariable, String> {
    draft_commands::get_custom_variable_impl(state, variable_id)
}

/// Save a custom variable (create or update)
#[tauri::command]
pub fn save_custom_variable(
    state: State<'_, AppState>,
    variable: crate::db::CustomVariable,
) -> Result<(), String> {
    draft_commands::save_custom_variable_impl(state, variable)
}

/// Delete a custom variable by ID
#[tauri::command]
pub fn delete_custom_variable(
    state: State<'_, AppState>,
    variable_id: String,
) -> Result<(), String> {
    draft_commands::delete_custom_variable_impl(state, variable_id)
}

// Export and Backup commands moved to commands/backup.rs

// =============================================================================
// CONTENT INGESTION COMMANDS
// =============================================================================

/// Result of an ingestion operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct IngestResult {
    pub document_id: String,
    pub title: String,
    pub source_uri: String,
    pub chunk_count: usize,
    pub word_count: usize,
}

/// Result of a batch ingestion operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct BatchIngestResult {
    pub successful: Vec<IngestResult>,
    pub failed: Vec<FailedSource>,
    pub cancelled: bool,
}

/// A failed source in a batch operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct FailedSource {
    pub source: String,
    pub error: String,
}

/// Result of a disk folder ingestion
#[derive(Debug, Clone, serde::Serialize)]
pub struct DiskIngestResultResponse {
    pub total_files: usize,
    pub ingested: usize,
    pub skipped: usize,
    pub errors: usize,
    pub documents: Vec<IngestResult>,
}

/// Ingest a folder of documents from disk with source tracking
/// Creates ingest_sources and ingest_runs entries so disk-indexed
/// articles appear in the source management UI
#[tauri::command]
pub fn ingest_kb_from_disk(
    state: State<'_, AppState>,
    folder_path: String,
    namespace_id: String,
) -> Result<DiskIngestResultResponse, String> {
    ingestion_commands::ingest_kb_from_disk_impl(state, folder_path, namespace_id)
}

/// Ingest a web page URL
/// Uses block_in_place to run async operations while holding DB lock
#[tauri::command]
pub fn ingest_url(
    state: State<'_, AppState>,
    url: String,
    namespace_id: String,
) -> Result<IngestResult, String> {
    ingestion_commands::ingest_url_impl(state, url, namespace_id)
}

/// Ingest a YouTube video transcript
/// Uses block_in_place to run async operations while holding DB lock
#[tauri::command]
pub fn ingest_youtube(
    state: State<'_, AppState>,
    url: String,
    namespace_id: String,
) -> Result<IngestResult, String> {
    ingestion_commands::ingest_youtube_impl(state, url, namespace_id)
}

/// Ingest a GitHub repository (local path)
/// Path must be within user's home directory
#[tauri::command]
pub fn ingest_github(
    state: State<'_, AppState>,
    repo_path: String,
    namespace_id: String,
) -> Result<Vec<IngestResult>, String> {
    ingestion_commands::ingest_github_impl(state, repo_path, namespace_id)
}

/// Ingest a GitHub repository from a remote HTTPS URL
#[tauri::command]
pub fn ingest_github_remote(
    state: State<'_, AppState>,
    repo_url: String,
    namespace_id: String,
) -> Result<Vec<IngestResult>, String> {
    ingestion_commands::ingest_github_remote_impl(state, repo_url, namespace_id)
}

/// Process a YAML source file for batch ingestion
/// Uses block_in_place to run async operations while holding DB lock
#[tauri::command]
pub fn process_source_file(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<BatchIngestResult, String> {
    ingestion_commands::process_source_file_impl(state, file_path)
}

/// List all namespaces
#[tauri::command]
pub fn list_namespaces(state: State<'_, AppState>) -> Result<Vec<crate::db::Namespace>, String> {
    ingestion_commands::list_namespaces_impl(state)
}

/// List all namespaces with document and source counts (optimized single query)
#[tauri::command]
pub fn list_namespaces_with_counts(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::NamespaceWithCounts>, String> {
    ingestion_commands::list_namespaces_with_counts_impl(state)
}

/// Create a new namespace
#[tauri::command]
pub fn create_namespace(
    state: State<'_, AppState>,
    name: String,
    description: Option<String>,
    color: Option<String>,
) -> Result<crate::db::Namespace, String> {
    ingestion_commands::create_namespace_impl(state, name, description, color)
}

/// Rename a namespace
#[tauri::command]
pub fn rename_namespace(
    state: State<'_, AppState>,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    ingestion_commands::rename_namespace_impl(state, old_name, new_name)
}

/// Delete a namespace and all its content
#[tauri::command]
pub fn delete_namespace(state: State<'_, AppState>, name: String) -> Result<(), String> {
    ingestion_commands::delete_namespace_impl(state, name)
}

/// List ingestion sources, optionally filtered by namespace
#[tauri::command]
pub fn list_ingest_sources(
    state: State<'_, AppState>,
    namespace_id: Option<String>,
) -> Result<Vec<crate::db::IngestSource>, String> {
    ingestion_commands::list_ingest_sources_impl(state, namespace_id)
}

/// Delete an ingestion source and its documents
#[tauri::command]
pub fn delete_ingest_source(state: State<'_, AppState>, source_id: String) -> Result<(), String> {
    ingestion_commands::delete_ingest_source_impl(state, source_id)
}

/// Get source health summary for a namespace
#[derive(serde::Serialize)]
pub struct SourceHealthSummary {
    pub total_sources: u32,
    pub active_sources: u32,
    pub stale_sources: u32,
    pub error_sources: u32,
    pub pending_sources: u32,
    pub sources: Vec<SourceHealth>,
}

#[derive(serde::Serialize)]
pub struct SourceHealth {
    pub id: String,
    pub source_type: String,
    pub source_uri: String,
    pub title: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub last_ingested_at: Option<String>,
    pub document_count: u32,
    pub days_since_refresh: Option<i64>,
}

#[tauri::command]
pub fn get_source_health(
    state: State<'_, AppState>,
    namespace_id: Option<String>,
) -> Result<SourceHealthSummary, String> {
    ingestion_commands::get_source_health_impl(state, namespace_id)
}

/// Retry a failed or stale source
#[tauri::command]
pub fn retry_source(state: State<'_, AppState>, source_id: String) -> Result<IngestResult, String> {
    ingestion_commands::retry_source_impl(state, source_id)
}

/// Mark sources as stale if they haven't been refreshed in N days
#[tauri::command]
pub fn mark_stale_sources(
    state: State<'_, AppState>,
    days_threshold: Option<u32>,
) -> Result<u32, String> {
    ingestion_commands::mark_stale_sources_impl(state, days_threshold)
}

/// Get document chunks
#[tauri::command]
pub fn get_document_chunks(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<Vec<DocumentChunk>, String> {
    ingestion_commands::get_document_chunks_impl(state, document_id)
}

/// A document chunk for API responses
#[derive(Debug, Clone, serde::Serialize)]
pub struct DocumentChunk {
    pub id: String,
    pub chunk_index: i32,
    pub heading_path: Option<String>,
    pub content: String,
    pub word_count: Option<i32>,
}

/// Delete a specific document
#[tauri::command]
pub fn delete_kb_document(state: State<'_, AppState>, document_id: String) -> Result<(), String> {
    ingestion_commands::delete_kb_document_impl(state, document_id)
}

/// Clear all knowledge data, optionally for a specific namespace
#[tauri::command]
pub fn clear_knowledge_data(
    state: State<'_, AppState>,
    namespace_id: Option<String>,
) -> Result<(), String> {
    ingestion_commands::clear_knowledge_data_impl(state, namespace_id)
}

/// Check if yt-dlp is available
#[tauri::command]
pub fn check_ytdlp_available() -> Result<bool, String> {
    ingestion_commands::check_ytdlp_available_impl()
}
