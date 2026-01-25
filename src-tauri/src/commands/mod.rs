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

// Re-export commands from submodules
pub use backup::{export_backup, export_draft, import_backup, preview_backup_import, ExportFormat};
pub use diagnostics::{
    get_database_stats_cmd, get_failure_modes_cmd, get_llm_resource_limits,
    get_resource_metrics_cmd, get_system_health, get_vector_maintenance_info_cmd,
    rebuild_vector_store, repair_database_cmd, run_database_maintenance_cmd,
    run_quick_health_check, set_llm_resource_limits, QuickHealthResult,
};

use crate::audit::{self, AuditLogger};
use crate::db::{Database, get_db_path, get_app_data_dir, get_vectors_dir};
use crate::kb::vectors::{VectorStore, VectorStoreConfig};
use crate::llm::{LlmEngine, GenerationParams, ModelInfo};
use crate::security::{FileKeyStore, KeyStorageMode, TOKEN_HUGGINGFACE, TOKEN_JIRA};
use crate::validation::{validate_text_size, validate_non_empty, validate_url, is_http_url, validate_ticket_id, validate_within_home, normalize_and_validate_namespace_id, MAX_QUERY_BYTES, MAX_TEXT_INPUT_BYTES, ValidationError};
use crate::AppState;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::State;
use tokio::sync::mpsc;
use once_cell::sync::Lazy;

/// Global cancel flag for generation - shared between generate and cancel commands
static GENERATION_CANCEL_FLAG: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));
static DOWNLOAD_CANCEL_FLAG: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));

/// Initialize the application
#[tauri::command]
pub async fn initialize_app(state: State<'_, AppState>) -> Result<InitResult, String> {
    // Ensure app data directory exists with secure permissions (0o700)
    let app_dir = get_app_data_dir();
    crate::security::create_secure_dir(&app_dir).map_err(|e| e.to_string())?;

    // Initialize audit logger
    let _ = AuditLogger::init();

    // Check if this is first run (no master key in any storage)
    // FileKeyStore::get_master_key() handles migration from legacy/Keychain automatically
    let is_first_run = !FileKeyStore::has_master_key();

    // Get or create master key (handles passphrase mode check internally)
    let master_key = match FileKeyStore::get_master_key() {
        Ok(key) => key,
        Err(crate::security::SecurityError::PassphraseRequired) => {
            // Passphrase mode - return special result indicating passphrase needed
            return Ok(InitResult {
                is_first_run,
                vector_enabled: false,
                vector_store_ready: false,
                key_storage_mode: KeyStorageMode::Passphrase.to_string(),
                passphrase_required: true,
            });
        }
        Err(e) => return Err(e.to_string()),
    };

    // Log app initialization
    audit::audit_app_initialized(is_first_run);

    // Open database
    let db_path = get_db_path();
    let db = Database::open(&db_path, &master_key).map_err(|e| e.to_string())?;
    db.initialize().map_err(|e| e.to_string())?;

    // Seed built-in decision trees on first run
    db.seed_builtin_trees().map_err(|e| e.to_string())?;

    // Ensure response_templates table exists
    db.ensure_templates_table().map_err(|e| e.to_string())?;

    // Check vector consent from database
    let vector_enabled = db.get_vector_consent()
        .map(|c| c.enabled)
        .unwrap_or(false);

    // Store in app state - use scope to ensure lock is dropped before async operations
    {
        let mut db_lock = state.db.lock().map_err(|e| e.to_string())?;
        *db_lock = Some(db);
    } // db_lock dropped here

    // Initialize vector store if consent given
    let vector_store_ready = if vector_enabled {
        let vectors_path = get_vectors_dir();
        let config = VectorStoreConfig {
            path: vectors_path,
            embedding_dim: 768, // nomic-embed-text default
            encryption_enabled: false,
        };

        let mut vector_store = VectorStore::new(config);
        match vector_store.init().await {
            Ok(()) => {
                // Enable with user consent (already given)
                let _ = vector_store.enable(true);
                // Create table if needed
                let _ = vector_store.create_table().await;
                *state.vectors.write().await = Some(vector_store);
                true
            }
            Err(e) => {
                eprintln!("Vector store init failed (continuing without vectors): {}", e);
                false
            }
        }
    } else {
        false
    };

    Ok(InitResult {
        is_first_run,
        vector_enabled,
        vector_store_ready,
        key_storage_mode: KeyStorageMode::Keychain.to_string(),
        passphrase_required: false,
    })
}

/// Verify FTS5 is available (release gate command)
#[tauri::command]
pub fn check_fts5_enabled(state: State<'_, AppState>) -> Result<bool, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.verify_fts5().map_err(|e| e.to_string())
}

/// Check database integrity
#[tauri::command]
pub fn check_db_integrity(state: State<'_, AppState>) -> Result<bool, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.check_integrity().map_err(|e| e.to_string())?;
    Ok(true)
}

/// Get vector search consent status
#[tauri::command]
pub fn get_vector_consent(state: State<'_, AppState>) -> Result<crate::db::VectorConsent, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.get_vector_consent().map_err(|e| e.to_string())
}

/// Set vector search consent (requires explicit opt-in if unencrypted)
#[tauri::command]
pub fn set_vector_consent(
    state: State<'_, AppState>,
    enabled: bool,
    encryption_supported: bool,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.set_vector_consent(enabled, encryption_supported).map_err(|e| e.to_string())
}

/// Check if credential storage is available
/// (Always true now that we use file-based storage)
#[tauri::command]
pub fn check_keychain_available() -> bool {
    true // File-based storage is always available
}

/// Search options for advanced search tuning
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SearchOptionsParam {
    /// Weight for FTS5 results (0.0-1.0, default 0.5)
    pub fts_weight: Option<f64>,
    /// Weight for vector results (0.0-1.0, default 0.5)
    pub vector_weight: Option<f64>,
    /// Enable deduplication (default true)
    pub enable_dedup: Option<bool>,
    /// Deduplication threshold (0.0-1.0, default 0.85)
    pub dedup_threshold: Option<f64>,
}

/// Hybrid search for KB (FTS5 + vector when enabled)
/// Uses parallel execution when vector search is available
#[tauri::command]
pub async fn search_kb(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
    namespace_id: Option<String>,
) -> Result<Vec<crate::kb::search::SearchResult>, String> {
    search_kb_with_options(state, query, limit, namespace_id, None).await
}

/// Advanced search with configurable options
#[tauri::command]
pub async fn search_kb_with_options(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
    namespace_id: Option<String>,
    options: Option<SearchOptionsParam>,
) -> Result<Vec<crate::kb::search::SearchResult>, String> {
    use crate::kb::search::{HybridSearch, SearchOptions};

    // Validate query input
    validate_non_empty(&query).map_err(|e| e.to_string())?;
    validate_text_size(&query, MAX_QUERY_BYTES).map_err(|e| e.to_string())?;

    let limit = limit.unwrap_or(10).min(100); // Cap limit at 100

    // Build search options
    let mut search_opts = SearchOptions::new(limit)
        .with_namespace(namespace_id.clone());

    if let Some(opts) = options {
        if let (Some(fts_w), Some(vec_w)) = (opts.fts_weight, opts.vector_weight) {
            search_opts = search_opts.with_weights(fts_w, vec_w);
        }
        if let Some(enable) = opts.enable_dedup {
            let threshold = opts.dedup_threshold.unwrap_or(0.85);
            search_opts = search_opts.with_dedup(enable, threshold);
        }
    }

    let ns_id = namespace_id.clone();
    let ns_id_for_vector = namespace_id.clone();

    // Get query embedding if vector search is available (sync operation)
    let query_embedding = {
        let vectors_lock = state.vectors.read().await;
        let embeddings_lock = state.embeddings.read();

        if let (Some(vectors), Some(embeddings)) = (vectors_lock.as_ref(), embeddings_lock.as_ref()) {
            if vectors.is_enabled() && embeddings.is_model_loaded() {
                embeddings.embed(&query).ok()
            } else {
                None
            }
        } else {
            None
        }
    }; // Locks released here

    // Clone state references for parallel execution
    let vectors_state = state.vectors.clone();

    // Start vector search as a spawned task (runs in parallel with FTS)
    let vector_handle = tokio::spawn(async move {
        if let Some(embedding) = query_embedding {
            let vectors_lock = vectors_state.read().await;
            if let Some(vectors) = vectors_lock.as_ref() {
                return vectors
                    .search_similar_in_namespace(&embedding, ns_id_for_vector.as_deref(), limit * 3)
                    .await
                    .ok();
            }
        }
        None
    });

    // Do FTS search (sync, fast) - runs while vector search is in progress
    let fts_results = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;
        HybridSearch::fts_search_with_namespace(db, &query, ns_id.as_deref(), limit * 3)
            .map_err(|e| e.to_string())?
    }; // DB lock released here

    // Wait for vector search to complete
    let vector_results = vector_handle.await.unwrap_or(None);

    // Re-acquire DB lock for fusion (needed for vector result enrichment)
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    // Fuse results with configurable options (weights, dedup)
    HybridSearch::fuse_results_with_options(db, fts_results, vector_results, search_opts)
        .map_err(|e| e.to_string())
}

/// Get formatted context for LLM injection from search results
#[tauri::command]
pub async fn get_search_context(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
    namespace_id: Option<String>,
) -> Result<String, String> {
    let results = search_kb(state, query, limit, namespace_id).await?;
    Ok(crate::kb::search::HybridSearch::format_context(&results))
}

/// Greet command (placeholder for testing)
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Initialization result
#[derive(serde::Serialize)]
pub struct InitResult {
    pub is_first_run: bool,
    pub vector_enabled: bool,
    pub vector_store_ready: bool,
    pub key_storage_mode: String,
    pub passphrase_required: bool,
}

// ============================================================================
// LLM Commands
// ============================================================================

/// Initialize the LLM engine
#[tauri::command]
pub fn init_llm_engine(state: State<'_, AppState>) -> Result<(), String> {
    let engine = LlmEngine::new().map_err(|e| e.to_string())?;
    *state.llm.write() = Some(engine);
    Ok(())
}

/// Load a model by ID
#[tauri::command]
pub fn load_model(
    state: State<'_, AppState>,
    model_id: String,
    n_gpu_layers: Option<u32>,
) -> Result<ModelInfo, String> {
    // Get filename from model ID
    let (_, filename) = get_model_source(&model_id)?;

    // Build path to model file
    let models_dir = crate::db::get_models_dir();
    let path = models_dir.join(filename);

    if !path.exists() {
        return Err(format!("Model file not found: {}. Please download the model first.", filename));
    }

    let llm_guard = state.llm.read();
    let engine = llm_guard.as_ref().ok_or("LLM engine not initialized")?;

    let layers = n_gpu_layers.unwrap_or(1000); // Default to full GPU offload

    engine.load_model(&path, layers, model_id).map_err(|e| e.to_string())
}

