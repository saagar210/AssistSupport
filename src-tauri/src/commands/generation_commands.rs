use super::*;

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

    // Validate and normalize namespace_id if provided
    let namespace_id = namespace_id
        .map(|ns| normalize_and_validate_namespace_id(&ns))
        .transpose()
        .map_err(|e| e.to_string())?;

    let limit = limit.unwrap_or(10).min(100); // Cap limit at 100

    // Build search options
    let mut search_opts = SearchOptions::new(limit)
        .with_namespace(namespace_id.clone())
        .with_query_text(&query);

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

        if let (Some(vectors), Some(embeddings)) = (vectors_lock.as_ref(), embeddings_lock.as_ref())
        {
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
    let mut results = HybridSearch::fuse_results_with_options(
        db,
        fts_results,
        vector_results,
        search_opts.clone(),
    )
    .map_err(|e| e.to_string())?;

    // Apply post-processing (policy boost, score normalization, snippet sanitization)
    results = HybridSearch::post_process_results(results, &search_opts);

    Ok(results)
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

/// Initialize the LLM engine (idempotent — skips if already initialized)
#[tauri::command]
pub fn init_llm_engine(state: State<'_, AppState>) -> Result<(), String> {
    model_commands::init_llm_engine_impl(state)
}

/// Load a model by ID
#[tauri::command]
pub fn load_model(
    state: State<'_, AppState>,
    model_id: String,
    n_gpu_layers: Option<u32>,
) -> Result<ModelInfo, String> {
    model_commands::load_model_impl(state, model_id, n_gpu_layers)
}

/// Load a custom GGUF model from a file path
#[tauri::command]
pub fn load_custom_model(
    state: State<'_, AppState>,
    model_path: String,
    n_gpu_layers: Option<u32>,
) -> Result<ModelInfo, String> {
    model_commands::load_custom_model_impl(state, model_path, n_gpu_layers)
}

/// Validate a GGUF file without loading it (returns model metadata)
#[tauri::command]
pub fn validate_gguf_file(model_path: String) -> Result<GgufFileInfo, String> {
    model_commands::validate_gguf_file_impl(model_path)
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
    model_commands::unload_model_impl(state)
}

/// Get current model info
#[tauri::command]
pub fn get_model_info(state: State<'_, AppState>) -> Result<Option<ModelInfo>, String> {
    model_commands::get_model_info_impl(state)
}

/// Check if a model is loaded
#[tauri::command]
pub fn is_model_loaded(state: State<'_, AppState>) -> Result<bool, String> {
    model_commands::is_model_loaded_impl(state)
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
        if let Some(v) = p.max_tokens {
            params.max_tokens = v;
        }
        if let Some(v) = p.temperature {
            params.temperature = v;
        }
        if let Some(v) = p.top_p {
            params.top_p = v;
        }
        if let Some(v) = p.top_k {
            params.top_k = v;
        }
        if let Some(v) = p.repeat_penalty {
            params.repeat_penalty = v;
        }
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
        let temp_engine = LlmEngine {
            state: engine_state,
        };
        let _ = temp_engine
            .generate_streaming(&prompt_clone, gen_params, tx_clone, cancel_clone)
            .await;
    });

    // Collect output
    let mut text = String::new();
    let mut tokens_generated = 0u32;
    let mut duration_ms = 0u64;

    while let Some(event) = rx.recv().await {
        match event {
            crate::llm::GenerationEvent::Token(t) => text.push_str(&t),
            crate::llm::GenerationEvent::Done {
                tokens_generated: t,
                duration_ms: d,
            } => {
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

/// Confidence mode for trust-gated generation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceMode {
    Answer,
    Clarify,
    Abstain,
}

/// Confidence assessment attached to generated output.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfidenceAssessment {
    pub mode: ConfidenceMode,
    pub score: f64,
    pub rationale: String,
}

/// Grounding result for a generated claim.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GroundedClaim {
    pub claim: String,
    pub source_indexes: Vec<usize>,
    pub support_level: String,
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
    /// Confidence-gating decision.
    pub confidence: ConfidenceAssessment,
    /// Per-claim grounding map.
    pub grounding: Vec<GroundedClaim>,
}

