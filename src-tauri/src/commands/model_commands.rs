use super::*;
use crate::downloads::{recommended_models, DownloadManager, ModelSource};
use crate::kb::embeddings::{EmbeddingEngine, EmbeddingModelInfo};
use crate::model_integrity::{verify_model_integrity, ModelAllowlist, VerificationResult};

const CONTEXT_WINDOW_SETTING: &str = "llm_context_window";

fn get_model_source(model_id: &str) -> Result<(&'static str, &'static str), String> {
    match model_id {
        "llama-3.1-8b-instruct" => Ok((
            "bartowski/Meta-Llama-3.1-8B-Instruct-GGUF",
            "Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf",
        )),
        "llama-3.2-1b-instruct" => Ok((
            "bartowski/Llama-3.2-1B-Instruct-GGUF",
            "Llama-3.2-1B-Instruct-Q4_K_M.gguf",
        )),
        "llama-3.2-3b-instruct" => Ok((
            "bartowski/Llama-3.2-3B-Instruct-GGUF",
            "Llama-3.2-3B-Instruct-Q4_K_M.gguf",
        )),
        "phi-3-mini-4k-instruct" => Ok((
            "bartowski/Phi-3.1-mini-4k-instruct-GGUF",
            "Phi-3.1-mini-4k-instruct-Q4_K_M.gguf",
        )),
        "nomic-embed-text" => Ok((
            "nomic-ai/nomic-embed-text-v1.5-GGUF",
            "nomic-embed-text-v1.5.Q5_K_M.gguf",
        )),
        _ => Err(format!("Unknown model ID: {}", model_id)),
    }
}

fn get_embedding_model_filename(model_id: &str) -> Option<&'static str> {
    match model_id {
        "nomic-embed-text" => Some("nomic-embed-text-v1.5.Q5_K_M.gguf"),
        _ => None,
    }
}

pub(crate) fn init_llm_engine_impl(state: State<'_, AppState>) -> Result<(), String> {
    if state.llm.read().is_some() {
        return Ok(());
    }
    let engine = LlmEngine::new(state.backend.clone()).map_err(|e| e.to_string())?;
    *state.llm.write() = Some(engine);
    Ok(())
}

pub(crate) fn load_model_impl(
    state: State<'_, AppState>,
    model_id: String,
    n_gpu_layers: Option<u32>,
) -> Result<ModelInfo, String> {
    let load_start = std::time::Instant::now();

    let (_, filename) = get_model_source(&model_id)?;
    let models_dir = crate::db::get_models_dir();
    let path = models_dir.join(filename);

    if !path.exists() {
        return Err(format!(
            "Model file not found: {}. Please download the model first.",
            filename
        ));
    }

    let llm_guard = state.llm.read();
    let engine = llm_guard.as_ref().ok_or("LLM engine not initialized")?;

    let layers = n_gpu_layers.unwrap_or(1000);
    let info = engine
        .load_model(&path, layers, model_id.clone())
        .map_err(|e| e.to_string())?;

    let load_time_ms = load_start.elapsed().as_millis() as i64;
    if let Ok(db_lock) = state.db.lock() {
        if let Some(db) = db_lock.as_ref() {
            let _ = db.save_model_state(
                "llm",
                path.to_str().unwrap_or(""),
                Some(&model_id),
                Some(load_time_ms),
            );
        }
    }
    tracing::info!("LLM model '{}' loaded in {}ms", model_id, load_time_ms);

    Ok(info)
}

pub(crate) fn load_custom_model_impl(
    state: State<'_, AppState>,
    model_path: String,
    n_gpu_layers: Option<u32>,
) -> Result<ModelInfo, String> {
    use std::path::Path;

    let path = Path::new(&model_path);
    if !path.exists() {
        return Err(format!("Model file not found: {}", model_path));
    }

    let validated_path = validate_within_home(path).map_err(|e| match e {
        ValidationError::PathTraversal => {
            "Model file must be within your home directory".to_string()
        }
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This path is blocked because it contains sensitive data".to_string()
        }
        _ => format!("Invalid model path: {}", e),
    })?;

    if !validated_path.is_file() {
        return Err("Model path is not a file".into());
    }

    let ext = validated_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    if ext.to_lowercase() != "gguf" {
        return Err("Invalid file type. Only .gguf files are supported.".into());
    }

    let metadata = std::fs::metadata(&validated_path).map_err(|e| e.to_string())?;
    if metadata.len() < 1_000_000 {
        return Err("File too small to be a valid GGUF model.".into());
    }

    let model_id = validated_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("custom-model")
        .to_string();

    match verify_model_integrity(&validated_path, false) {
        Ok(VerificationResult::Verified { sha256, .. }) => {
            audit::audit_model_integrity_verified(&model_id, &sha256);
        }
        Ok(VerificationResult::Unverified { sha256, .. }) => {
            audit::audit_model_integrity_unverified(&model_id, &sha256);
            tracing::warn!(
                "Loading unverified model '{}' (sha256: {}). Prefer allowlisted models.",
                model_id,
                sha256
            );
        }
        Err(e) => return Err(format!("Model integrity verification failed: {}", e)),
    }

    let llm_guard = state.llm.read();
    let engine = llm_guard.as_ref().ok_or("LLM engine not initialized")?;

    let layers = n_gpu_layers.unwrap_or(1000);
    engine
        .load_model(&validated_path, layers, model_id)
        .map_err(|e| e.to_string())
}