/// Load a custom GGUF model from a file path
#[tauri::command]
pub fn load_custom_model(
    state: State<'_, AppState>,
    model_path: String,
    n_gpu_layers: Option<u32>,
) -> Result<ModelInfo, String> {
    use std::path::Path;

    let path = Path::new(&model_path);

    // Validate path exists
    if !path.exists() {
        return Err(format!("Model file not found: {}", model_path));
    }

    let validated_path = validate_within_home(path).map_err(|e| match e {
        ValidationError::PathTraversal => "Model file must be within your home directory".to_string(),
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This path is blocked because it contains sensitive data".to_string()
        }
        _ => format!("Invalid model path: {}", e),
    })?;

    if !validated_path.is_file() {
        return Err("Model path is not a file".into());
    }

    // Validate GGUF extension
    let ext = validated_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext.to_lowercase() != "gguf" {
        return Err("Invalid file type. Only .gguf files are supported.".into());
    }

    // Validate file size (at least 1MB, sanity check)
    let metadata = std::fs::metadata(&validated_path).map_err(|e| e.to_string())?;
    if metadata.len() < 1_000_000 {
        return Err("File too small to be a valid GGUF model.".into());
    }

    let model_id = validated_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("custom-model")
        .to_string();

    let llm_guard = state.llm.read();
    let engine = llm_guard.as_ref().ok_or("LLM engine not initialized")?;

    let layers = n_gpu_layers.unwrap_or(1000); // Default to full GPU offload

    engine.load_model(&validated_path, layers, model_id).map_err(|e| e.to_string())
}

/// Validate a GGUF file without loading it (returns model metadata)
#[tauri::command]
pub fn validate_gguf_file(model_path: String) -> Result<GgufFileInfo, String> {
    use std::path::Path;
    use std::fs;

    let path = Path::new(&model_path);

    if !path.exists() {
        return Err(format!("File not found: {}", model_path));
    }

    let validated_path = validate_within_home(path).map_err(|e| match e {
        ValidationError::PathTraversal => "Model file must be within your home directory".to_string(),
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This path is blocked because it contains sensitive data".to_string()
        }
        _ => format!("Invalid model path: {}", e),
    })?;

    if !validated_path.is_file() {
        return Err("Model path is not a file".into());
    }

    let ext = validated_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext.to_lowercase() != "gguf" {
        return Err("Invalid file type. Only .gguf files are supported.".into());
    }

    let metadata = fs::metadata(&validated_path).map_err(|e| e.to_string())?;
    let filename = validated_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Check GGUF magic bytes (optional, basic validation)
    let mut file = fs::File::open(&validated_path).map_err(|e| e.to_string())?;
    let mut magic = [0u8; 4];
    use std::io::Read;
    file.read_exact(&mut magic).map_err(|e| e.to_string())?;

    // GGUF magic is "GGUF" in ASCII
    if &magic != b"GGUF" {
        return Err("Invalid GGUF file: magic bytes mismatch".into());
    }

    Ok(GgufFileInfo {
        path: validated_path.to_string_lossy().to_string(),
        filename,
        size_bytes: metadata.len(),
        is_valid: true,
    })
}

/// Information about a GGUF file
#[derive(Debug, Clone, serde::Serialize)]
pub struct GgufFileInfo {
    pub path: String,
    #[serde(rename = "file_name")]
    pub filename: String,
    #[serde(rename = "file_size")]
    pub size_bytes: u64,
    pub is_valid: bool,
}

/// Unload the current model
#[tauri::command]
pub fn unload_model(state: State<'_, AppState>) -> Result<(), String> {
    let llm_guard = state.llm.read();
    let engine = llm_guard.as_ref().ok_or("LLM engine not initialized")?;
    engine.unload_model();
    Ok(())
}

/// Get current model info
#[tauri::command]
pub fn get_model_info(state: State<'_, AppState>) -> Result<Option<ModelInfo>, String> {
    let llm_guard = state.llm.read();
    let engine = llm_guard.as_ref().ok_or("LLM engine not initialized")?;
    Ok(engine.model_info())
}

/// Check if a model is loaded
#[tauri::command]
pub fn is_model_loaded(state: State<'_, AppState>) -> Result<bool, String> {
    let llm_guard = state.llm.read();
    match llm_guard.as_ref() {
        Some(engine) => Ok(engine.is_model_loaded()),
        None => Ok(false),
    }
}

/// Generation parameters for the API
#[derive(serde::Deserialize)]
pub struct GenerateParams {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
    pub repeat_penalty: Option<f32>,
    pub context_window: Option<u32>,
}

impl From<GenerateParams> for GenerationParams {
    fn from(p: GenerateParams) -> Self {
        let mut params = GenerationParams::default();
        if let Some(v) = p.max_tokens { params.max_tokens = v; }
        if let Some(v) = p.temperature { params.temperature = v; }
        if let Some(v) = p.top_p { params.top_p = v; }
        if let Some(v) = p.top_k { params.top_k = v; }
        if let Some(v) = p.repeat_penalty { params.repeat_penalty = v; }
        params.context_window = p.context_window;
        params
    }
}

/// Generation result
#[derive(serde::Serialize)]
pub struct GenerationResult {
    pub text: String,
    pub tokens_generated: u32,
    pub duration_ms: u64,
}

