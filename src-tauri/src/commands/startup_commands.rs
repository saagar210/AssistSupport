use super::vector_runtime::vector_store_requires_rebuild;
use crate::audit::{self, AuditLogger};
use crate::db::{get_app_data_dir, get_db_path, get_vectors_dir, Database};
use crate::kb::vectors::{VectorStore, VectorStoreConfig};
use crate::security::{FileKeyStore, KeyStorageMode};
use crate::{
    AppState, PendingRecoveryContext, StartupRecoveryConflict, StartupRecoveryIssue,
};
use std::path::PathBuf;
use tauri::State;

#[derive(serde::Serialize, Clone)]
pub struct InitResult {
    pub is_first_run: bool,
    pub vector_enabled: bool,
    pub vector_store_ready: bool,
    pub key_storage_mode: String,
    pub passphrase_required: bool,
    pub recovery_issue: Option<StartupRecoveryIssue>,
}

fn init_result(
    is_first_run: bool,
    vector_enabled: bool,
    vector_store_ready: bool,
    key_storage_mode: String,
    passphrase_required: bool,
    recovery_issue: Option<StartupRecoveryIssue>,
) -> InitResult {
    InitResult {
        is_first_run,
        vector_enabled,
        vector_store_ready,
        key_storage_mode,
        passphrase_required,
        recovery_issue,
    }
}

fn set_pending_recovery(
    state: &AppState,
    context: PendingRecoveryContext,
) -> Result<(), String> {
    let mut recovery_lock = state.recovery.lock().map_err(|e| e.to_string())?;
    *recovery_lock = Some(context);
    Ok(())
}

fn clear_pending_recovery(state: &AppState) -> Result<(), String> {
    let mut recovery_lock = state.recovery.lock().map_err(|e| e.to_string())?;
    *recovery_lock = None;
    Ok(())
}

fn build_migration_conflict_issue(
    report: &crate::migration::MigrationReport,
) -> StartupRecoveryIssue {
    let migration_conflicts = report
        .conflicts
        .iter()
        .map(|conflict| StartupRecoveryConflict {
            name: conflict.name.clone(),
            old_path: conflict.old_path.display().to_string(),
            new_path: conflict.new_path.display().to_string(),
            reason: conflict.reason.clone(),
        })
        .collect::<Vec<_>>();

    StartupRecoveryIssue {
        code: "migration_conflict".to_string(),
        summary: "Startup requires manual migration conflict resolution".to_string(),
        details: Some(
            "AssistSupport found data in both the old and new app-data locations. Review the conflicting paths below, decide which copy to keep, and then retry startup."
                .to_string(),
        ),
        can_repair: false,
        can_restore_backup: false,
        requires_manual_resolution: true,
        migration_conflicts,
    }
}

fn classify_startup_recovery_issue(error: &str) -> Option<StartupRecoveryIssue> {
    let lower = error.to_ascii_lowercase();

    if lower.contains("database disk image is malformed")
        || lower.contains("integrity check failed")
        || lower.contains("database corruption")
        || lower.contains("foreign key")
        || lower.contains("corrupt")
    {
        return Some(StartupRecoveryIssue {
            code: "database_recovery_required".to_string(),
            summary: "Startup entered recovery mode".to_string(),
            details: Some(error.to_string()),
            can_repair: true,
            can_restore_backup: true,
            requires_manual_resolution: false,
            migration_conflicts: Vec::new(),
        });
    }

    None
}

fn recovery_result_from_database_error(
    state: &AppState,
    is_first_run: bool,
    key_storage_mode: String,
    db_path: PathBuf,
    master_key: crate::security::MasterKey,
    error: String,
) -> Result<InitResult, String> {
    if let Some(issue) = classify_startup_recovery_issue(&error) {
        set_pending_recovery(
            state,
            PendingRecoveryContext {
                issue: issue.clone(),
                db_path: Some(db_path),
                master_key: Some(master_key),
                key_storage_mode: Some(key_storage_mode.clone()),
            },
        )?;

        return Ok(init_result(
            is_first_run,
            false,
            false,
            key_storage_mode,
            false,
            Some(issue),
        ));
    }

    Err(error)
}