/// First-response tone for Slack/Jira
#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FirstResponseTone {
    Slack,
    Jira,
}

/// Parameters for first-response generation
#[derive(serde::Deserialize)]
pub struct FirstResponseParams {
    pub user_input: String,
    pub tone: FirstResponseTone,
    pub ocr_text: Option<String>,
    pub jira_ticket: Option<crate::jira::JiraTicket>,
}

/// First-response generation result
#[derive(serde::Serialize)]
pub struct FirstResponseResult {
    pub text: String,
    pub tokens_generated: u32,
    pub duration_ms: u64,
}

/// Checklist item generated by the LLM
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ChecklistItem {
    pub id: String,
    pub text: String,
    pub category: Option<String>,
    pub priority: Option<String>,
    pub details: Option<String>,
}

/// Checklist state for updates
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ChecklistState {
    pub items: Vec<ChecklistItem>,
    pub completed_ids: Vec<String>,
}

/// Parameters for checklist generation
#[derive(serde::Deserialize)]
pub struct ChecklistGenerateParams {
    pub user_input: String,
    pub ocr_text: Option<String>,
    pub diagnostic_notes: Option<String>,
    pub tree_decisions: Option<crate::prompts::TreeDecisions>,
    pub jira_ticket: Option<crate::jira::JiraTicket>,
}

/// Parameters for checklist update
#[derive(serde::Deserialize)]
pub struct ChecklistUpdateParams {
    pub user_input: String,
    pub ocr_text: Option<String>,
    pub diagnostic_notes: Option<String>,
    pub tree_decisions: Option<crate::prompts::TreeDecisions>,
    pub jira_ticket: Option<crate::jira::JiraTicket>,
    pub checklist: ChecklistState,
}

/// Checklist generation result
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ChecklistResult {
    pub items: Vec<ChecklistItem>,
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
    pub search_method: Option<String>,
    pub source_type: Option<String>,
}

fn extract_json_block(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    let first_brace = trimmed.find('{');
    let first_bracket = trimmed.find('[');

    let (start_idx, open_char, close_char) = match (first_brace, first_bracket) {
        (Some(b), Some(a)) => {
            if b < a {
                (b, '{', '}')
            } else {
                (a, '[', ']')
            }
        }
        (Some(b), None) => (b, '{', '}'),
        (None, Some(a)) => (a, '[', ']'),
        (None, None) => return None,
    };

    let mut depth = 0i32;
    for (idx, ch) in trimmed.char_indices().skip(start_idx) {
        if ch == open_char {
            depth += 1;
        } else if ch == close_char {
            depth -= 1;
            if depth == 0 {
                return Some(&trimmed[start_idx..=idx]);
            }
        }
    }

    None
}

fn normalize_category(value: Option<String>) -> Option<String> {
    let normalized = value?.trim().to_lowercase();
    match normalized.as_str() {
        "triage" | "diagnostic" | "resolution" | "escalation" => Some(normalized),
        _ => None,
    }
}

fn normalize_priority(value: Option<String>) -> Option<String> {
    let normalized = value?.trim().to_lowercase();
    match normalized.as_str() {
        "high" | "medium" | "low" => Some(normalized),
        _ => None,
    }
}

fn normalize_checklist_items(mut items: Vec<ChecklistItem>) -> Vec<ChecklistItem> {
    use std::collections::HashSet;

    const MAX_ITEMS: usize = 10;
    let mut seen_texts = HashSet::new();
    let mut seen_ids = HashSet::new();
    let mut normalized = Vec::new();

    for (idx, item) in items.drain(..).enumerate() {
        let text = item.text.trim();
        if text.is_empty() {
            continue;
        }

        let normalized_text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        let text_key = normalized_text.to_lowercase();
        if !seen_texts.insert(text_key) {
            continue;
        }

        let mut id = item.id.trim().to_string();
        if id.is_empty() {
            id = format!("step-{}", idx + 1);
        }
        if !seen_ids.insert(id.clone()) {
            let mut suffix = 2;
            let mut candidate = format!("{}-{}", id, suffix);
            while !seen_ids.insert(candidate.clone()) {
                suffix += 1;
                candidate = format!("{}-{}", id, suffix);
            }
            id = candidate;
        }

        normalized.push(ChecklistItem {
            id,
            text: normalized_text,
            category: normalize_category(item.category),
            priority: normalize_priority(item.priority),
            details: item
                .details
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty()),
        });

        if normalized.len() >= MAX_ITEMS {
            break;
        }
    }

    normalized
}