pub(crate) fn validate_gguf_file_impl(model_path: String) -> Result<GgufFileInfo, String> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(&model_path);
    if !path.exists() {
        return Err(format!("File not found: {}", model_path));
    }

    let validated_path = validate_within_home(path).map_err(|e| match e {
        ValidationError::PathTraversal => {
            "Model file must be within your home directory".to_string()
        }
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This path is blocked because it contains sensitive data".to_string()
        }
        _ => format!("Invalid model path: {}", e),
    })?;

    if !validated_path.is_file() {
        return Err("Model path is not a file".into());
    }

    let ext = validated_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    if ext.to_lowercase() != "gguf" {
        return Err("Invalid file type. Only .gguf files are supported.".into());
    }

    let metadata = fs::metadata(&validated_path).map_err(|e| e.to_string())?;
    let filename = validated_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut file = fs::File::open(&validated_path).map_err(|e| e.to_string())?;
    let mut magic = [0u8; 4];
    use std::io::Read;
    file.read_exact(&mut magic).map_err(|e| e.to_string())?;

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

pub(crate) fn unload_model_impl(state: State<'_, AppState>) -> Result<(), String> {
    let llm_guard = state.llm.read();
    let engine = llm_guard.as_ref().ok_or("LLM engine not initialized")?;
    engine.unload_model();

    if let Ok(db_lock) = state.db.lock() {
        if let Some(db) = db_lock.as_ref() {
            let _ = db.clear_model_state("llm");
        }
    }
    Ok(())
}

pub(crate) fn get_model_info_impl(state: State<'_, AppState>) -> Result<Option<ModelInfo>, String> {
    let llm_guard = state.llm.read();
    let engine = llm_guard.as_ref().ok_or("LLM engine not initialized")?;
    Ok(engine.model_info())
}

pub(crate) fn is_model_loaded_impl(state: State<'_, AppState>) -> Result<bool, String> {
    let llm_guard = state.llm.read();
    match llm_guard.as_ref() {
        Some(engine) => Ok(engine.is_model_loaded()),
        None => Ok(false),
    }
}