async fn finalize_initialized_app(
    state: &AppState,
    db: Database,
    is_first_run: bool,
    init_start: std::time::Instant,
    key_storage_mode: String,
) -> Result<InitResult, String> {
    db.seed_builtin_trees().map_err(|e| e.to_string())?;
    db.ensure_templates_table().map_err(|e| e.to_string())?;

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
        }
    }

    let vector_enabled = db.get_vector_consent().map(|c| c.enabled).unwrap_or(false);

    clear_pending_recovery(state)?;

    {
        let mut db_lock = state.db.lock().map_err(|e| e.to_string())?;
        *db_lock = Some(db);
    }

    let vector_store_ready = if vector_enabled {
        let config = VectorStoreConfig {
            path: get_vectors_dir(),
            embedding_dim: 768,
            encryption_enabled: false,
        };

        let mut vector_store = VectorStore::new(config);
        match vector_store.init().await {
            Ok(()) => {
                if let Err(error) = vector_store.create_table().await {
                    tracing::warn!("Vector store table init failed: {}", error);
                    *state.vectors.write().await = Some(vector_store);
                    false
                } else {
                    let tracked_vector_version = {
                        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
                        let db = db_lock.as_ref().ok_or("Database not initialized")?;
                        db.get_vector_store_version().map_err(|e| e.to_string())?
                    };
                    let ready =
                        !vector_store_requires_rebuild(tracked_vector_version, &vector_store)
                            .await?;

                    if ready {
                        vector_store.enable(true).map_err(|e| e.to_string())?;
                    } else {
                        tracing::warn!(
                            "Vector store requires rebuild before semantic search can be enabled"
                        );
                    }

                    *state.vectors.write().await = Some(vector_store);
                    ready
                }
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

    Ok(init_result(
        is_first_run,
        vector_enabled,
        vector_store_ready,
        key_storage_mode,
        false,
        None,
    ))
}

/// Initialize the application
#[tauri::command]
pub async fn initialize_app(state: State<'_, AppState>) -> Result<InitResult, String> {
    let init_start = std::time::Instant::now();
    clear_pending_recovery(state.inner())?;
    let is_first_run_hint = !FileKeyStore::has_master_key();

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

            if !report.conflicts.is_empty() {
                let issue = build_migration_conflict_issue(&report);
                set_pending_recovery(
                    state.inner(),
                    PendingRecoveryContext {
                        issue: issue.clone(),
                        db_path: None,
                        master_key: None,
                        key_storage_mode: None,
                    },
                )?;

                return Ok(init_result(
                    is_first_run_hint,
                    false,
                    false,
                    "unknown".to_string(),
                    false,
                    Some(issue),
                ));
            }
        }
        Err(e) => {
            tracing::error!("Data migration failed: {}", e);
        }
    }

    let app_dir = get_app_data_dir();
    crate::security::create_secure_dir(&app_dir).map_err(|e| e.to_string())?;

    let _ = AuditLogger::init();

    let is_first_run = is_first_run_hint;

    let master_key = match FileKeyStore::get_master_key() {
        Ok(key) => key,
        Err(crate::security::SecurityError::PassphraseRequired) => {
            return Ok(init_result(
                is_first_run,
                false,
                false,
                KeyStorageMode::Passphrase.to_string(),
                true,
                None,
            ));
        }
        Err(e) => return Err(e.to_string()),
    };

    audit::audit_app_initialized(is_first_run);

    let db_path = get_db_path();
    let db = match Database::open(&db_path, &master_key) {
        Ok(db) => db,
        Err(e) => {
            return recovery_result_from_database_error(
                state.inner(),
                is_first_run,
                KeyStorageMode::Keychain.to_string(),
                db_path,
                master_key,
                e.to_string(),
            );
        }
    };
    if let Err(error) = db.initialize() {
        return recovery_result_from_database_error(
            state.inner(),
            is_first_run,
            KeyStorageMode::Keychain.to_string(),
            db_path,
            master_key,
            error.to_string(),
        );
    }
    finalize_initialized_app(
        state.inner(),
        db,
        is_first_run,
        init_start,
        KeyStorageMode::Keychain.to_string(),
    )
    .await
}

/// Unlock the application when passphrase-based key storage is configured.
#[tauri::command]
pub async fn unlock_with_passphrase(
    state: State<'_, AppState>,
    passphrase: String,
) -> Result<InitResult, String> {
    let init_start = std::time::Instant::now();
    clear_pending_recovery(state.inner())?;
    let master_key = FileKeyStore::get_master_key_with_passphrase(&passphrase)
        .map_err(|e| e.to_string())?;

    let db_path = get_db_path();
    let db = match Database::open(&db_path, &master_key) {
        Ok(db) => db,
        Err(e) => {
            return recovery_result_from_database_error(
                state.inner(),
                false,
                KeyStorageMode::Passphrase.to_string(),
                db_path,
                master_key,
                e.to_string(),
            );
        }
    };
    if let Err(error) = db.initialize() {
        return recovery_result_from_database_error(
            state.inner(),
            false,
            KeyStorageMode::Passphrase.to_string(),
            db_path,
            master_key,
            error.to_string(),
        );
    }

    finalize_initialized_app(
        state.inner(),
        db,
        false,
        init_start,
        KeyStorageMode::Passphrase.to_string(),
    )
    .await
}

/// Check if credential storage is available
/// (Always true now that we use file-based storage)
#[tauri::command]
pub fn check_keychain_available() -> bool {
    true
}