fn parse_checklist_output(raw: &str) -> Result<Vec<ChecklistItem>, String> {
    let trimmed = raw.trim();
    let json_candidate = extract_json_block(trimmed).unwrap_or(trimmed);

    if let Ok(wrapper) = serde_json::from_str::<ChecklistResult>(json_candidate) {
        return Ok(normalize_checklist_items(wrapper.items));
    }

    if let Ok(items) = serde_json::from_str::<Vec<ChecklistItem>>(json_candidate) {
        return Ok(normalize_checklist_items(items));
    }

    Err("Checklist response was not valid JSON.".to_string())
}

fn extract_output_section_for_grounding(text: &str) -> String {
    let lower = text.to_lowercase();
    let output_header = "### output";
    let instructions_header = "### it support instructions";
    if let Some(output_idx) = lower.find(output_header) {
        let content_start = output_idx + output_header.len();
        let content_end = lower[content_start..]
            .find(instructions_header)
            .map(|idx| content_start + idx)
            .unwrap_or(text.len());
        text[content_start..content_end].trim().to_string()
    } else {
        text.trim().to_string()
    }
}

fn split_claims(text: &str) -> Vec<String> {
    let mut claims = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '.' | '!' | '?') {
            let claim = current.trim();
            if !claim.is_empty() {
                claims.push(claim.to_string());
            }
            current.clear();
        }
    }

    let tail = current.trim();
    if !tail.is_empty() {
        claims.push(tail.to_string());
    }

    if claims.is_empty() {
        text.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| line.trim_start_matches("- ").to_string())
            .collect()
    } else {
        claims
    }
}

fn parse_source_indexes(claim: &str, source_count: usize) -> Vec<usize> {
    let mut indexes = Vec::new();
    let re = regex_lite::Regex::new(r"(?i)\[source\s+(\d+)\]")
        .expect("source citation regex must compile");

    for caps in re.captures_iter(claim) {
        let parsed = caps
            .get(1)
            .and_then(|m| m.as_str().parse::<usize>().ok())
            .unwrap_or(0);
        if parsed > 0 {
            let index = parsed - 1;
            if index < source_count && !indexes.contains(&index) {
                indexes.push(index);
            }
        }
    }
    indexes
}

fn build_grounding(claims: &[String], sources: &[ContextSource]) -> Vec<GroundedClaim> {
    claims
        .iter()
        .map(|claim| {
            let source_indexes = parse_source_indexes(claim, sources.len());
            let support_level = if source_indexes.is_empty() {
                "unsupported".to_string()
            } else {
                let avg = source_indexes
                    .iter()
                    .map(|i| sources[*i].score)
                    .sum::<f64>()
                    / source_indexes.len() as f64;
                if avg >= 0.75 {
                    "strong".to_string()
                } else if avg >= 0.5 {
                    "moderate".to_string()
                } else {
                    "weak".to_string()
                }
            };

            GroundedClaim {
                claim: claim.clone(),
                source_indexes,
                support_level,
            }
        })
        .collect()
}