/// Generate text (non-streaming, for simple use cases)
#[tauri::command]
pub async fn generate_text(
    state: State<'_, AppState>,
    prompt: String,
    params: Option<GenerateParams>,
) -> Result<GenerationResult, String> {
    // Validate prompt input
    validate_non_empty(&prompt).map_err(|e| e.to_string())?;
    validate_text_size(&prompt, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;

    // Clone the Arc to use in async context
    let llm = state.llm.clone();

    // Get engine state without holding lock across await
    let engine_state = {
        let llm_guard = llm.read();
        let engine = llm_guard.as_ref().ok_or("LLM engine not initialized")?;
        if !engine.is_model_loaded() {
            return Err("No model loaded".into());
        }
        engine.state.clone()
    }; // Lock released here

    let gen_params = params.map(|p| p.into()).unwrap_or_default();

    let (tx, mut rx) = mpsc::channel(100);
    let cancel_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Start generation in background
    let prompt_clone = prompt.clone();
    let cancel_clone = cancel_flag.clone();
    let tx_clone = tx.clone();

    tokio::spawn(async move {
        let temp_engine = LlmEngine { state: engine_state };
        let _ = temp_engine.generate_streaming(&prompt_clone, gen_params, tx_clone, cancel_clone).await;
    });

    // Collect output
    let mut text = String::new();
    let mut tokens_generated = 0u32;
    let mut duration_ms = 0u64;

    while let Some(event) = rx.recv().await {
        match event {
            crate::llm::GenerationEvent::Token(t) => text.push_str(&t),
            crate::llm::GenerationEvent::Done { tokens_generated: t, duration_ms: d } => {
                tokens_generated = t;
                duration_ms = d;
                break;
            }
            crate::llm::GenerationEvent::Error(e) => return Err(e),
        }
    }

    Ok(GenerationResult {
        text,
        tokens_generated,
        duration_ms,
    })
}

/// Parameters for context-aware generation
#[derive(serde::Deserialize)]
pub struct GenerateWithContextParams {
    /// User's input text (ticket content, question)
    pub user_input: String,
    /// Optional KB search query (defaults to user_input if not provided)
    pub kb_query: Option<String>,
    /// Number of KB results to include (default: 3)
    pub kb_limit: Option<usize>,
    /// OCR text from screenshots
    pub ocr_text: Option<String>,
    /// Diagnostic notes
    pub diagnostic_notes: Option<String>,
    /// Decision tree results
    pub tree_decisions: Option<crate::prompts::TreeDecisions>,
    /// Jira ticket for context
    pub jira_ticket: Option<crate::jira::JiraTicket>,
    /// Response length preference
    pub response_length: Option<crate::prompts::ResponseLength>,
    /// Generation parameters
    pub gen_params: Option<GenerateParams>,
}

/// Quality metrics for generation
#[derive(serde::Serialize)]
pub struct GenerationMetrics {
    /// Tokens generated per second
    pub tokens_per_second: f64,
    /// Number of KB sources used in context
    pub sources_used: u32,
    /// Approximate word count of response
    pub word_count: u32,
    /// Response length vs target (Short/Medium/Long)
    pub length_target_met: bool,
    /// Percentage of context window used
    pub context_utilization: f64,
}

/// Result of context-aware generation
#[derive(serde::Serialize)]
pub struct GenerateWithContextResult {
    /// Generated response text
    pub text: String,
    /// Number of tokens generated
    pub tokens_generated: u32,
    /// Generation duration in milliseconds
    pub duration_ms: u64,
    /// KB chunk IDs used as sources
    pub source_chunk_ids: Vec<String>,
    /// KB search results used for context
    pub sources: Vec<ContextSource>,
    /// Quality metrics for the generation
    pub metrics: GenerationMetrics,
    /// Prompt template version used for this generation (for A/B tracking)
    pub prompt_template_version: String,
}

/// Source information for context
#[derive(serde::Serialize)]
pub struct ContextSource {
    pub chunk_id: String,
    pub document_id: String,
    pub file_path: String,
    pub title: Option<String>,
    pub heading_path: Option<String>,
    pub score: f64,
}

/// Generate text with KB context injection
#[tauri::command]
pub async fn generate_with_context(
    state: State<'_, AppState>,
    params: GenerateWithContextParams,
) -> Result<GenerateWithContextResult, String> {
    use crate::prompts::PromptBuilder;

    validate_non_empty(&params.user_input).map_err(|e| e.to_string())?;
    validate_text_size(&params.user_input, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;
    if let Some(query) = &params.kb_query {
        validate_text_size(query, MAX_QUERY_BYTES).map_err(|e| e.to_string())?;
    }
    if let Some(ocr) = &params.ocr_text {
        validate_text_size(ocr, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;
    }
    if let Some(notes) = &params.diagnostic_notes {
        validate_text_size(notes, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;
    }

    // Search KB if database is available
    let kb_results = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        if let Some(db) = db_lock.as_ref() {
            let query = params.kb_query.as_ref().unwrap_or(&params.user_input);
            let limit = params.kb_limit.unwrap_or(3);
            crate::kb::search::HybridSearch::search(db, query, limit)
                .unwrap_or_default()
        } else {
            vec![]
        }
    };

    // Build sources info for response
    let sources: Vec<ContextSource> = kb_results
        .iter()
        .map(|r| ContextSource {
            chunk_id: r.chunk_id.clone(),
            document_id: r.document_id.clone(),
            file_path: r.file_path.clone(),
            title: r.title.clone(),
            heading_path: r.heading_path.clone(),
            score: r.score,
        })
        .collect();

    // Build prompt with context
    let mut builder = PromptBuilder::new()
        .with_kb_results(kb_results)
        .with_user_input(&params.user_input);

    if let Some(ocr) = &params.ocr_text {
        builder = builder.with_ocr_text(ocr);
    }

    if let Some(notes) = &params.diagnostic_notes {
        builder = builder.with_diagnostic_notes(notes);
    }

    if let Some(tree) = params.tree_decisions {
        builder = builder.with_tree_decisions(tree);
    }

    if let Some(ticket) = params.jira_ticket {
        builder = builder.with_jira_ticket(ticket);
    }

    if let Some(length) = params.response_length {
        builder = builder.with_response_length(length);
    }

    let source_chunk_ids = builder.get_source_chunk_ids();
    let response_length = params.response_length.unwrap_or_default();
    let prompt = builder.build();
    let prompt_length = prompt.len();

    validate_text_size(&prompt, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;

    // Generate using the built prompt
    let gen_result = generate_text(
        state,
        prompt,
        params.gen_params,
    ).await?;

    // Calculate quality metrics
    let word_count = gen_result.text.split_whitespace().count() as u32;
    let target_words = response_length.target_words() as u32;
    let length_target_met = match response_length {
        crate::prompts::ResponseLength::Short => word_count <= target_words + 40,
        crate::prompts::ResponseLength::Medium => word_count >= target_words / 2 && word_count <= target_words * 2,
        crate::prompts::ResponseLength::Long => word_count >= target_words / 2,
    };

    let tokens_per_second = if gen_result.duration_ms > 0 {
        (gen_result.tokens_generated as f64 * 1000.0) / gen_result.duration_ms as f64
    } else {
        0.0
    };

    // Estimate context utilization (prompt tokens / typical 4096 context window)
    let estimated_prompt_tokens = prompt_length / 4;
    let context_window = 4096; // Default, could be read from model
    let context_utilization = (estimated_prompt_tokens as f64 / context_window as f64 * 100.0).min(100.0);

    let metrics = GenerationMetrics {
        tokens_per_second,
        sources_used: sources.len() as u32,
        word_count,
        length_target_met,
        context_utilization,
    };

    Ok(GenerateWithContextResult {
        text: gen_result.text,
        tokens_generated: gen_result.tokens_generated,
        duration_ms: gen_result.duration_ms,
        source_chunk_ids,
        sources,
        metrics,
        prompt_template_version: crate::prompts::PROMPT_TEMPLATE_VERSION.to_string(),
    })
}

/// Streaming token event
#[derive(Clone, serde::Serialize)]
pub struct StreamToken {
    pub token: String,
    pub done: bool,
}

/// Generate text with streaming (emits events as tokens are generated)
#[tauri::command]
pub async fn generate_streaming(
    window: tauri::Window,
    state: State<'_, AppState>,
    params: GenerateWithContextParams,
) -> Result<GenerateWithContextResult, String> {
    use crate::prompts::PromptBuilder;

    validate_non_empty(&params.user_input).map_err(|e| e.to_string())?;
    validate_text_size(&params.user_input, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;
    if let Some(query) = &params.kb_query {
        validate_text_size(query, MAX_QUERY_BYTES).map_err(|e| e.to_string())?;
    }
    if let Some(ocr) = &params.ocr_text {
        validate_text_size(ocr, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;
    }
    if let Some(notes) = &params.diagnostic_notes {
        validate_text_size(notes, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;
    }

    // Search KB if database is available
    let kb_results = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        if let Some(db) = db_lock.as_ref() {
            let query = params.kb_query.as_ref().unwrap_or(&params.user_input);
            let limit = params.kb_limit.unwrap_or(3);
            crate::kb::search::HybridSearch::search(db, query, limit)
                .unwrap_or_default()
        } else {
            vec![]
        }
    };

    // Build sources info for response
    let sources: Vec<ContextSource> = kb_results
        .iter()
        .map(|r| ContextSource {
            chunk_id: r.chunk_id.clone(),
            document_id: r.document_id.clone(),
            file_path: r.file_path.clone(),
            title: r.title.clone(),
            heading_path: r.heading_path.clone(),
            score: r.score,
        })
        .collect();

    // Build prompt with context
    let mut builder = PromptBuilder::new()
        .with_kb_results(kb_results)
        .with_user_input(&params.user_input);

    if let Some(ocr) = &params.ocr_text {
        builder = builder.with_ocr_text(ocr);
    }

    if let Some(notes) = &params.diagnostic_notes {
        builder = builder.with_diagnostic_notes(notes);
    }

    if let Some(tree) = params.tree_decisions {
        builder = builder.with_tree_decisions(tree);
    }

    if let Some(ticket) = params.jira_ticket {
        builder = builder.with_jira_ticket(ticket);
    }

    if let Some(length) = params.response_length {
        builder = builder.with_response_length(length);
    }

    let source_chunk_ids = builder.get_source_chunk_ids();
    let response_length = params.response_length.unwrap_or_default();
    let prompt = builder.build();
    let prompt_length = prompt.len();

    validate_text_size(&prompt, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;

    // Get engine state
    let llm = state.llm.clone();
    let engine_state = {
        let llm_guard = llm.read();
        let engine = llm_guard.as_ref().ok_or("LLM engine not initialized")?;
        if !engine.is_model_loaded() {
            return Err("No model loaded".into());
        }
        engine.state.clone()
    };

    let gen_params = params.gen_params.map(|p| p.into()).unwrap_or_default();

    let (tx, mut rx) = mpsc::channel(100);

    // Reset the global cancel flag before starting
    GENERATION_CANCEL_FLAG.store(false, Ordering::SeqCst);
    let cancel_flag = GENERATION_CANCEL_FLAG.clone();

    // Start generation in background
    let prompt_clone = prompt.clone();
    let cancel_clone = cancel_flag.clone();
    let tx_clone = tx.clone();

    tokio::spawn(async move {
        let temp_engine = LlmEngine { state: engine_state };
        let _ = temp_engine.generate_streaming(&prompt_clone, gen_params, tx_clone, cancel_clone).await;
    });

    // Forward tokens to frontend as events and collect output
    let mut text = String::new();
    let mut tokens_generated = 0u32;
    let mut duration_ms = 0u64;

    while let Some(event) = rx.recv().await {
        match event {
            crate::llm::GenerationEvent::Token(t) => {
                // Emit token to frontend
                let _ = window.emit("llm-token", StreamToken {
                    token: t.clone(),
                    done: false,
                });
                text.push_str(&t);
            }
            crate::llm::GenerationEvent::Done { tokens_generated: t, duration_ms: d } => {
                tokens_generated = t;
                duration_ms = d;
                // Emit done signal
                let _ = window.emit("llm-token", StreamToken {
                    token: String::new(),
                    done: true,
                });
                break;
            }
            crate::llm::GenerationEvent::Error(e) => {
                let _ = window.emit("llm-token", StreamToken {
                    token: String::new(),
                    done: true,
                });
                return Err(e);
            }
        }
    }

    // Calculate quality metrics
    let word_count = text.split_whitespace().count() as u32;
    let target_words = response_length.target_words() as u32;
    let length_target_met = match response_length {
        crate::prompts::ResponseLength::Short => word_count <= target_words + 40,
        crate::prompts::ResponseLength::Medium => word_count >= target_words / 2 && word_count <= target_words * 2,
        crate::prompts::ResponseLength::Long => word_count >= target_words / 2,
    };

    let tokens_per_second = if duration_ms > 0 {
        (tokens_generated as f64 * 1000.0) / duration_ms as f64
    } else {
        0.0
    };

    // Estimate context utilization (prompt tokens / typical 4096 context window)
    let estimated_prompt_tokens = prompt_length / 4;
    let context_window = 4096; // Default, could be read from model
    let context_utilization = (estimated_prompt_tokens as f64 / context_window as f64 * 100.0).min(100.0);

    let metrics = GenerationMetrics {
        tokens_per_second,
        sources_used: sources.len() as u32,
        word_count,
        length_target_met,
        context_utilization,
    };

    Ok(GenerateWithContextResult {
        text,
        tokens_generated,
        duration_ms,
        source_chunk_ids,
        sources,
        metrics,
        prompt_template_version: crate::prompts::PROMPT_TEMPLATE_VERSION.to_string(),
    })
}

/// Test model with a simple prompt
#[tauri::command]
pub async fn test_model(state: State<'_, AppState>) -> Result<TestModelResult, String> {
    let result = generate_text(
        state,
        "Say hello in one sentence.".to_string(),
        Some(GenerateParams {
            max_tokens: Some(50),
            temperature: Some(0.7),
            top_p: None,
            top_k: None,
            repeat_penalty: None,
            context_window: None,
        }),
    ).await?;

    let tokens_per_sec = if result.duration_ms > 0 {
        (result.tokens_generated as f64 / result.duration_ms as f64) * 1000.0
    } else {
        0.0
    };

    Ok(TestModelResult {
        output: result.text,
        tokens_generated: result.tokens_generated,
        duration_ms: result.duration_ms,
        tokens_per_sec,
    })
}

/// Cancel an ongoing text generation
#[tauri::command]
pub fn cancel_generation() -> Result<(), String> {
    GENERATION_CANCEL_FLAG.store(true, Ordering::SeqCst);
    Ok(())
}

/// Test model result
#[derive(serde::Serialize)]
pub struct TestModelResult {
    pub output: String,
    pub tokens_generated: u32,
    pub duration_ms: u64,
    pub tokens_per_sec: f64,
}

const CONTEXT_WINDOW_SETTING: &str = "llm_context_window";

/// Get the configured context window size
#[tauri::command]
pub fn get_context_window(state: State<'_, AppState>) -> Result<Option<u32>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let result: Result<String, _> = db.conn().query_row(
        "SELECT value FROM settings WHERE key = ?",
        rusqlite::params![CONTEXT_WINDOW_SETTING],
        |row| row.get(0),
    );

    match result {
        Ok(value) => {
            let parsed = value.parse::<u32>().ok();
            Ok(parsed)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Set the context window size (2048-32768, or None for model default)
#[tauri::command]
pub fn set_context_window(
    state: State<'_, AppState>,
    size: Option<u32>,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    match size {
        Some(s) => {
            // Validate range
            if !(2048..=32768).contains(&s) {
                return Err("Context window must be between 2048 and 32768".to_string());
            }
            db.conn().execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
                rusqlite::params![CONTEXT_WINDOW_SETTING, s.to_string()],
            ).map_err(|e| e.to_string())?;
        }
        None => {
            // Remove setting to use model default
            db.conn().execute(
                "DELETE FROM settings WHERE key = ?",
                rusqlite::params![CONTEXT_WINDOW_SETTING],
            ).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

// ============================================================================
// Download Commands
// ============================================================================

use crate::downloads::{DownloadManager, ModelSource, recommended_models};

/// Get recommended models list
#[tauri::command]
pub fn get_recommended_models() -> Vec<ModelSource> {
    recommended_models()
}

/// List downloaded models
#[tauri::command]
pub fn list_downloaded_models() -> Result<Vec<String>, String> {
    let app_dir = get_app_data_dir();
    let manager = DownloadManager::new(&app_dir);

    let models = manager.list_models().map_err(|e| e.to_string())?;

    // Map filenames back to model IDs
    let model_ids: Vec<String> = models.into_iter()
        .filter_map(|p| {
            let filename = p.file_name()?.to_str()?;
            // Reverse lookup: filename -> model_id
            match filename {
                "Llama-3.2-1B-Instruct-Q4_K_M.gguf" => Some("llama-3.2-1b-instruct".to_string()),
                "Llama-3.2-3B-Instruct-Q4_K_M.gguf" => Some("llama-3.2-3b-instruct".to_string()),
                "Phi-3.1-mini-4k-instruct-Q4_K_M.gguf" => Some("phi-3-mini-4k-instruct".to_string()),
                _ => None, // Unknown model files are ignored
            }
        })
        .collect();

    Ok(model_ids)
}

/// Check if embedding model is downloaded and return its path if so
#[tauri::command]
pub fn get_embedding_model_path(model_id: String) -> Result<Option<String>, String> {
    let filename = get_embedding_model_filename(&model_id)
        .ok_or_else(|| format!("Unknown embedding model ID: {}", model_id))?;

    let app_dir = get_app_data_dir();
    let model_path = app_dir.join("models").join(filename);

    if model_path.exists() {
        Ok(Some(model_path.to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
}

/// Check if the default embedding model is downloaded
#[tauri::command]
pub fn is_embedding_model_downloaded() -> Result<bool, String> {
    let app_dir = get_app_data_dir();
    let model_path = app_dir.join("models").join("nomic-embed-text-v1.5.Q5_K_M.gguf");
    Ok(model_path.exists())
}

/// Get models directory path
#[tauri::command]
pub fn get_models_dir() -> Result<String, String> {
    let app_dir = get_app_data_dir();
    let manager = DownloadManager::new(&app_dir);
    Ok(manager.models_dir().display().to_string())
}

/// Delete a downloaded model
#[tauri::command]
pub fn delete_downloaded_model(filename: String) -> Result<(), String> {
    use std::path::Component;
    use std::path::Path;

    let path = Path::new(&filename);
    let mut components = path.components();
    let is_single_filename = matches!(components.next(), Some(Component::Normal(_)))
        && components.next().is_none();
    if path.is_absolute() || !is_single_filename {
        return Err("Invalid model filename".into());
    }

    let has_gguf_ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("gguf"))
        .unwrap_or(false);
    if !has_gguf_ext {
        return Err("Only .gguf model files can be deleted".into());
    }

    let app_dir = get_app_data_dir();
    let manager = DownloadManager::new(&app_dir);
    manager.delete_model(&filename).map_err(|e| e.to_string())
}

/// Get HuggingFace token status (not the actual token for security)
#[tauri::command]
pub fn has_hf_token() -> Result<bool, String> {
    FileKeyStore::get_token(TOKEN_HUGGINGFACE)
        .map(|t| t.is_some())
        .map_err(|e| e.to_string())
}

/// Store HuggingFace token
#[tauri::command]
pub fn set_hf_token(token: String) -> Result<(), String> {
    FileKeyStore::store_token(TOKEN_HUGGINGFACE, &token).map_err(|e| e.to_string())?;
    audit::audit_token_set("huggingface");
    Ok(())
}

/// Delete HuggingFace token
#[tauri::command]
pub fn clear_hf_token() -> Result<(), String> {
    FileKeyStore::delete_token(TOKEN_HUGGINGFACE).map_err(|e| e.to_string())?;
    audit::audit_token_cleared("huggingface");
    Ok(())
}

use tauri::Emitter;

/// Map model ID to HuggingFace repo and filename
fn get_model_source(model_id: &str) -> Result<(&'static str, &'static str), String> {
    match model_id {
        "llama-3.2-1b-instruct" => Ok(("bartowski/Llama-3.2-1B-Instruct-GGUF", "Llama-3.2-1B-Instruct-Q4_K_M.gguf")),
        "llama-3.2-3b-instruct" => Ok(("bartowski/Llama-3.2-3B-Instruct-GGUF", "Llama-3.2-3B-Instruct-Q4_K_M.gguf")),
        "phi-3-mini-4k-instruct" => Ok(("bartowski/Phi-3.1-mini-4k-instruct-GGUF", "Phi-3.1-mini-4k-instruct-Q4_K_M.gguf")),
        "nomic-embed-text" => Ok(("nomic-ai/nomic-embed-text-v1.5-GGUF", "nomic-embed-text-v1.5.Q5_K_M.gguf")),
        _ => Err(format!("Unknown model ID: {}", model_id)),
    }
}

/// Get the filename for an embedding model ID
fn get_embedding_model_filename(model_id: &str) -> Option<&'static str> {
    match model_id {
        "nomic-embed-text" => Some("nomic-embed-text-v1.5.Q5_K_M.gguf"),
        _ => None,
    }
}

/// Download a model from HuggingFace with progress events
#[tauri::command]
pub async fn download_model(
    window: tauri::Window,
    model_id: String,
) -> Result<String, String> {
    let (repo, filename) = get_model_source(&model_id)?;

    let app_dir = get_app_data_dir();
    let manager = DownloadManager::new(&app_dir);
    manager.init().map_err(|e| e.to_string())?;

    // Fetch file info (size and SHA256) from HuggingFace API for verification
    let mut source = ModelSource::huggingface(repo, filename);
    match crate::downloads::fetch_hf_file_info(repo, filename).await {
        Ok((size, sha256)) => {
            source.size_bytes = Some(size);
            source.sha256 = Some(sha256);
        }
        Err(e) => {
            // Log warning but continue without checksum verification
            eprintln!("Warning: Could not fetch file info for checksum verification: {}", e);
        }
    }

    // Create progress channel
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    DOWNLOAD_CANCEL_FLAG.store(false, Ordering::SeqCst);
    let cancel_flag = DOWNLOAD_CANCEL_FLAG.clone();

    // Spawn download task
    let download_handle = {
        let cancel = cancel_flag.clone();
        tokio::spawn(async move {
            manager.download(&source, tx, cancel).await
        })
    };

    // Forward progress events to frontend
    let window_clone = window.clone();
    let event_handle = tokio::spawn(async move {
        while let Some(progress) = rx.recv().await {
            let _ = window_clone.emit("download-progress", &progress);
        }
    });

    // Wait for download to complete
    let result = download_handle.await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

    // Wait for event forwarding to finish
    let _ = event_handle.await;

    Ok(result.display().to_string())
}

/// Cancel an ongoing download
#[tauri::command]
pub fn cancel_download() -> Result<(), String> {
    DOWNLOAD_CANCEL_FLAG.store(true, Ordering::SeqCst);
    Ok(())
}

// ============================================================================
// KB Indexer Commands
// ============================================================================

use crate::kb::indexer::{KbIndexer, IndexResult, IndexStats};

/// KB folder setting key
const KB_FOLDER_SETTING: &str = "kb_folder";

/// Set the KB folder path
/// Path must be within user's home directory (auto-creates if needed)
/// Blocks sensitive directories like .ssh, .aws, .gnupg, .config
#[tauri::command]
pub fn set_kb_folder(state: State<'_, AppState>, folder_path: String) -> Result<(), String> {
    let path = std::path::Path::new(&folder_path);

    // Validate path is within home directory (auto-creates if needed)
    let validated_path = validate_within_home(path).map_err(|e| match e {
        ValidationError::PathTraversal => {
            "KB folder must be within your home directory".to_string()
        }
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This directory cannot be used as it contains sensitive data".to_string()
        }
        _ => format!("Invalid KB folder: {}", e),
    })?;

    // Verify it's a directory
    if !validated_path.is_dir() {
        return Err("Path is not a directory".into());
    }

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    // Store in settings
    db.conn()
        .execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            rusqlite::params![KB_FOLDER_SETTING, validated_path.to_string_lossy().as_ref()],
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get the current KB folder path
#[tauri::command]
pub fn get_kb_folder(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let result: Result<String, _> = db.conn().query_row(
        "SELECT value FROM settings WHERE key = ?",
        rusqlite::params![KB_FOLDER_SETTING],
        |row| row.get(0),
    );

    match result {
        Ok(path) => Ok(Some(path)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Index the KB folder with progress events
#[tauri::command]
pub async fn index_kb(
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<IndexResult, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    // Get KB folder
    let folder_path: String = db.conn()
        .query_row(
            "SELECT value FROM settings WHERE key = ?",
            rusqlite::params![KB_FOLDER_SETTING],
            |row| row.get(0),
        )
        .map_err(|_| "KB folder not configured")?;

    let path = std::path::Path::new(&folder_path);
    if !path.exists() {
        return Err("KB folder does not exist".into());
    }

    // Run indexing with progress events
    let indexer = KbIndexer::new();
    let result = indexer.index_folder(db, path, |progress| {
        // Emit progress event to frontend
        let _ = window.emit("kb:indexing:progress", &progress);
    }).map_err(|e| e.to_string())?;

    Ok(result)
}

/// Get KB statistics
#[tauri::command]
pub fn get_kb_stats(state: State<'_, AppState>) -> Result<IndexStats, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let indexer = KbIndexer::new();
    indexer.get_stats(db).map_err(|e| e.to_string())
}

/// List indexed KB documents, optionally filtered by namespace and/or source
#[tauri::command]
pub fn list_kb_documents(
    state: State<'_, AppState>,
    namespace_id: Option<String>,
    source_id: Option<String>,
) -> Result<Vec<KbDocumentInfo>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let docs = db.list_kb_documents(namespace_id.as_deref(), source_id.as_deref())
        .map_err(|e| e.to_string())?;

    Ok(docs.into_iter().map(|d| KbDocumentInfo {
        id: d.id,
        file_path: d.file_path,
        title: d.title,
        indexed_at: d.indexed_at,
        chunk_count: d.chunk_count.map(|c| c as i64),
        namespace_id: d.namespace_id,
        source_type: d.source_type,
        source_id: d.source_id,
    }).collect())
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
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let indexer = KbIndexer::new();
    indexer.remove_document(db, &file_path).map_err(|e| e.to_string())
}

// ============================================================================
// KB Watcher Commands
// ============================================================================

use crate::kb::watcher::KbWatcher;
use std::sync::Mutex as StdMutex;

/// Global watcher instance
static KB_WATCHER: Lazy<StdMutex<Option<KbWatcher>>> = Lazy::new(|| StdMutex::new(None));

/// Start watching KB folder for changes
#[tauri::command]
pub async fn start_kb_watcher(
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    // Get KB folder path
    let folder_path = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;
        db.conn()
            .query_row(
                "SELECT value FROM settings WHERE key = ?",
                rusqlite::params![KB_FOLDER_SETTING],
                |row| row.get::<_, String>(0),
            )
            .map_err(|_| "KB folder not configured")?
    };

    let path = std::path::Path::new(&folder_path);

    // Create and start watcher
    let mut watcher = KbWatcher::new(path).map_err(|e| e.to_string())?;
    let mut rx = watcher.start().map_err(|e| e.to_string())?;

    // Store watcher instance
    {
        let mut guard = KB_WATCHER.lock().map_err(|e| e.to_string())?;
        *guard = Some(watcher);
    }

    // Spawn event handler
    let window_clone = window.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            // Emit event to frontend
            let _ = window_clone.emit("kb:file:changed", &event);
        }
    });

    Ok(true)
}

/// Stop watching KB folder
#[tauri::command]
pub fn stop_kb_watcher() -> Result<bool, String> {
    let mut guard = KB_WATCHER.lock().map_err(|e| e.to_string())?;
    if let Some(mut watcher) = guard.take() {
        watcher.stop();
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Check if KB watcher is running
#[tauri::command]
pub fn is_kb_watcher_running() -> Result<bool, String> {
    let guard = KB_WATCHER.lock().map_err(|e| e.to_string())?;
    Ok(guard.as_ref().map(|w| w.is_running()).unwrap_or(false))
}

/// Generate embeddings for all KB chunks
/// This should be called after indexing if vector search is enabled
#[tauri::command]
pub async fn generate_kb_embeddings(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<EmbeddingGenerationResult, String> {
    // Check if vector search is enabled and embedding model is loaded
    {
        let vectors_lock = state.vectors.read().await;
        let embeddings_lock = state.embeddings.read();

        let vectors = vectors_lock.as_ref().ok_or("Vector store not initialized")?;
        if !vectors.is_enabled() {
            return Err("Vector search is disabled".into());
        }

        let embeddings = embeddings_lock.as_ref().ok_or("Embedding engine not initialized")?;
        if !embeddings.is_model_loaded() {
            return Err("Embedding model not loaded".into());
        }
    }

    // Get all chunks from database
    let chunks: Vec<(String, String)> = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;
        db.get_all_chunks_for_embedding().map_err(|e| e.to_string())?
    };

    if chunks.is_empty() {
        return Ok(EmbeddingGenerationResult {
            chunks_processed: 0,
            vectors_created: 0,
        });
    }

    let total_chunks = chunks.len();
    let batch_size = 32; // Process in batches for efficiency
    let mut vectors_created = 0;

    // Emit start event
    let _ = app_handle.emit("kb:embeddings:start", serde_json::json!({
        "total_chunks": total_chunks
    }));

    // Process chunks in batches
    for (batch_idx, batch) in chunks.chunks(batch_size).enumerate() {
        let chunk_ids: Vec<String> = batch.iter().map(|(id, _)| id.clone()).collect();
        let chunk_texts: Vec<String> = batch.iter().map(|(_, text)| text.clone()).collect();

        // Generate embeddings (sync operation)
        let embeddings: Vec<Vec<f32>> = {
            let embeddings_lock = state.embeddings.read();
            let engine = embeddings_lock.as_ref().ok_or("Embedding engine not available")?;
            engine.embed_batch(&chunk_texts).map_err(|e| e.to_string())?
        };

        // Store in vector store (async operation)
        {
            let vectors_lock = state.vectors.read().await;
            let vectors = vectors_lock.as_ref().ok_or("Vector store not available")?;
            vectors.insert_embeddings(&chunk_ids, &embeddings).await.map_err(|e| e.to_string())?;
        }

        vectors_created += embeddings.len();

        // Emit progress event
        let progress = ((batch_idx + 1) * batch_size).min(total_chunks);
        let _ = app_handle.emit("kb:embeddings:progress", serde_json::json!({
            "processed": progress,
            "total": total_chunks,
            "percentage": (progress * 100) / total_chunks
        }));
    }

    // Emit complete event
    let _ = app_handle.emit("kb:embeddings:complete", serde_json::json!({
        "vectors_created": vectors_created
    }));

    Ok(EmbeddingGenerationResult {
        chunks_processed: total_chunks,
        vectors_created,
    })
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

use crate::kb::embeddings::{EmbeddingEngine, EmbeddingModelInfo};
use crate::kb::ocr::OcrManager;

/// Initialize the embedding engine
#[tauri::command]
pub fn init_embedding_engine(state: State<'_, AppState>) -> Result<(), String> {
    let engine = EmbeddingEngine::new().map_err(|e| e.to_string())?;
    *state.embeddings.write() = Some(engine);
    Ok(())
}

/// Load an embedding model from file
#[tauri::command]
pub fn load_embedding_model(
    state: State<'_, AppState>,
    path: String,
    n_gpu_layers: Option<u32>,
) -> Result<EmbeddingModelInfo, String> {
    use std::path::Path;

    let emb_guard = state.embeddings.read();
    let engine = emb_guard.as_ref().ok_or("Embedding engine not initialized")?;

    let path = Path::new(&path);
    if !path.exists() {
        return Err(format!("Embedding model file not found: {}", path.display()));
    }

    let validated_path = validate_within_home(path).map_err(|e| match e {
        ValidationError::PathTraversal => "Embedding model file must be within your home directory".to_string(),
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This path is blocked because it contains sensitive data".to_string()
        }
        _ => format!("Invalid embedding model path: {}", e),
    })?;

    if !validated_path.is_file() {
        return Err("Embedding model path is not a file".into());
    }

    let layers = n_gpu_layers.unwrap_or(1000); // Default to full GPU offload

    engine.load_model(&validated_path, layers).map_err(|e| e.to_string())
}

/// Unload the current embedding model
#[tauri::command]
pub fn unload_embedding_model(state: State<'_, AppState>) -> Result<(), String> {
    let emb_guard = state.embeddings.read();
    let engine = emb_guard.as_ref().ok_or("Embedding engine not initialized")?;
    engine.unload_model();
    Ok(())
}

/// Get current embedding model info
#[tauri::command]
pub fn get_embedding_model_info(state: State<'_, AppState>) -> Result<Option<EmbeddingModelInfo>, String> {
    let emb_guard = state.embeddings.read();
    let engine = emb_guard.as_ref().ok_or("Embedding engine not initialized")?;
    Ok(engine.model_info())
}

/// Check if an embedding model is loaded
#[tauri::command]
pub fn is_embedding_model_loaded(state: State<'_, AppState>) -> Result<bool, String> {
    let emb_guard = state.embeddings.read();
    match emb_guard.as_ref() {
        Some(engine) => Ok(engine.is_model_loaded()),
        None => Ok(false),
    }
}

// ============================================================================
// Vector Store Commands
// ============================================================================

/// Initialize the vector store
#[tauri::command]
pub async fn init_vector_store(state: State<'_, AppState>) -> Result<(), String> {
    let app_dir = get_app_data_dir();
    let vectors_path = app_dir.join("vectors");

    // Get embedding dimension from loaded model, or use default
    let embedding_dim = {
        let emb_guard = state.embeddings.read();
        emb_guard.as_ref()
            .and_then(|e| e.embedding_dim())
            .unwrap_or(768)
    };

    let config = VectorStoreConfig {
        path: vectors_path,
        embedding_dim,
        encryption_enabled: false,
    };

    let mut store = VectorStore::new(config);
    store.init().await.map_err(|e| e.to_string())?;
    store.create_table().await.map_err(|e| e.to_string())?;

    // Check if user has consented to vector search
    let consented = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        if let Some(db) = db_lock.as_ref() {
            db.get_vector_consent()
                .map(|c| c.enabled)
                .unwrap_or(false)
        } else {
            false
        }
    };

    // Enable if user has consented
    if consented {
        store.enable(true).map_err(|e| e.to_string())?;
    }

    *state.vectors.write().await = Some(store);
    Ok(())
}

/// Enable or disable vector search
#[tauri::command]
pub async fn set_vector_enabled(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    let mut vectors_lock = state.vectors.write().await;
    let store = vectors_lock.as_mut().ok_or("Vector store not initialized")?;

    if enabled {
        store.enable(true).map_err(|e| e.to_string())?;
    } else {
        store.disable();
    }

    Ok(())
}

/// Check if vector store is enabled
#[tauri::command]
pub async fn is_vector_enabled(state: State<'_, AppState>) -> Result<bool, String> {
    let vectors_lock = state.vectors.read().await;
    Ok(vectors_lock.as_ref().map(|s| s.is_enabled()).unwrap_or(false))
}

/// Get vector store statistics
#[tauri::command]
pub async fn get_vector_stats(state: State<'_, AppState>) -> Result<VectorStats, String> {
    let vectors_lock = state.vectors.read().await;
    let store = vectors_lock.as_ref().ok_or("Vector store not initialized")?;

    let count = store.count().await.map_err(|e| e.to_string())?;

    Ok(VectorStats {
        enabled: store.is_enabled(),
        vector_count: count,
        embedding_dim: store.embedding_dim(),
        encryption_supported: store.encryption_supported(),
    })
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
    let ocr = OcrManager::new();
    let path = PathBuf::from(&image_path);

    if !path.exists() {
        return Err(format!("Image file not found: {}", image_path));
    }

    let validated_path = validate_within_home(&path).map_err(|e| match e {
        ValidationError::PathTraversal => "Image file must be within your home directory".to_string(),
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This path is blocked because it contains sensitive data".to_string()
        }
        _ => format!("Invalid image path: {}", e),
    })?;

    if !validated_path.is_file() {
        return Err("Image path is not a file".into());
    }

    let result = ocr.recognize(&validated_path).map_err(|e| e.to_string())?;

    Ok(OcrResult {
        text: result.text,
        confidence: result.confidence.unwrap_or(1.0),
    })
}

/// Maximum base64 payload size for OCR (10MB encoded = ~7.5MB decoded)
const MAX_OCR_BASE64_BYTES: usize = 10 * 1024 * 1024;

/// Process OCR from base64-encoded image data (for clipboard paste)
#[tauri::command]
pub fn process_ocr_bytes(image_base64: String) -> Result<OcrResult, String> {
    use base64::{Engine as _, engine::general_purpose};

    // Validate payload size before processing to prevent memory spikes
    if image_base64.len() > MAX_OCR_BASE64_BYTES {
        return Err(format!(
            "Image too large: {} bytes exceeds limit of {} bytes. Please use a smaller image.",
            image_base64.len(),
            MAX_OCR_BASE64_BYTES
        ));
    }

    // Decode base64
    let image_data = general_purpose::STANDARD
        .decode(&image_base64)
        .map_err(|e| format!("Invalid base64 data: {}", e))?;

    // Save to temp file
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("assistsupport_ocr_{}.png", uuid::Uuid::new_v4()));

    std::fs::write(&temp_path, &image_data)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    // Process OCR
    let ocr = OcrManager::new();
    let result = ocr.recognize(&temp_path).map_err(|e| e.to_string());

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    let ocr_result = result?;
    Ok(OcrResult {
        text: ocr_result.text,
        confidence: ocr_result.confidence.unwrap_or(1.0),
    })
}

/// Check if OCR is available on this system
#[tauri::command]
pub fn is_ocr_available() -> bool {
    let ocr = OcrManager::new();
    !ocr.available_providers().is_empty()
}

// ============================================================================
// Decision Tree Commands
// ============================================================================

/// List all decision trees
#[tauri::command]
pub fn list_decision_trees(state: State<'_, AppState>) -> Result<Vec<crate::db::DecisionTree>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_decision_trees().map_err(|e| e.to_string())
}

/// Get a single decision tree by ID
#[tauri::command]
pub fn get_decision_tree(
    state: State<'_, AppState>,
    tree_id: String,
) -> Result<crate::db::DecisionTree, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.get_decision_tree(&tree_id).map_err(|e| e.to_string())
}

// ============================================================================
// Jira Integration Commands
// ============================================================================

use crate::jira::{JiraClient, JiraConfig, JiraTicket};

/// Jira settings keys
const JIRA_BASE_URL_SETTING: &str = "jira_base_url";
const JIRA_EMAIL_SETTING: &str = "jira_email";

/// Check if Jira is configured
#[tauri::command]
pub fn is_jira_configured(state: State<'_, AppState>) -> Result<bool, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let base_url: Result<String, _> = db.conn().query_row(
        "SELECT value FROM settings WHERE key = ?",
        rusqlite::params![JIRA_BASE_URL_SETTING],
        |row| row.get(0),
    );

    let has_token = FileKeyStore::get_token(TOKEN_JIRA)
        .map(|t| t.is_some())
        .unwrap_or(false);

    Ok(base_url.is_ok() && has_token)
}

/// Get Jira configuration (without token)
#[tauri::command]
pub fn get_jira_config(state: State<'_, AppState>) -> Result<Option<JiraConfig>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let base_url: Result<String, _> = db.conn().query_row(
        "SELECT value FROM settings WHERE key = ?",
        rusqlite::params![JIRA_BASE_URL_SETTING],
        |row| row.get(0),
    );

    let email: Result<String, _> = db.conn().query_row(
        "SELECT value FROM settings WHERE key = ?",
        rusqlite::params![JIRA_EMAIL_SETTING],
        |row| row.get(0),
    );

    match (base_url, email) {
        (Ok(base_url), Ok(email)) => Ok(Some(JiraConfig { base_url, email })),
        _ => Ok(None),
    }
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
    // Validate URL format
    validate_url(&base_url).map_err(|e| e.to_string())?;

    // Enforce HTTPS by default
    let using_http = is_http_url(&base_url);
    if using_http {
        if allow_http != Some(true) {
            return Err(
                "HTTPS is required for Jira connections. HTTP connections expose credentials \
                 in transit. If you must use HTTP (e.g., local testing), enable the \
                 'allow_http' option explicitly.".to_string()
            );
        }
        // Log security warning for HTTP opt-in
        audit::audit_jira_http_opt_in(&base_url);
    }

    // Test connection first
    let client = JiraClient::new(&base_url, &email, &api_token);
    if !client.test_connection().await.map_err(|e| e.to_string())? {
        return Err("Connection failed - check credentials".to_string());
    }

    // Store token in file storage
    FileKeyStore::store_token(TOKEN_JIRA, &api_token).map_err(|e| e.to_string())?;
    audit::audit_token_set("jira");

    // Store config in DB
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.conn().execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
        rusqlite::params![JIRA_BASE_URL_SETTING, &base_url],
    ).map_err(|e| e.to_string())?;

    db.conn().execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
        rusqlite::params![JIRA_EMAIL_SETTING, &email],
    ).map_err(|e| e.to_string())?;

    // Store HTTP opt-in preference if used
    if using_http {
        db.conn().execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            rusqlite::params!["jira_http_opt_in", "true"],
        ).map_err(|e| e.to_string())?;
    } else {
        // Clear HTTP opt-in if switching to HTTPS
        db.conn().execute(
            "DELETE FROM settings WHERE key = ?",
            rusqlite::params!["jira_http_opt_in"],
        ).map_err(|e| e.to_string())?;
    }

    // Audit log successful configuration
    audit::audit_jira_configured(!using_http);

    Ok(())
}

/// Clear Jira configuration
#[tauri::command]
pub fn clear_jira_config(state: State<'_, AppState>) -> Result<(), String> {
    // Delete token from file storage
    let _ = FileKeyStore::delete_token(TOKEN_JIRA);
    audit::audit_token_cleared("jira");

    // Delete config from DB
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.conn().execute(
        "DELETE FROM settings WHERE key IN (?, ?)",
        rusqlite::params![JIRA_BASE_URL_SETTING, JIRA_EMAIL_SETTING],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

/// Get a Jira ticket by key
#[tauri::command]
pub async fn get_jira_ticket(
    state: State<'_, AppState>,
    ticket_key: String,
) -> Result<JiraTicket, String> {
    // Validate ticket key format
    validate_ticket_id(&ticket_key).map_err(|e| e.to_string())?;

    // Get config from DB
    let (base_url, email) = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;

        let base_url: String = db.conn().query_row(
            "SELECT value FROM settings WHERE key = ?",
            rusqlite::params![JIRA_BASE_URL_SETTING],
            |row| row.get(0),
        ).map_err(|_| "Jira not configured")?;

        let email: String = db.conn().query_row(
            "SELECT value FROM settings WHERE key = ?",
            rusqlite::params![JIRA_EMAIL_SETTING],
            |row| row.get(0),
        ).map_err(|_| "Jira not configured")?;

        (base_url, email)
    };

    // Get token from file storage
    let token = FileKeyStore::get_token(TOKEN_JIRA)
        .map_err(|e| e.to_string())?
        .ok_or("Jira token not found")?;

    // Fetch ticket
    let client = JiraClient::new(&base_url, &email, &token);
    client.get_ticket(&ticket_key).await.map_err(|e| e.to_string())
}

/// Add a comment to a Jira ticket (Phase 18)
#[tauri::command]
pub async fn add_jira_comment(
    state: State<'_, AppState>,
    ticket_key: String,
    comment_body: String,
    visibility: Option<String>,
) -> Result<String, String> {
    use crate::jira::CommentVisibility;

    // Validate ticket key format
    validate_ticket_id(&ticket_key).map_err(|e| e.to_string())?;

    // Get config from DB
    let (base_url, email) = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;

        let base_url: String = db.conn().query_row(
            "SELECT value FROM settings WHERE key = ?",
            rusqlite::params![JIRA_BASE_URL_SETTING],
            |row| row.get(0),
        ).map_err(|_| "Jira not configured")?;

        let email: String = db.conn().query_row(
            "SELECT value FROM settings WHERE key = ?",
            rusqlite::params![JIRA_EMAIL_SETTING],
            |row| row.get(0),
        ).map_err(|_| "Jira not configured")?;

        (base_url, email)
    };

    // Get token from file storage
    let token = FileKeyStore::get_token(TOKEN_JIRA)
        .map_err(|e| e.to_string())?
        .ok_or("Jira token not found")?;

    // Parse visibility
    let vis = visibility.map(|v| match v.as_str() {
        "internal" => CommentVisibility::Internal,
        "public" => CommentVisibility::Public,
        _ if v.starts_with("role:") => CommentVisibility::Role(v[5..].to_string()),
        _ if v.starts_with("group:") => CommentVisibility::Group(v[6..].to_string()),
        _ => CommentVisibility::Public,
    });

    // Post comment
    let client = JiraClient::new(&base_url, &email, &token);
    client.add_comment(&ticket_key, &comment_body, vis)
        .await
        .map_err(|e| e.to_string())
}

/// Push draft to Jira as a comment with KB citations (Phase 18)
#[tauri::command]
pub async fn push_draft_to_jira(
    state: State<'_, AppState>,
    draft_id: String,
    ticket_key: String,
    visibility: Option<String>,
) -> Result<String, String> {
    use crate::jira::{CommentVisibility, KbCitation};

    // Validate ticket key format
    validate_ticket_id(&ticket_key).map_err(|e| e.to_string())?;

    // Get draft and parse KB sources
    let (response_text, sources_json) = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;
        let draft = db.get_draft(&draft_id).map_err(|e| e.to_string())?;

        let response = draft.response_text.ok_or("Draft has no response text")?;
        (response, draft.kb_sources_json)
    };

    // Parse citations from KB sources JSON
    let citations: Vec<KbCitation> = sources_json
        .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        .map(|sources| {
            sources.iter().map(|s| KbCitation {
                title: s["title"].as_str().unwrap_or("Unknown").to_string(),
                url: s["url"].as_str().map(|u| u.to_string()),
                chunk_id: s["chunk_id"].as_str().map(|c| c.to_string()),
            }).collect()
        })
        .unwrap_or_default();

    // Get Jira config
    let (base_url, email) = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;

        let base_url: String = db.conn().query_row(
            "SELECT value FROM settings WHERE key = ?",
            rusqlite::params![JIRA_BASE_URL_SETTING],
            |row| row.get(0),
        ).map_err(|_| "Jira not configured")?;

        let email: String = db.conn().query_row(
            "SELECT value FROM settings WHERE key = ?",
            rusqlite::params![JIRA_EMAIL_SETTING],
            |row| row.get(0),
        ).map_err(|_| "Jira not configured")?;

        (base_url, email)
    };

    // Get token
    let token = FileKeyStore::get_token(TOKEN_JIRA)
        .map_err(|e| e.to_string())?
        .ok_or("Jira token not found")?;

    // Parse visibility
    let vis = visibility.map(|v| match v.as_str() {
        "internal" => CommentVisibility::Internal,
        "public" => CommentVisibility::Public,
        _ if v.starts_with("role:") => CommentVisibility::Role(v[5..].to_string()),
        _ if v.starts_with("group:") => CommentVisibility::Group(v[6..].to_string()),
        _ => CommentVisibility::Public,
    });

    // Post comment with citations
    let client = JiraClient::new(&base_url, &email, &token);
    client.add_comment_with_citations(&ticket_key, &response_text, &citations, vis)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// Export Commands (Phase 18)
// ============================================================================

use crate::exports::{
    ExportFormat as DraftExportFormat,
    SafeExportOptions, ExportedSource, format_draft, format_for_clipboard
};

/// Export a draft in various formats
#[tauri::command]
pub fn export_draft_formatted(
    state: State<'_, AppState>,
    draft_id: String,
    format: String,
    safe_export: Option<SafeExportOptions>,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    let draft = db.get_draft(&draft_id).map_err(|e| e.to_string())?;

    let response_text = draft.response_text.as_deref().unwrap_or("");

    // Parse KB sources
    let sources: Vec<ExportedSource> = draft.kb_sources_json
        .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        .map(|sources| {
            sources.iter().map(|s| ExportedSource {
                title: s["title"].as_str().unwrap_or("Unknown").to_string(),
                path: s["file_path"].as_str().map(|p| p.to_string()),
                url: s["url"].as_str().map(|u| u.to_string()),
            }).collect()
        })
        .unwrap_or_default();

    let export_format = match format.as_str() {
        "html" => DraftExportFormat::Html,
        "ticket_html" => DraftExportFormat::TicketHtml,
        "json" => DraftExportFormat::Json,
        _ => DraftExportFormat::Plaintext,
    };

    Ok(format_draft(
        response_text,
        draft.summary_text.as_deref(),
        &sources,
        export_format,
        safe_export.as_ref(),
    ))
}

/// Format draft for clipboard (optimized for ticket systems)
#[tauri::command]
pub fn format_draft_for_clipboard(
    state: State<'_, AppState>,
    draft_id: String,
    include_sources: bool,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    let draft = db.get_draft(&draft_id).map_err(|e| e.to_string())?;

    let response_text = draft.response_text.as_deref().unwrap_or("");

    // Parse KB sources
    let sources: Vec<ExportedSource> = draft.kb_sources_json
        .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        .map(|sources| {
            sources.iter().map(|s| ExportedSource {
                title: s["title"].as_str().unwrap_or("Unknown").to_string(),
                path: s["file_path"].as_str().map(|p| p.to_string()),
                url: s["url"].as_str().map(|u| u.to_string()),
            }).collect()
        })
        .unwrap_or_default();

    Ok(format_for_clipboard(response_text, &sources, include_sources))
}

// ============================================================================
// Draft & Template Commands
// ============================================================================

use crate::db::{SavedDraft, ResponseTemplate};

/// List saved drafts (most recent first)
#[tauri::command]
pub fn list_drafts(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<SavedDraft>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_drafts(limit.unwrap_or(50)).map_err(|e| e.to_string())
}

/// Search drafts by text content
#[tauri::command]
pub fn search_drafts(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<SavedDraft>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.search_drafts(&query, limit.unwrap_or(50)).map_err(|e| e.to_string())
}

/// Get a single draft by ID
#[tauri::command]
pub fn get_draft(
    state: State<'_, AppState>,
    draft_id: String,
) -> Result<SavedDraft, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.get_draft(&draft_id).map_err(|e| e.to_string())
}

/// Save a draft (insert or update)
#[tauri::command]
pub fn save_draft(
    state: State<'_, AppState>,
    draft: SavedDraft,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.save_draft(&draft).map_err(|e| e.to_string())
}

/// Delete a draft by ID
#[tauri::command]
pub fn delete_draft(
    state: State<'_, AppState>,
    draft_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.delete_draft(&draft_id).map_err(|e| e.to_string())
}

/// List autosave drafts (most recent first)
#[tauri::command]
pub fn list_autosaves(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<SavedDraft>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_autosaves(limit.unwrap_or(10)).map_err(|e| e.to_string())
}

/// Cleanup old autosaves, keeping only the most recent ones
#[tauri::command]
pub fn cleanup_autosaves(
    state: State<'_, AppState>,
    keep_count: Option<usize>,
) -> Result<usize, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.cleanup_autosaves(keep_count.unwrap_or(10)).map_err(|e| e.to_string())
}

