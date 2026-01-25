//! Diagnostics and health check commands

use crate::diagnostics::{
    check_database_health, check_embedding_health, check_filesystem_health, check_llm_health,
    get_database_stats, get_failure_modes, get_resource_metrics, get_vector_maintenance_info,
    repair_database, run_database_maintenance, ComponentHealth, DatabaseStats, FailureMode,
    HealthStatus, LlmResourceLimits, RepairResult, ResourceMetrics, SystemHealth,
    VectorMaintenanceInfo,
};
use crate::AppState;
use tauri::State;

/// Get comprehensive system health status
#[tauri::command]
pub async fn get_system_health(state: State<'_, AppState>) -> Result<SystemHealth, String> {
    // Check database
    let database = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        match db_lock.as_ref() {
            Some(db) => check_database_health(db),
            None => ComponentHealth::unavailable("Database", "Not initialized"),
        }
    };

    // Check vector store
    let vector_store = {
        let vectors = state.vectors.read().await;
        crate::diagnostics::check_vector_store_health(vectors.as_ref()).await
    };

    // Check LLM engine
    let llm_engine = {
        let llm = state.llm.read();
        check_llm_health(llm.as_ref())
    };

    // Check embedding model
    let embedding_model = {
        let embeddings = state.embeddings.read();
        check_embedding_health(embeddings.as_ref())
    };

    // Check file system
    let file_system = check_filesystem_health();

    // Calculate overall status
    let overall_status = database
        .status
        .worst(vector_store.status)
        .worst(llm_engine.status)
        .worst(embedding_model.status)
        .worst(file_system.status);

    Ok(SystemHealth {
        database,
        vector_store,
        llm_engine,
        embedding_model,
        file_system,
        overall_status,
        checked_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Attempt to repair the database
#[tauri::command]
pub fn repair_database_cmd(state: State<'_, AppState>) -> Result<RepairResult, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    Ok(repair_database(db))
}

/// Get guidance on rebuilding vector store
#[tauri::command]
pub fn rebuild_vector_store() -> RepairResult {
    crate::diagnostics::get_vector_rebuild_guidance()
}

/// Get list of known failure modes and their solutions
#[tauri::command]
pub fn get_failure_modes_cmd() -> Vec<FailureMode> {
    get_failure_modes()
}

/// Run a quick connectivity/health test
#[tauri::command]
pub async fn run_quick_health_check(
    state: State<'_, AppState>,
) -> Result<QuickHealthResult, String> {
    let mut checks_passed = 0;
    let mut checks_total = 0;
    let mut issues: Vec<String> = Vec::new();

    // Check 1: Database accessible
    checks_total += 1;
    {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        if let Some(db) = db_lock.as_ref() {
            if db.check_integrity().is_ok() {
                checks_passed += 1;
            } else {
                issues.push("Database integrity check failed".to_string());
            }
        } else {
            issues.push("Database not initialized".to_string());
        }
    }

    // Check 2: Can query KB
    checks_total += 1;
    {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        if let Some(db) = db_lock.as_ref() {
            if db
                .conn()
                .query_row::<i64, _, _>("SELECT COUNT(*) FROM kb_documents", [], |r| r.get(0))
                .is_ok()
            {
                checks_passed += 1;
            } else {
                issues.push("Cannot query knowledge base".to_string());
            }
        }
    }

    // Check 3: File system writable
    checks_total += 1;
    let fs_health = check_filesystem_health();
    if fs_health.status == HealthStatus::Healthy {
        checks_passed += 1;
    } else {
        issues.push(format!("File system: {}", fs_health.message));
    }

    // Check 4: Model loaded (optional, warning only)
    checks_total += 1;
    {
        let llm = state.llm.read();
        if let Some(engine) = llm.as_ref() {
            if engine.is_model_loaded() {
                checks_passed += 1;
            } else {
                issues.push("No LLM model loaded".to_string());
            }
        } else {
            issues.push("LLM engine not initialized".to_string());
        }
    }

    Ok(QuickHealthResult {
        healthy: issues.is_empty() || (checks_passed >= 3), // Allow model to be unloaded
        checks_passed,
        checks_total,
        issues,
    })
}

/// Result of a quick health check
#[derive(Debug, Clone, serde::Serialize)]
pub struct QuickHealthResult {
    pub healthy: bool,
    pub checks_passed: u32,
    pub checks_total: u32,
    pub issues: Vec<String>,
}

/// Get database statistics for monitoring
#[tauri::command]
pub fn get_database_stats_cmd(state: State<'_, AppState>) -> Result<DatabaseStats, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let db_path = dirs::data_dir()
        .map(|d| d.join("AssistSupport/assistsupport.db"))
        .ok_or("Could not determine database path")?;

    get_database_stats(db, &db_path)
}

/// Run scheduled database maintenance (VACUUM if needed)
#[tauri::command]
pub fn run_database_maintenance_cmd(state: State<'_, AppState>) -> Result<RepairResult, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    Ok(run_database_maintenance(db))
}

/// Get current resource usage metrics
#[tauri::command]
pub fn get_resource_metrics_cmd() -> ResourceMetrics {
    get_resource_metrics()
}

/// Get LLM resource limits configuration
#[tauri::command]
pub fn get_llm_resource_limits(state: State<'_, AppState>) -> Result<LlmResourceLimits, String> {
    // Try to load from settings, otherwise return defaults
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(db) = db_lock.as_ref() {
        if let Ok(json) = db.conn().query_row::<String, _, _>(
            "SELECT value FROM settings WHERE key = 'llm_resource_limits'",
            [],
            |r| r.get(0),
        ) {
            if let Ok(limits) = serde_json::from_str(&json) {
                return Ok(limits);
            }
        }
    }
    Ok(LlmResourceLimits::default())
}

/// Set LLM resource limits configuration
#[tauri::command]
pub fn set_llm_resource_limits(
    state: State<'_, AppState>,
    limits: LlmResourceLimits,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let json = serde_json::to_string(&limits).map_err(|e| e.to_string())?;
    db.conn()
        .execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('llm_resource_limits', ?)",
            [&json],
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get vector store maintenance info
#[tauri::command]
pub async fn get_vector_maintenance_info_cmd(
    state: State<'_, AppState>,
) -> Result<Option<VectorMaintenanceInfo>, String> {
    let vectors = state.vectors.read().await;
    Ok(get_vector_maintenance_info(vectors.as_ref()).await)
}
