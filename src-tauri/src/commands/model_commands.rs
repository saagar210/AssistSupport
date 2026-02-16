use super::*;

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