/// Get draft versions by input hash (autosaves with matching input_text hash)
/// Used for version history UI
#[tauri::command]
pub fn get_draft_versions(
    state: State<'_, AppState>,
    input_hash: String,
) -> Result<Vec<SavedDraft>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.get_draft_versions(&input_hash).map_err(|e| e.to_string())
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
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.create_draft_version(&draft_id, change_reason.as_deref())
        .map_err(|e| e.to_string())
}

/// List draft versions for a specific draft
#[tauri::command]
pub fn list_draft_versions(
    state: State<'_, AppState>,
    draft_id: String,
) -> Result<Vec<crate::db::DraftVersion>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_draft_versions(&draft_id).map_err(|e| e.to_string())
}

/// Finalize a draft (lock and mark as read-only)
#[tauri::command]
pub fn finalize_draft(
    state: State<'_, AppState>,
    draft_id: String,
    finalized_by: Option<String>,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.finalize_draft(&draft_id, finalized_by.as_deref())
        .map_err(|e| e.to_string())
}

/// Archive a draft
#[tauri::command]
pub fn archive_draft(
    state: State<'_, AppState>,
    draft_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.archive_draft(&draft_id).map_err(|e| e.to_string())
}

/// Update draft handoff summary for escalations
#[tauri::command]
pub fn update_draft_handoff(
    state: State<'_, AppState>,
    draft_id: String,
    handoff_summary: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.update_draft_handoff(&draft_id, &handoff_summary)
        .map_err(|e| e.to_string())
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
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_playbooks(category.as_deref()).map_err(|e| e.to_string())
}