fn assess_confidence(
    grounding: &[GroundedClaim],
    sources: &[ContextSource],
) -> ConfidenceAssessment {
    let source_count = sources.len();
    let avg_source_score = if source_count > 0 {
        sources.iter().map(|s| s.score).sum::<f64>() / source_count as f64
    } else {
        0.0
    };

    let total_claims = grounding.len();
    let unsupported_claims = grounding
        .iter()
        .filter(|c| c.support_level == "unsupported")
        .count();
    let coverage = if total_claims > 0 {
        1.0 - (unsupported_claims as f64 / total_claims as f64)
    } else {
        0.0
    };
    let score = ((avg_source_score * 0.6) + (coverage * 0.4)).clamp(0.0, 1.0);

    let (mode, rationale) = if source_count == 0 || score < 0.45 {
        (
            ConfidenceMode::Abstain,
            "Low retrieval confidence or no grounded evidence".to_string(),
        )
    } else if score < 0.7 || unsupported_claims > 0 {
        (
            ConfidenceMode::Clarify,
            "Some claims are weakly grounded; clarify before sending".to_string(),
        )
    } else {
        (
            ConfidenceMode::Answer,
            "Strong grounded evidence across cited sources".to_string(),
        )
    };

    ConfidenceAssessment {
        mode,
        score,
        rationale,
    }
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
            crate::kb::search::HybridSearch::search(db, query, limit).unwrap_or_default()
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
            search_method: Some(format!("{:?}", r.source)),
            source_type: r.source_type.clone(),
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
    let gen_result = generate_text(state.clone(), prompt, params.gen_params).await?;

    // Calculate quality metrics
    let word_count = gen_result.text.split_whitespace().count() as u32;
    let target_words = response_length.target_words() as u32;
    let length_target_met = match response_length {
        crate::prompts::ResponseLength::Short => word_count <= target_words + 40,
        crate::prompts::ResponseLength::Medium => {
            word_count >= target_words / 2 && word_count <= target_words * 2
        }
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
    let context_utilization =
        (estimated_prompt_tokens as f64 / context_window as f64 * 100.0).min(100.0);

    let metrics = GenerationMetrics {
        tokens_per_second,
        sources_used: sources.len() as u32,
        word_count,
        length_target_met,
        context_utilization,
    };

    let output_section = extract_output_section_for_grounding(&gen_result.text);
    let claims = split_claims(&output_section);
    let grounding = build_grounding(&claims, &sources);
    let confidence = assess_confidence(&grounding, &sources);

    let confidence_mode = match confidence.mode {
        ConfidenceMode::Answer => "answer",
        ConfidenceMode::Clarify => "clarify",
        ConfidenceMode::Abstain => "abstain",
    };
    let unsupported_claims = grounding
        .iter()
        .filter(|claim| claim.support_level == "unsupported")
        .count() as i32;
    let avg_source_score = if sources.is_empty() {
        0.0
    } else {
        sources.iter().map(|s| s.score).sum::<f64>() / sources.len() as f64
    };
    if let Ok(db_lock) = state.db.lock() {
        if let Some(db) = db_lock.as_ref() {
            let _ = db.record_generation_quality_event(GenerationQualityEvent {
                query_text: &params.user_input,
                confidence_mode,
                confidence_score: confidence.score,
                unsupported_claims,
                total_claims: grounding.len() as i32,
                source_count: sources.len() as i32,
                avg_source_score,
            });
        }
    }

    Ok(GenerateWithContextResult {
        text: gen_result.text,
        tokens_generated: gen_result.tokens_generated,
        duration_ms: gen_result.duration_ms,
        source_chunk_ids,
        sources,
        metrics,
        prompt_template_version: crate::prompts::PROMPT_TEMPLATE_VERSION.to_string(),
        confidence,
        grounding,
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
            crate::kb::search::HybridSearch::search(db, query, limit).unwrap_or_default()
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
            search_method: Some(format!("{:?}", r.source)),
            source_type: r.source_type.clone(),
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
        let temp_engine = LlmEngine {
            state: engine_state,
        };
        let _ = temp_engine
            .generate_streaming(&prompt_clone, gen_params, tx_clone, cancel_clone)
            .await;
    });

    // Forward tokens to frontend as events and collect output
    let mut text = String::new();
    let mut tokens_generated = 0u32;
    let mut duration_ms = 0u64;

    while let Some(event) = rx.recv().await {
        match event {
            crate::llm::GenerationEvent::Token(t) => {
                // Emit token to frontend
                let _ = window.emit(
                    "llm-token",
                    StreamToken {
                        token: t.clone(),
                        done: false,
                    },
                );
                text.push_str(&t);
            }
            crate::llm::GenerationEvent::Done {
                tokens_generated: t,
                duration_ms: d,
            } => {
                tokens_generated = t;
                duration_ms = d;
                // Emit done signal
                let _ = window.emit(
                    "llm-token",
                    StreamToken {
                        token: String::new(),
                        done: true,
                    },
                );
                break;
            }
            crate::llm::GenerationEvent::Error(e) => {
                let _ = window.emit(
                    "llm-token",
                    StreamToken {
                        token: String::new(),
                        done: true,
                    },
                );
                return Err(e);
            }
        }
    }

    // Calculate quality metrics
    let word_count = text.split_whitespace().count() as u32;
    let target_words = response_length.target_words() as u32;
    let length_target_met = match response_length {
        crate::prompts::ResponseLength::Short => word_count <= target_words + 40,
        crate::prompts::ResponseLength::Medium => {
            word_count >= target_words / 2 && word_count <= target_words * 2
        }
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
    let context_utilization =
        (estimated_prompt_tokens as f64 / context_window as f64 * 100.0).min(100.0);

    let metrics = GenerationMetrics {
        tokens_per_second,
        sources_used: sources.len() as u32,
        word_count,
        length_target_met,
        context_utilization,
    };

    let output_section = extract_output_section_for_grounding(&text);
    let claims = split_claims(&output_section);
    let grounding = build_grounding(&claims, &sources);
    let confidence = assess_confidence(&grounding, &sources);

    let confidence_mode = match confidence.mode {
        ConfidenceMode::Answer => "answer",
        ConfidenceMode::Clarify => "clarify",
        ConfidenceMode::Abstain => "abstain",
    };
    let unsupported_claims = grounding
        .iter()
        .filter(|claim| claim.support_level == "unsupported")
        .count() as i32;
    let avg_source_score = if sources.is_empty() {
        0.0
    } else {
        sources.iter().map(|s| s.score).sum::<f64>() / sources.len() as f64
    };
    if let Ok(db_lock) = state.db.lock() {
        if let Some(db) = db_lock.as_ref() {
            let _ = db.record_generation_quality_event(GenerationQualityEvent {
                query_text: &params.user_input,
                confidence_mode,
                confidence_score: confidence.score,
                unsupported_claims,
                total_claims: grounding.len() as i32,
                source_count: sources.len() as i32,
                avg_source_score,
            });
        }
    }

    Ok(GenerateWithContextResult {
        text,
        tokens_generated,
        duration_ms,
        source_chunk_ids,
        sources,
        metrics,
        prompt_template_version: crate::prompts::PROMPT_TEMPLATE_VERSION.to_string(),
        confidence,
        grounding,
    })
}

/// Generate a short first-response message for Slack or Jira
#[tauri::command]
pub async fn generate_first_response(
    state: State<'_, AppState>,
    params: FirstResponseParams,
) -> Result<FirstResponseResult, String> {
    use crate::prompts::{
        PromptBuilder, ResponseLength, FIRST_RESPONSE_JIRA_PROMPT, FIRST_RESPONSE_SLACK_PROMPT,
    };

    validate_non_empty(&params.user_input).map_err(|e| e.to_string())?;
    validate_text_size(&params.user_input, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;
    if let Some(ocr) = &params.ocr_text {
        validate_text_size(ocr, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;
    }

    let system_prompt = match params.tone {
        FirstResponseTone::Slack => FIRST_RESPONSE_SLACK_PROMPT,
        FirstResponseTone::Jira => FIRST_RESPONSE_JIRA_PROMPT,
    };

    let mut builder = PromptBuilder::new()
        .with_system_prompt(system_prompt)
        .with_user_input(&params.user_input)
        .with_response_length(ResponseLength::Short);

    if let Some(ocr) = &params.ocr_text {
        builder = builder.with_ocr_text(ocr);
    }

    if let Some(ticket) = params.jira_ticket {
        builder = builder.with_jira_ticket(ticket);
    }

    let prompt = builder.build();
    validate_text_size(&prompt, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;

    let gen_result = generate_text(
        state,
        prompt,
        Some(GenerateParams {
            max_tokens: Some(200),
            temperature: Some(0.4),
            top_p: Some(0.9),
            top_k: None,
            repeat_penalty: Some(1.05),
            context_window: None,
        }),
    )
    .await?;

    Ok(FirstResponseResult {
        text: gen_result.text.trim().to_string(),
        tokens_generated: gen_result.tokens_generated,
        duration_ms: gen_result.duration_ms,
    })
}

/// Generate a troubleshooting checklist for the issue
#[tauri::command]
pub async fn generate_troubleshooting_checklist(
    state: State<'_, AppState>,
    params: ChecklistGenerateParams,
) -> Result<ChecklistResult, String> {
    use crate::prompts::{PromptBuilder, ResponseLength, CHECKLIST_SYSTEM_PROMPT};

    validate_non_empty(&params.user_input).map_err(|e| e.to_string())?;
    validate_text_size(&params.user_input, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;
    if let Some(ocr) = &params.ocr_text {
        validate_text_size(ocr, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;
    }
    if let Some(notes) = &params.diagnostic_notes {
        validate_text_size(notes, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;
    }

    let mut builder = PromptBuilder::new()
        .with_system_prompt(CHECKLIST_SYSTEM_PROMPT)
        .with_user_input(&params.user_input)
        .with_response_length(ResponseLength::Long);

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

    let prompt = builder.build();
    validate_text_size(&prompt, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;

    let gen_result = generate_text(
        state,
        prompt,
        Some(GenerateParams {
            max_tokens: Some(400),
            temperature: Some(0.2),
            top_p: Some(0.9),
            top_k: None,
            repeat_penalty: Some(1.05),
            context_window: None,
        }),
    )
    .await?;

    let items = parse_checklist_output(&gen_result.text)?;
    Ok(ChecklistResult { items })
}

/// Update an existing troubleshooting checklist based on completed steps
#[tauri::command]
pub async fn update_troubleshooting_checklist(
    state: State<'_, AppState>,
    params: ChecklistUpdateParams,
) -> Result<ChecklistResult, String> {
    use crate::prompts::{PromptBuilder, ResponseLength, CHECKLIST_UPDATE_SYSTEM_PROMPT};
    use std::collections::HashSet;

    validate_non_empty(&params.user_input).map_err(|e| e.to_string())?;
    validate_text_size(&params.user_input, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;
    if let Some(ocr) = &params.ocr_text {
        validate_text_size(ocr, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;
    }
    if let Some(notes) = &params.diagnostic_notes {
        validate_text_size(notes, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;
    }

    let items = normalize_checklist_items(params.checklist.items);
    let valid_ids: HashSet<&str> = items.iter().map(|item| item.id.as_str()).collect();
    let completed_ids: Vec<String> = params
        .checklist
        .completed_ids
        .into_iter()
        .filter(|id| valid_ids.contains(id.as_str()))
        .collect();

    let checklist_state = ChecklistState {
        items,
        completed_ids,
    };

    let checklist_json = serde_json::to_string_pretty(&checklist_state)
        .or_else(|_| serde_json::to_string(&checklist_state))
        .map_err(|e| e.to_string())?;

    let completed_label = if checklist_state.completed_ids.is_empty() {
        "none".to_string()
    } else {
        checklist_state.completed_ids.join(", ")
    };

    let update_context = format!(
        "Current checklist JSON:\n{}\n\nCompleted item IDs: {}",
        checklist_json, completed_label
    );

    let mut builder = PromptBuilder::new()
        .with_system_prompt(CHECKLIST_UPDATE_SYSTEM_PROMPT)
        .with_user_input(&params.user_input)
        .with_response_length(ResponseLength::Long)
        .with_extra_section("Checklist Update Context", &update_context);

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

    let prompt = builder.build();
    validate_text_size(&prompt, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;

    let gen_result = generate_text(
        state,
        prompt,
        Some(GenerateParams {
            max_tokens: Some(400),
            temperature: Some(0.2),
            top_p: Some(0.9),
            top_k: None,
            repeat_penalty: Some(1.05),
            context_window: None,
        }),
    )
    .await?;

    let items = parse_checklist_output(&gen_result.text)?;
    Ok(ChecklistResult { items })
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
    )
    .await?;

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