pub(crate) fn get_context_window_impl(state: State<'_, AppState>) -> Result<Option<u32>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let result: Result<String, _> = db.conn().query_row(
        "SELECT value FROM settings WHERE key = ?",
        rusqlite::params![CONTEXT_WINDOW_SETTING],
        |row| row.get(0),
    );

    match result {
        Ok(value) => Ok(value.parse::<u32>().ok()),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub(crate) fn set_context_window_impl(
    state: State<'_, AppState>,
    size: Option<u32>,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    match size {
        Some(s) => {
            if !(2048..=32768).contains(&s) {
                return Err("Context window must be between 2048 and 32768".to_string());
            }
            db.conn()
                .execute(
                    "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
                    rusqlite::params![CONTEXT_WINDOW_SETTING, s.to_string()],
                )
                .map_err(|e| e.to_string())?;
        }
        None => {
            db.conn()
                .execute(
                    "DELETE FROM settings WHERE key = ?",
                    rusqlite::params![CONTEXT_WINDOW_SETTING],
                )
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

pub(crate) fn get_model_state_impl(state: State<'_, AppState>) -> Result<ModelStateResult, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let llm = db.get_model_state("llm").map_err(|e| e.to_string())?;
    let embeddings = db
        .get_model_state("embeddings")
        .map_err(|e| e.to_string())?;

    // Check if models are currently loaded in memory
    let llm_loaded = state
        .llm
        .read()
        .as_ref()
        .map(|e| e.model_info().is_some())
        .unwrap_or(false);

    let embeddings_loaded = state
        .embeddings
        .read()
        .as_ref()
        .map(|e| e.model_info().is_some())
        .unwrap_or(false);

    Ok(ModelStateResult {
        llm_model_id: llm.as_ref().and_then(|(_, id)| id.clone()),
        llm_model_path: llm.map(|(p, _)| p),
        llm_loaded,
        embeddings_model_path: embeddings.map(|(p, _)| p),
        embeddings_loaded,
    })
}

pub(crate) fn get_startup_metrics_impl(
    state: State<'_, AppState>,
) -> Result<StartupMetricsResult, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let metrics = db.get_last_startup_metric().map_err(|e| e.to_string())?;

    match metrics {
        Some((total_ms, init_app_ms, models_cached)) => Ok(StartupMetricsResult {
            total_ms,
            init_app_ms,
            models_cached,
        }),
        None => Ok(StartupMetricsResult {
            total_ms: 0,
            init_app_ms: 0,
            models_cached: false,
        }),
    }
}

pub(crate) fn get_recommended_models_impl() -> Vec<ModelSource> {
    recommended_models()
}

pub(crate) fn list_downloaded_models_impl() -> Result<Vec<String>, String> {
    let app_dir = get_app_data_dir();
    let manager = DownloadManager::new(&app_dir);

    let models = manager.list_models().map_err(|e| e.to_string())?;

    let model_ids: Vec<String> = models
        .into_iter()
        .filter_map(|p| {
            let filename = p.file_name()?.to_str()?;
            match filename {
                "Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf" => {
                    Some("llama-3.1-8b-instruct".to_string())
                }
                "Llama-3.2-1B-Instruct-Q4_K_M.gguf" => Some("llama-3.2-1b-instruct".to_string()),
                "Llama-3.2-3B-Instruct-Q4_K_M.gguf" => Some("llama-3.2-3b-instruct".to_string()),
                "Phi-3.1-mini-4k-instruct-Q4_K_M.gguf" => {
                    Some("phi-3-mini-4k-instruct".to_string())
                }
                _ => None,
            }
        })
        .collect();

    Ok(model_ids)
}

pub(crate) fn get_embedding_model_path_impl(model_id: String) -> Result<Option<String>, String> {
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

pub(crate) fn is_embedding_model_downloaded_impl() -> Result<bool, String> {
    let app_dir = get_app_data_dir();
    let model_path = app_dir
        .join("models")
        .join("nomic-embed-text-v1.5.Q5_K_M.gguf");
    Ok(model_path.exists())
}

pub(crate) fn get_models_dir_impl() -> Result<String, String> {
    let app_dir = get_app_data_dir();
    let manager = DownloadManager::new(&app_dir);
    Ok(manager.models_dir().display().to_string())
}

pub(crate) fn delete_downloaded_model_impl(filename: String) -> Result<(), String> {
    use std::path::Component;
    use std::path::Path;

    let path = Path::new(&filename);
    let mut components = path.components();
    let is_single_filename =
        matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none();
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

pub(crate) async fn download_model_impl(
    window: tauri::Window,
    model_id: String,
) -> Result<String, String> {
    use tauri::Emitter;

    let (repo, filename) = get_model_source(&model_id)?;
    audit::audit_model_download_started(&model_id, repo, filename);

    let app_dir = get_app_data_dir();
    let manager = DownloadManager::new(&app_dir);
    manager.init().map_err(|e| e.to_string())?;

    let mut source = ModelSource::huggingface(repo, filename);
    let (size, sha256) = crate::downloads::fetch_hf_file_info(repo, filename)
        .await
        .map_err(|e| {
            audit::audit_model_download_failed(&model_id, "metadata_fetch_failed", &e.to_string());
            format!("Failed to fetch checksum metadata: {}", e)
        })?;
    let allowlist = ModelAllowlist::new();
    let allowed = allowlist.get_allowed_model(filename).ok_or_else(|| {
        audit::audit_model_download_failed(&model_id, "allowlist_missing", filename);
        "Model is not in the allowlist".to_string()
    })?;

    if allowed.repo != repo {
        audit::audit_model_download_failed(&model_id, "allowlist_repo_mismatch", repo);
        return Err("Model allowlist mismatch (repo)".to_string());
    }

    if allowed.size_bytes != size || allowed.sha256.to_lowercase() != sha256.to_lowercase() {
        audit::audit_model_download_failed(&model_id, "allowlist_metadata_mismatch", filename);
        return Err("Model allowlist mismatch (size or checksum)".to_string());
    }

    source.size_bytes = Some(allowed.size_bytes);
    source.sha256 = Some(allowed.sha256.clone());

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    DOWNLOAD_CANCEL_FLAG.store(false, Ordering::SeqCst);
    let cancel_flag = DOWNLOAD_CANCEL_FLAG.clone();

    let download_handle = {
        let cancel = cancel_flag.clone();
        tokio::spawn(async move { manager.download(&source, tx, cancel).await })
    };

    let window_clone = window.clone();
    let event_handle = tokio::spawn(async move {
        while let Some(progress) = rx.recv().await {
            let _ = window_clone.emit("download-progress", &progress);
        }
    });

    let download_result = download_handle.await.map_err(|e| {
        audit::audit_model_download_failed(&model_id, "download_task_failed", &e.to_string());
        e.to_string()
    })?;

    let _ = event_handle.await;

    let result = download_result.map_err(|e| {
        audit::audit_model_download_failed(&model_id, "download_failed", &e.to_string());
        e.to_string()
    })?;

    let verify_path = result.clone();
    let verify_result =
        tokio::task::spawn_blocking(move || verify_model_integrity(&verify_path, true))
            .await
            .map_err(|e| {
                audit::audit_model_download_failed(
                    &model_id,
                    "integrity_task_failed",
                    &e.to_string(),
                );
                e.to_string()
            })?;

    match verify_result {
        Ok(verification) => {
            if verification.is_verified() {
                audit::audit_model_integrity_verified(&model_id, verification.sha256());
            } else {
                audit::audit_model_integrity_unverified(&model_id, verification.sha256());
            }
        }
        Err(e) => {
            audit::audit_model_download_failed(&model_id, "integrity_check_failed", &e.to_string());
            return Err(format!("Model integrity verification failed: {}", e));
        }
    }

    audit::audit_model_download_completed(&model_id, &sha256, size);
    Ok(result.display().to_string())
}

pub(crate) fn cancel_download_impl() -> Result<(), String> {
    DOWNLOAD_CANCEL_FLAG.store(true, Ordering::SeqCst);
    Ok(())
}

pub(crate) fn init_embedding_engine_impl(state: State<'_, AppState>) -> Result<(), String> {
    if state.embeddings.read().is_some() {
        return Ok(());
    }
    let engine = EmbeddingEngine::new(state.backend.clone()).map_err(|e| e.to_string())?;
    *state.embeddings.write() = Some(engine);
    Ok(())
}

pub(crate) fn load_embedding_model_impl(
    state: State<'_, AppState>,
    path: String,
    n_gpu_layers: Option<u32>,
) -> Result<EmbeddingModelInfo, String> {
    use std::path::Path;

    let emb_guard = state.embeddings.read();
    let engine = emb_guard
        .as_ref()
        .ok_or("Embedding engine not initialized")?;

    let path = Path::new(&path);
    if !path.exists() {
        return Err(format!(
            "Embedding model file not found: {}",
            path.display()
        ));
    }

    let validated_path = validate_within_home(path).map_err(|e| match e {
        ValidationError::PathTraversal => {
            "Embedding model file must be within your home directory".to_string()
        }
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This path is blocked because it contains sensitive data".to_string()
        }
        _ => format!("Invalid embedding model path: {}", e),
    })?;

    if !validated_path.is_file() {
        return Err("Embedding model path is not a file".into());
    }

    let load_start = std::time::Instant::now();
    let layers = n_gpu_layers.unwrap_or(1000);

    let info = engine
        .load_model(&validated_path, layers)
        .map_err(|e| e.to_string())?;

    let load_time_ms = load_start.elapsed().as_millis() as i64;
    if let Ok(db_lock) = state.db.lock() {
        if let Some(db) = db_lock.as_ref() {
            let _ = db.save_model_state(
                "embeddings",
                validated_path.to_str().unwrap_or(""),
                None,
                Some(load_time_ms),
            );
        }
    }
    tracing::info!("Embedding model loaded in {}ms", load_time_ms);

    Ok(info)
}

pub(crate) fn unload_embedding_model_impl(state: State<'_, AppState>) -> Result<(), String> {
    let emb_guard = state.embeddings.read();
    let engine = emb_guard
        .as_ref()
        .ok_or("Embedding engine not initialized")?;
    engine.unload_model();
    if let Ok(db_lock) = state.db.lock() {
        if let Some(db) = db_lock.as_ref() {
            let _ = db.clear_model_state("embeddings");
        }
    }
    Ok(())
}

pub(crate) fn get_embedding_model_info_impl(
    state: State<'_, AppState>,
) -> Result<Option<EmbeddingModelInfo>, String> {
    let emb_guard = state.embeddings.read();
    let engine = emb_guard
        .as_ref()
        .ok_or("Embedding engine not initialized")?;
    Ok(engine.model_info())
}

pub(crate) fn is_embedding_model_loaded_impl(state: State<'_, AppState>) -> Result<bool, String> {
    let emb_guard = state.embeddings.read();
    match emb_guard.as_ref() {
        Some(engine) => Ok(engine.is_model_loaded()),
        None => Ok(false),
    }
}