/// Get a playbook by ID
#[tauri::command]
pub fn get_playbook(
    state: State<'_, AppState>,
    playbook_id: String,
) -> Result<crate::db::Playbook, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.get_playbook(&playbook_id).map_err(|e| e.to_string())
}

/// Save a playbook (insert or update)
#[tauri::command]
pub fn save_playbook(
    state: State<'_, AppState>,
    playbook: crate::db::Playbook,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.save_playbook(&playbook).map_err(|e| e.to_string())
}

/// Record playbook usage
#[tauri::command]
pub fn use_playbook(
    state: State<'_, AppState>,
    playbook_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.increment_playbook_usage(&playbook_id).map_err(|e| e.to_string())
}

/// Delete a playbook
#[tauri::command]
pub fn delete_playbook(
    state: State<'_, AppState>,
    playbook_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.delete_playbook(&playbook_id).map_err(|e| e.to_string())
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
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_action_shortcuts(category.as_deref()).map_err(|e| e.to_string())
}

/// Get an action shortcut by ID
#[tauri::command]
pub fn get_action_shortcut(
    state: State<'_, AppState>,
    shortcut_id: String,
) -> Result<crate::db::ActionShortcut, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.get_action_shortcut(&shortcut_id).map_err(|e| e.to_string())
}

