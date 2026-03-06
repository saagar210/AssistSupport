use super::*;

// Pilot logging is a quality/UX feature that can persist user-entered text.
// For compliance defaults, we keep it *off unless explicitly enabled*.
const PILOT_LOGGING_POLICY_ENV: &str = "ASSISTSUPPORT_ENABLE_PILOT_LOGGING";
const PILOT_RETENTION_DAYS_ENV: &str = "ASSISTSUPPORT_PILOT_RETENTION_DAYS";
const PILOT_MAX_ROWS_ENV: &str = "ASSISTSUPPORT_PILOT_MAX_ROWS";

fn parse_bool_env(var: &str, default: bool) -> bool {
    std::env::var(var)
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .map(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(default)
}

fn parse_i64_env(var: &str, default: i64) -> i64 {
    std::env::var(var)
        .ok()
        .and_then(|v| v.trim().parse::<i64>().ok())
        .unwrap_or(default)
}

fn pilot_logging_enabled() -> bool {
    parse_bool_env(PILOT_LOGGING_POLICY_ENV, false)
}

fn require_pilot_logging_enabled() -> Result<(), String> {
    if pilot_logging_enabled() {
        Ok(())
    } else {
        Err(format!(
            "Pilot logging is disabled by policy. Set {}=1 and restart AssistSupport to enable.",
            PILOT_LOGGING_POLICY_ENV
        ))
    }
}

#[derive(serde::Serialize, Clone)]
pub struct PilotLoggingPolicy {
    pub enabled: bool,
    pub retention_days: i64,
    pub max_rows: i64,
}

#[tauri::command]
pub fn get_pilot_logging_policy() -> PilotLoggingPolicy {
    PilotLoggingPolicy {
        enabled: pilot_logging_enabled(),
        retention_days: parse_i64_env(PILOT_RETENTION_DAYS_ENV, 14).clamp(1, 365),
        max_rows: parse_i64_env(PILOT_MAX_ROWS_ENV, 500).clamp(50, 50_000),
    }
}

/// Initialize the application
#[tauri::command]
pub async fn initialize_app(state: State<'_, AppState>) -> Result<InitResult, String> {
    let init_start = std::time::Instant::now();

    // Run data migration from old path (com.d.assistsupport -> AssistSupport)
    // This must happen BEFORE any other operations to ensure data is in the right place
    match crate::migration::migrate_data_directories() {
        Ok(report) => {
            if report.migration_performed {
                tracing::info!(
                    "Data migration completed: {} items migrated, {} skipped, {} conflicts",
                    report.migrated.len(),
                    report.skipped.len(),
                    report.conflicts.len()
                );
                for item in &report.migrated {
                    tracing::info!("  Migrated: {}", item.name);
                }
                for conflict in &report.conflicts {
                    tracing::warn!("  Conflict: {} - {}", conflict.name, conflict.reason);
                }
            }
        }
        Err(e) => {
            tracing::error!("Data migration failed: {}", e);
            // Continue anyway - migration failures shouldn't block app startup
        }
    }

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

    if let Some(maintenance_result) = crate::diagnostics::run_database_maintenance_if_due(&db) {
        if maintenance_result.success {
            tracing::info!(
                "Database maintenance policy run completed during startup: {}",
                maintenance_result.action_taken
            );
        } else {
            tracing::warn!(
                "Database maintenance policy run reported issues during startup: {} ({})",
                maintenance_result.action_taken,
                maintenance_result.message.as_deref().unwrap_or("no details")
            );
        }
    }

    // Seed built-in decision trees on first run
    db.seed_builtin_trees().map_err(|e| e.to_string())?;

    // Ensure response_templates table exists
    db.ensure_templates_table().map_err(|e| e.to_string())?;

    // Migrate namespace IDs to canonical normalized form
    match db.migrate_namespace_ids() {
        Ok(migrated) => {
            if !migrated.is_empty() {
                tracing::info!(
                    "Namespace ID migration: {} namespaces updated",
                    migrated.len()
                );
                for (old_id, new_id) in &migrated {
                    tracing::info!("  '{}' -> '{}'", old_id, new_id);
                }
            }
        }
        Err(e) => {
            tracing::error!("Namespace ID migration failed: {}", e);
            // Continue anyway - this shouldn't block app startup
        }
    }

    // Check vector consent from database
    let vector_enabled = db.get_vector_consent().map(|c| c.enabled).unwrap_or(false);

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
                eprintln!(
                    "Vector store init failed (continuing without vectors): {}",
                    e
                );
                false
            }
        }
    } else {
        false
    };

    // Record startup metrics
    let init_app_ms = init_start.elapsed().as_millis() as i64;
    {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        if let Some(db) = db_lock.as_ref() {
            let _ = db.record_startup_metric(
                &chrono::Utc::now().to_rfc3339(),
                None,
                Some(init_app_ms),
                Some(init_app_ms),
                false,
            );
        }
    }
    tracing::info!("App initialized in {}ms", init_app_ms);

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
    db.set_vector_consent(enabled, encryption_supported)
        .map_err(|e| e.to_string())
}

/// Check if credential storage is available
/// (Always true now that we use file-based storage)
#[tauri::command]
pub fn check_keychain_available() -> bool {
    true // File-based storage is always available
}

// ── Pilot Feedback commands ─────────────────────────────────────────────

/// Log a query and its response for pilot tracking
#[tauri::command]
pub fn log_pilot_query(
    state: State<'_, AppState>,
    query: String,
    response: String,
    operator_id: String,
) -> Result<String, String> {
    require_pilot_logging_enabled()?;
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    crate::feedback::log_query(db, &query, &response, &operator_id)
}

/// Submit user feedback on a pilot query response
#[tauri::command]
pub fn submit_pilot_feedback(
    state: State<'_, AppState>,
    query_log_id: String,
    operator_id: String,
    accuracy: i32,
    clarity: i32,
    helpfulness: i32,
    comment: Option<String>,
) -> Result<String, String> {
    require_pilot_logging_enabled()?;
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    crate::feedback::submit_feedback(
        db,
        &query_log_id,
        &operator_id,
        accuracy,
        clarity,
        helpfulness,
        comment.as_deref(),
    )
}

/// Get pilot dashboard summary stats
#[tauri::command]
pub fn get_pilot_stats(state: State<'_, AppState>) -> Result<crate::feedback::PilotStats, String> {
    require_pilot_logging_enabled()?;
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    crate::feedback::get_pilot_stats(db)
}

/// Get all pilot query logs
#[tauri::command]
pub fn get_pilot_query_logs(
    state: State<'_, AppState>,
) -> Result<Vec<crate::feedback::QueryLog>, String> {
    require_pilot_logging_enabled()?;
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    crate::feedback::get_query_logs(db)
}

/// Export pilot data to CSV
#[tauri::command]
pub fn export_pilot_data(state: State<'_, AppState>, path: String) -> Result<usize, String> {
    use std::path::Path;

    require_pilot_logging_enabled()?;

    let candidate = Path::new(&path);
    let ext = candidate
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext != "csv" {
        return Err("Export path must be a .csv file".into());
    }

    let validated_path = validate_output_file_within_home(candidate).map_err(|e| match e {
        ValidationError::PathTraversal => "Export path must be within your home directory".into(),
        ValidationError::PathNotFound(_) => "Export parent directory does not exist".into(),
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This export path is blocked because it contains sensitive data".into()
        }
        _ => format!("Invalid export path: {}", e),
    })?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    crate::feedback::export::export_to_csv(db, validated_path.as_path())
}