/// Save an action shortcut (insert or update)
#[tauri::command]
pub fn save_action_shortcut(
    state: State<'_, AppState>,
    shortcut: crate::db::ActionShortcut,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.save_action_shortcut(&shortcut).map_err(|e| e.to_string())
}

/// Delete an action shortcut
#[tauri::command]
pub fn delete_action_shortcut(
    state: State<'_, AppState>,
    shortcut_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.delete_action_shortcut(&shortcut_id).map_err(|e| e.to_string())
}

/// List all response templates
#[tauri::command]
pub fn list_templates(state: State<'_, AppState>) -> Result<Vec<ResponseTemplate>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_templates().map_err(|e| e.to_string())
}

/// Get a single template by ID
#[tauri::command]
pub fn get_template(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<ResponseTemplate, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.get_template(&template_id).map_err(|e| e.to_string())
}

/// Save a template (insert or update)
#[tauri::command]
pub fn save_template(
    state: State<'_, AppState>,
    template: ResponseTemplate,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    let template_id = template.id.clone();
    db.save_template(&template).map_err(|e| e.to_string())?;
    Ok(template_id)
}

/// Delete a template by ID
#[tauri::command]
pub fn delete_template(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.delete_template(&template_id).map_err(|e| e.to_string())
}

// ============================================================================
// Custom Variable Commands
// ============================================================================

/// List all custom template variables
#[tauri::command]
pub fn list_custom_variables(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::CustomVariable>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_custom_variables().map_err(|e| e.to_string())
}

/// Get a custom variable by ID
#[tauri::command]
pub fn get_custom_variable(
    state: State<'_, AppState>,
    variable_id: String,
) -> Result<crate::db::CustomVariable, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.get_custom_variable(&variable_id).map_err(|e| e.to_string())
}

/// Save a custom variable (create or update)
#[tauri::command]
pub fn save_custom_variable(
    state: State<'_, AppState>,
    variable: crate::db::CustomVariable,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.save_custom_variable(&variable).map_err(|e| e.to_string())
}

/// Delete a custom variable by ID
#[tauri::command]
pub fn delete_custom_variable(
    state: State<'_, AppState>,
    variable_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.delete_custom_variable(&variable_id).map_err(|e| e.to_string())
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

/// Ingest a web page URL
/// Uses block_in_place to run async operations while holding DB lock
#[tauri::command]
pub fn ingest_url(
    state: State<'_, AppState>,
    url: String,
    namespace_id: String,
) -> Result<IngestResult, String> {
    use crate::kb::ingest::web::{WebIngester, WebIngestConfig};
    use crate::kb::ingest::CancellationToken;

    // Validate and normalize namespace ID
    let namespace_id = normalize_and_validate_namespace_id(&namespace_id)
        .map_err(|e| e.to_string())?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    // Ensure namespace exists
    db.ensure_namespace_exists(&namespace_id).map_err(|e| e.to_string())?;

    let config = WebIngestConfig::default();
    let ingester = WebIngester::new(config).map_err(|e| e.to_string())?;
    let cancel_token = CancellationToken::new();

    // Use block_in_place to run async code in sync context
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            ingester
                .ingest_page(db, &url, &namespace_id, &cancel_token, None)
                .await
        })
    })
    .map_err(|e| e.to_string())?;

    Ok(IngestResult {
        document_id: result.id,
        title: result.title,
        source_uri: result.source_uri,
        chunk_count: result.chunk_count,
        word_count: result.word_count,
    })
}

/// Ingest a YouTube video transcript
/// Uses block_in_place to run async operations while holding DB lock
#[tauri::command]
pub fn ingest_youtube(
    state: State<'_, AppState>,
    url: String,
    namespace_id: String,
) -> Result<IngestResult, String> {
    use crate::kb::ingest::youtube::{YouTubeIngester, YouTubeIngestConfig};
    use crate::kb::ingest::CancellationToken;

    // Validate and normalize namespace ID
    let namespace_id = normalize_and_validate_namespace_id(&namespace_id)
        .map_err(|e| e.to_string())?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    // Ensure namespace exists
    db.ensure_namespace_exists(&namespace_id).map_err(|e| e.to_string())?;

    let config = YouTubeIngestConfig::default();
    let ingester = YouTubeIngester::new(config);

    // Check yt-dlp availability
    if !ingester.check_ytdlp_available() {
        return Err("yt-dlp not found. Install with: brew install yt-dlp".to_string());
    }

    let cancel_token = CancellationToken::new();

    // Use block_in_place to run async code in sync context
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            ingester
                .ingest_video(db, &url, &namespace_id, &cancel_token, None)
                .await
        })
    })
    .map_err(|e| e.to_string())?;

    Ok(IngestResult {
        document_id: result.id,
        title: result.title,
        source_uri: result.source_uri,
        chunk_count: result.chunk_count,
        word_count: result.word_count,
    })
}

/// Ingest a GitHub repository (local path)
/// Path must be within user's home directory
#[tauri::command]
pub fn ingest_github(
    state: State<'_, AppState>,
    repo_path: String,
    namespace_id: String,
) -> Result<Vec<IngestResult>, String> {
    use crate::kb::ingest::github::{GitHubIngester, GitHubIngestConfig};
    use crate::kb::ingest::CancellationToken;
    use std::path::Path;

    // Validate and normalize namespace ID
    let namespace_id = normalize_and_validate_namespace_id(&namespace_id)
        .map_err(|e| e.to_string())?;

    // Validate path is within home directory
    let validated_path = validate_within_home(Path::new(&repo_path)).map_err(|e| match e {
        ValidationError::PathTraversal => {
            "Repository must be within your home directory".to_string()
        }
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This directory cannot be used as it contains sensitive data".to_string()
        }
        _ => format!("Invalid repository path: {}", e),
    })?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    // Ensure namespace exists
    db.ensure_namespace_exists(&namespace_id).map_err(|e| e.to_string())?;

    let config = GitHubIngestConfig::default();
    let ingester = GitHubIngester::new(config);
    let cancel_token = CancellationToken::new();

    let results = ingester
        .ingest_local_repo(db, &validated_path, &namespace_id, &cancel_token, None)
        .map_err(|e| e.to_string())?;

    Ok(results
        .into_iter()
        .map(|r| IngestResult {
            document_id: r.id,
            title: r.title,
            source_uri: r.source_uri,
            chunk_count: r.chunk_count,
            word_count: r.word_count,
        })
        .collect())
}

/// Process a YAML source file for batch ingestion
/// Uses block_in_place to run async operations while holding DB lock
#[tauri::command]
pub fn process_source_file(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<BatchIngestResult, String> {
    use crate::sources::SourceFile;
    use crate::kb::ingest::batch::{BatchIngester, BatchIngestConfig};
    use crate::kb::ingest::CancellationToken;
    use std::path::Path;

    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(format!("Source file not found: {}", file_path));
    }

    let validated_path = validate_within_home(path).map_err(|e| match e {
        ValidationError::PathTraversal => "Source file must be within your home directory".to_string(),
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This path is blocked because it contains sensitive data".to_string()
        }
        _ => format!("Invalid source file path: {}", e),
    })?;

    if !validated_path.is_file() {
        return Err("Source file path is not a file".into());
    }

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    // Parse the source file
    let source_file = SourceFile::from_path(&validated_path)
        .map_err(|e| e.to_string())?;

    // Ensure namespace exists
    db.ensure_namespace_exists(&source_file.namespace).map_err(|e| e.to_string())?;

    // Convert to batch sources
    let sources: Vec<String> = source_file
        .enabled_sources()
        .map(|s| s.uri.clone())
        .collect();

    let config = BatchIngestConfig::default();
    let ingester = BatchIngester::new(config).map_err(|e| e.to_string())?;
    let cancel_token = CancellationToken::new();
    let namespace = source_file.namespace.clone();

    // Use block_in_place to run async code in sync context
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            ingester
                .ingest_from_strings(db, &sources, &namespace, &cancel_token, None)
                .await
        })
    });

    Ok(BatchIngestResult {
        successful: result
            .successful
            .into_iter()
            .map(|r| IngestResult {
                document_id: r.id,
                title: r.title,
                source_uri: r.source_uri,
                chunk_count: r.chunk_count,
                word_count: r.word_count,
            })
            .collect(),
        failed: result
            .failed
            .into_iter()
            .map(|f| FailedSource {
                source: f.source,
                error: f.error,
            })
            .collect(),
        cancelled: result.cancelled,
    })
}

/// List all namespaces
#[tauri::command]
pub fn list_namespaces(state: State<'_, AppState>) -> Result<Vec<crate::db::Namespace>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_namespaces().map_err(|e| e.to_string())
}

/// List all namespaces with document and source counts (optimized single query)
#[tauri::command]
pub fn list_namespaces_with_counts(state: State<'_, AppState>) -> Result<Vec<crate::db::NamespaceWithCounts>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_namespaces_with_counts().map_err(|e| e.to_string())
}

/// Create a new namespace
#[tauri::command]
pub fn create_namespace(
    state: State<'_, AppState>,
    name: String,
    description: Option<String>,
    color: Option<String>,
) -> Result<crate::db::Namespace, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.create_namespace(&name, description.as_deref(), color.as_deref())
        .map_err(|e| e.to_string())
}

/// Rename a namespace
#[tauri::command]
pub fn rename_namespace(
    state: State<'_, AppState>,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.rename_namespace(&old_name, &new_name).map_err(|e| e.to_string())
}

/// Delete a namespace and all its content
#[tauri::command]
pub fn delete_namespace(state: State<'_, AppState>, name: String) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.delete_namespace(&name).map_err(|e| e.to_string())
}

/// List ingestion sources, optionally filtered by namespace
#[tauri::command]
pub fn list_ingest_sources(
    state: State<'_, AppState>,
    namespace_id: Option<String>,
) -> Result<Vec<crate::db::IngestSource>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_ingest_sources(namespace_id.as_deref()).map_err(|e| e.to_string())
}

/// Delete an ingestion source and its documents
#[tauri::command]
pub fn delete_ingest_source(state: State<'_, AppState>, source_id: String) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.delete_ingest_source(&source_id).map_err(|e| e.to_string())
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
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    // Get all sources with document counts
    let sql = r#"
        SELECT
            s.id,
            s.source_type,
            s.source_uri,
            s.title,
            s.status,
            s.error_message,
            s.last_ingested_at,
            COUNT(d.id) as document_count,
            CASE
                WHEN s.last_ingested_at IS NOT NULL
                THEN julianday('now') - julianday(s.last_ingested_at)
                ELSE NULL
            END as days_since
        FROM ingest_sources s
        LEFT JOIN kb_documents d ON d.source_id = s.id
        WHERE (?1 IS NULL OR s.namespace_id = ?1)
        GROUP BY s.id
        ORDER BY s.updated_at DESC
    "#;

    let mut stmt = db.conn().prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([namespace_id], |row| {
        Ok(SourceHealth {
            id: row.get(0)?,
            source_type: row.get(1)?,
            source_uri: row.get(2)?,
            title: row.get(3)?,
            status: row.get(4)?,
            error_message: row.get(5)?,
            last_ingested_at: row.get(6)?,
            document_count: row.get::<_, i64>(7)? as u32,
            days_since_refresh: row.get(8)?,
        })
    }).map_err(|e| e.to_string())?;

    let sources: Vec<SourceHealth> = rows.filter_map(|r| r.ok()).collect();

    let mut summary = SourceHealthSummary {
        total_sources: sources.len() as u32,
        active_sources: 0,
        stale_sources: 0,
        error_sources: 0,
        pending_sources: 0,
        sources,
    };

    for source in &summary.sources {
        match source.status.as_str() {
            "active" => summary.active_sources += 1,
            "stale" => summary.stale_sources += 1,
            "error" => summary.error_sources += 1,
            "pending" => summary.pending_sources += 1,
            _ => {}
        }
    }

    Ok(summary)
}

/// Retry a failed or stale source
#[tauri::command]
pub fn retry_source(
    state: State<'_, AppState>,
    source_id: String,
) -> Result<IngestResult, String> {
    // Get source details
    let source = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;
        db.get_ingest_source(&source_id).map_err(|e| e.to_string())?
    };

    // Mark as pending
    {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;
        db.update_ingest_source_status(&source_id, "pending", None)
            .map_err(|e| e.to_string())?;
    }

    // Re-ingest based on source type
    match source.source_type.as_str() {
        "web" => {
            ingest_url(state, source.source_uri, source.namespace_id)
        }
        "youtube" => {
            ingest_youtube(state, source.source_uri, source.namespace_id)
        }
        "github" => {
            let results: Vec<IngestResult> = ingest_github(state, source.source_uri.clone(), source.namespace_id)?;
            // Return summary for multi-file results
            Ok(IngestResult {
                document_id: source_id,
                title: source.title.unwrap_or_else(|| "Repository".to_string()),
                source_uri: source.source_uri,
                chunk_count: results.iter().map(|r| r.chunk_count).sum(),
                word_count: results.iter().map(|r| r.word_count).sum(),
            })
        }
        _ => Err(format!("Unknown source type: {}", source.source_type)),
    }
}

/// Mark sources as stale if they haven't been refreshed in N days
#[tauri::command]
pub fn mark_stale_sources(
    state: State<'_, AppState>,
    days_threshold: Option<u32>,
) -> Result<u32, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let days = days_threshold.unwrap_or(7) as i64;

    let sql = r#"
        UPDATE ingest_sources
        SET status = 'stale', updated_at = datetime('now')
        WHERE status = 'active'
        AND last_ingested_at IS NOT NULL
        AND julianday('now') - julianday(last_ingested_at) > ?
    "#;

    let count = db.conn()
        .execute(sql, [days])
        .map_err(|e| e.to_string())?;

    Ok(count as u32)
}

/// Get document chunks
#[tauri::command]
pub fn get_document_chunks(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<Vec<DocumentChunk>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let chunks: Vec<DocumentChunk> = db
        .conn()
        .prepare(
            "SELECT id, chunk_index, heading_path, content, word_count
             FROM kb_chunks WHERE document_id = ? ORDER BY chunk_index",
        )
        .map_err(|e| e.to_string())?
        .query_map([&document_id], |row| {
            Ok(DocumentChunk {
                id: row.get(0)?,
                chunk_index: row.get(1)?,
                heading_path: row.get(2)?,
                content: row.get(3)?,
                word_count: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(chunks)
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
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.conn()
        .execute("DELETE FROM kb_documents WHERE id = ?", [&document_id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Clear all knowledge data, optionally for a specific namespace
#[tauri::command]
pub fn clear_knowledge_data(
    state: State<'_, AppState>,
    namespace_id: Option<String>,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    match namespace_id {
        Some(ns) => {
            // Clear only the specified namespace
            db.conn()
                .execute("DELETE FROM kb_documents WHERE namespace_id = ?", [&ns])
                .map_err(|e| e.to_string())?;
            db.conn()
                .execute("DELETE FROM ingest_sources WHERE namespace_id = ?", [&ns])
                .map_err(|e| e.to_string())?;
        }
        None => {
            // Clear all knowledge data
            db.conn()
                .execute("DELETE FROM kb_chunks", [])
                .map_err(|e| e.to_string())?;
            db.conn()
                .execute("DELETE FROM kb_documents", [])
                .map_err(|e| e.to_string())?;
            db.conn()
                .execute("DELETE FROM ingest_runs", [])
                .map_err(|e| e.to_string())?;
            db.conn()
                .execute("DELETE FROM ingest_sources", [])
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

/// Check if yt-dlp is available
#[tauri::command]
pub fn check_ytdlp_available() -> Result<bool, String> {
    use std::process::Command;

    Ok(Command::new("yt-dlp")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false))
}

// ============================================================================
// Job Commands
// ============================================================================

use crate::jobs::{Job, JobStatus, JobType};

/// Job summary for list responses (excludes logs and metadata)
#[derive(Debug, Clone, serde::Serialize)]
pub struct JobSummary {
    pub id: String,
    pub job_type: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub progress: f32,
    pub progress_message: Option<String>,
    pub error: Option<String>,
}

impl From<Job> for JobSummary {
    fn from(job: Job) -> Self {
        Self {
            id: job.id,
            job_type: job.job_type.to_string(),
            status: job.status.to_string(),
            created_at: job.created_at.to_rfc3339(),
            updated_at: job.updated_at.to_rfc3339(),
            progress: job.progress,
            progress_message: job.progress_message,
            error: job.error,
        }
    }
}

/// Create a new job
#[tauri::command]
pub fn create_job(
    state: State<'_, AppState>,
    job_type: String,
    metadata: Option<serde_json::Value>,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let job_type_enum = JobType::from_str(&job_type);
    let mut job = Job::new(job_type_enum);
    if let Some(meta) = metadata {
        job = job.with_metadata(meta);
    }

    let job_id = job.id.clone();
    db.create_job(&job).map_err(|e| e.to_string())?;

    Ok(job_id)
}

/// List jobs, optionally filtered by status
#[tauri::command]
pub fn list_jobs(
    state: State<'_, AppState>,
    status: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<JobSummary>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let status_filter = status.as_deref().and_then(JobStatus::from_str);
    let jobs = db
        .list_jobs(status_filter, limit.unwrap_or(50))
        .map_err(|e| e.to_string())?;

    Ok(jobs.into_iter().map(JobSummary::from).collect())
}

/// Get a single job by ID
#[tauri::command]
pub fn get_job(state: State<'_, AppState>, job_id: String) -> Result<Option<Job>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.get_job(&job_id).map_err(|e| e.to_string())
}

/// Cancel a job (signals cancellation token and sets status to cancelled)
#[tauri::command]
pub fn cancel_job(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    // Signal cancellation to any running task
    state.jobs.cancel_job(&job_id);

    // Update database status
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.update_job_status(&job_id, JobStatus::Cancelled, Some("Cancelled by user"))
        .map_err(|e| e.to_string())
}

/// Get logs for a job
#[tauri::command]
pub fn get_job_logs(
    state: State<'_, AppState>,
    job_id: String,
    limit: Option<usize>,
) -> Result<Vec<crate::jobs::JobLog>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.get_job_logs(&job_id, limit.unwrap_or(100))
        .map_err(|e| e.to_string())
}

/// Get job counts by status
#[tauri::command]
pub fn get_job_counts(state: State<'_, AppState>) -> Result<Vec<(String, i64)>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.get_job_counts().map_err(|e| e.to_string())
}

/// Clean up old completed jobs
#[tauri::command]
pub fn cleanup_old_jobs(
    state: State<'_, AppState>,
    keep_days: Option<i64>,
) -> Result<usize, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.cleanup_old_jobs(keep_days.unwrap_or(30))
        .map_err(|e| e.to_string())
}

// ============================================================================
// Document Versioning Commands (Phase 14)
// ============================================================================

/// List versions of a document
#[tauri::command]
pub fn list_document_versions(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<Vec<crate::db::DocumentVersion>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.list_document_versions(&document_id)
        .map_err(|e| e.to_string())
}

/// Rollback a document to a previous version
#[tauri::command]
pub fn rollback_document(
    state: State<'_, AppState>,
    document_id: String,
    version_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.rollback_document(&document_id, &version_id)
        .map_err(|e| e.to_string())
}

// ============================================================================
// Source Trust Commands (Phase 14)
// ============================================================================

/// Update trust score for a source
#[tauri::command]
pub fn update_source_trust(
    state: State<'_, AppState>,
    source_id: String,
    trust_score: f64,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.update_source_trust(&source_id, trust_score)
        .map_err(|e| e.to_string())
}

/// Pin or unpin a source
#[tauri::command]
pub fn set_source_pinned(
    state: State<'_, AppState>,
    source_id: String,
    pinned: bool,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.set_source_pinned(&source_id, pinned)
        .map_err(|e| e.to_string())
}

/// Set review status for a source
#[tauri::command]
pub fn set_source_review_status(
    state: State<'_, AppState>,
    source_id: String,
    status: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.set_source_review_status(&source_id, &status)
        .map_err(|e| e.to_string())
}

/// Get stale sources for review
#[tauri::command]
pub fn get_stale_sources(
    state: State<'_, AppState>,
    namespace_id: Option<String>,
) -> Result<Vec<crate::db::IngestSource>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.get_stale_sources(namespace_id.as_deref())
        .map_err(|e| e.to_string())
}

// ============================================================================
// Namespace Rules Commands (Phase 14)
// ============================================================================

/// Add a namespace ingestion rule
#[tauri::command]
pub fn add_namespace_rule(
    state: State<'_, AppState>,
    namespace_id: String,
    rule_type: String,
    pattern_type: String,
    pattern: String,
    reason: Option<String>,
) -> Result<String, String> {
    // Validate and normalize namespace ID
    let namespace_id = normalize_and_validate_namespace_id(&namespace_id)
        .map_err(|e| e.to_string())?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.add_namespace_rule(&namespace_id, &rule_type, &pattern_type, &pattern, reason.as_deref())
        .map_err(|e| e.to_string())
}

/// Delete a namespace rule
#[tauri::command]
pub fn delete_namespace_rule(
    state: State<'_, AppState>,
    rule_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.delete_namespace_rule(&rule_id)
        .map_err(|e| e.to_string())
}

/// List rules for a namespace
#[tauri::command]
pub fn list_namespace_rules(
    state: State<'_, AppState>,
    namespace_id: String,
) -> Result<Vec<crate::db::NamespaceRule>, String> {
    // Validate and normalize namespace ID
    let namespace_id = normalize_and_validate_namespace_id(&namespace_id)
        .map_err(|e| e.to_string())?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.list_namespace_rules(&namespace_id)
        .map_err(|e| e.to_string())
}

// Diagnostics commands moved to commands/diagnostics.rs
