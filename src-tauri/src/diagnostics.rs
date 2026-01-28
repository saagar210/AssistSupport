//! System diagnostics and health checks for AssistSupport
//! Provides health status for all components and recovery workflows

/// Overall system health status
#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemHealth {
    /// Database health status
    pub database: ComponentHealth,
    /// Vector store health status
    pub vector_store: ComponentHealth,
    /// LLM engine health status
    pub llm_engine: ComponentHealth,
    /// Embedding model health status
    pub embedding_model: ComponentHealth,
    /// File system health (data directories)
    pub file_system: ComponentHealth,
    /// Overall status (worst of all components)
    pub overall_status: HealthStatus,
    /// Timestamp of this health check
    pub checked_at: String,
}

/// Health status for a single component
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Current status
    pub status: HealthStatus,
    /// Human-readable status message
    pub message: String,
    /// Optional details for troubleshooting
    pub details: Option<String>,
    /// Whether auto-repair is available
    pub can_repair: bool,
}

/// Health status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Component is working normally
    Healthy,
    /// Component has warnings but is functional
    Warning,
    /// Component has errors and may not work correctly
    Error,
    /// Component is not initialized or unavailable
    Unavailable,
}

impl HealthStatus {
    /// Get the worst status between two
    pub fn worst(self, other: Self) -> Self {
        match (self, other) {
            (Self::Error, _) | (_, Self::Error) => Self::Error,
            (Self::Unavailable, _) | (_, Self::Unavailable) => Self::Unavailable,
            (Self::Warning, _) | (_, Self::Warning) => Self::Warning,
            _ => Self::Healthy,
        }
    }
}

impl ComponentHealth {
    pub fn healthy(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            status: HealthStatus::Healthy,
            message: message.to_string(),
            details: None,
            can_repair: false,
        }
    }

    pub fn warning(name: &str, message: &str, details: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            status: HealthStatus::Warning,
            message: message.to_string(),
            details: details.map(String::from),
            can_repair: false,
        }
    }

    pub fn error(name: &str, message: &str, details: Option<&str>, can_repair: bool) -> Self {
        Self {
            name: name.to_string(),
            status: HealthStatus::Error,
            message: message.to_string(),
            details: details.map(String::from),
            can_repair,
        }
    }

    pub fn unavailable(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            status: HealthStatus::Unavailable,
            message: message.to_string(),
            details: None,
            can_repair: false,
        }
    }
}

/// Result of a repair attempt
#[derive(Debug, Clone, serde::Serialize)]
pub struct RepairResult {
    /// Component that was repaired
    pub component: String,
    /// Whether repair was successful
    pub success: bool,
    /// Description of what was done
    pub action_taken: String,
    /// Any additional message
    pub message: Option<String>,
}

/// Check database health
pub fn check_database_health(db: &crate::db::Database) -> ComponentHealth {
    // Check integrity
    match db.check_integrity() {
        Ok(()) => {
            // Check basic operations work
            match db
                .conn()
                .query_row::<i64, _, _>("SELECT COUNT(*) FROM kb_documents", [], |r| r.get(0))
            {
                Ok(count) => ComponentHealth::healthy(
                    "Database",
                    &format!("OK - {} documents indexed", count),
                ),
                Err(e) => {
                    ComponentHealth::error("Database", "Query failed", Some(&e.to_string()), false)
                }
            }
        }
        Err(e) => {
            ComponentHealth::error(
                "Database",
                "Integrity check failed",
                Some(&e.to_string()),
                true, // Can attempt VACUUM/repair
            )
        }
    }
}

/// Check vector store health (async version needed due to count())
pub fn check_vector_store_health_sync(
    vectors: Option<&crate::kb::vectors::VectorStore>,
) -> ComponentHealth {
    match vectors {
        Some(store) => {
            if !store.is_enabled() {
                ComponentHealth::warning(
                    "Vector Store",
                    "Disabled",
                    Some("Enable vector search in Settings to use semantic search"),
                )
            } else {
                let dim = store.embedding_dim();
                ComponentHealth::healthy("Vector Store", &format!("OK - dim={}", dim))
            }
        }
        None => ComponentHealth::unavailable("Vector Store", "Not initialized"),
    }
}

/// Check vector store health with count (async)
pub async fn check_vector_store_health(
    vectors: Option<&crate::kb::vectors::VectorStore>,
) -> ComponentHealth {
    match vectors {
        Some(store) => {
            if !store.is_enabled() {
                return ComponentHealth::warning(
                    "Vector Store",
                    "Disabled",
                    Some("Enable vector search in Settings to use semantic search"),
                );
            }

            match store.count().await {
                Ok(count) => {
                    if count == 0 {
                        ComponentHealth::warning(
                            "Vector Store",
                            "No vectors indexed",
                            Some("Run 'Generate Embeddings' to enable semantic search"),
                        )
                    } else {
                        ComponentHealth::healthy(
                            "Vector Store",
                            &format!("OK - {} vectors, dim={}", count, store.embedding_dim()),
                        )
                    }
                }
                Err(e) => {
                    ComponentHealth::error(
                        "Vector Store",
                        "Count query failed",
                        Some(&e.to_string()),
                        true, // Can rebuild
                    )
                }
            }
        }
        None => ComponentHealth::unavailable("Vector Store", "Not initialized"),
    }
}

/// Check LLM engine health
pub fn check_llm_health(llm: Option<&crate::llm::LlmEngine>) -> ComponentHealth {
    match llm {
        Some(engine) => {
            if engine.is_model_loaded() {
                match engine.model_info() {
                    Some(info) => ComponentHealth::healthy(
                        "LLM Engine",
                        &format!("OK - {} loaded", info.name),
                    ),
                    None => ComponentHealth::warning(
                        "LLM Engine",
                        "Model loaded but info unavailable",
                        None,
                    ),
                }
            } else {
                ComponentHealth::warning(
                    "LLM Engine",
                    "No model loaded",
                    Some("Load a model from the Settings tab"),
                )
            }
        }
        None => ComponentHealth::unavailable("LLM Engine", "Not initialized"),
    }
}

/// Check embedding model health
pub fn check_embedding_health(
    embeddings: Option<&crate::kb::embeddings::EmbeddingEngine>,
) -> ComponentHealth {
    match embeddings {
        Some(engine) => {
            if engine.is_model_loaded() {
                ComponentHealth::healthy("Embedding Model", "OK - Model loaded")
            } else {
                ComponentHealth::warning(
                    "Embedding Model",
                    "No model loaded",
                    Some("Embedding model will load when generating embeddings"),
                )
            }
        }
        None => ComponentHealth::unavailable("Embedding Model", "Not initialized"),
    }
}

/// Check file system health (data directories)
pub fn check_filesystem_health() -> ComponentHealth {
    let app_data_dir = dirs::data_dir().map(|p| p.join("AssistSupport"));

    match app_data_dir {
        Some(path) => {
            if path.exists() {
                // Check if we can write
                let test_path = path.join(".health_check");
                match std::fs::write(&test_path, "test") {
                    Ok(()) => {
                        let _ = std::fs::remove_file(&test_path);

                        ComponentHealth::healthy("File System", &format!("OK - {}", path.display()))
                    }
                    Err(e) => ComponentHealth::error(
                        "File System",
                        "Cannot write to data directory",
                        Some(&e.to_string()),
                        false,
                    ),
                }
            } else {
                ComponentHealth::warning(
                    "File System",
                    "Data directory does not exist",
                    Some("Will be created on first run"),
                )
            }
        }
        None => ComponentHealth::error(
            "File System",
            "Cannot determine data directory",
            None,
            false,
        ),
    }
}

/// Attempt to repair the database
pub fn repair_database(db: &crate::db::Database) -> RepairResult {
    // Try VACUUM to repair
    match db.conn().execute("VACUUM", []) {
        Ok(_) => {
            // Re-check integrity
            match db.check_integrity() {
                Ok(()) => RepairResult {
                    component: "Database".to_string(),
                    success: true,
                    action_taken: "Ran VACUUM to compact and repair database".to_string(),
                    message: Some("Database integrity restored".to_string()),
                },
                Err(e) => RepairResult {
                    component: "Database".to_string(),
                    success: false,
                    action_taken: "VACUUM completed but integrity check still fails".to_string(),
                    message: Some(format!("Manual intervention may be required: {}", e)),
                },
            }
        }
        Err(e) => RepairResult {
            component: "Database".to_string(),
            success: false,
            action_taken: "VACUUM failed".to_string(),
            message: Some(e.to_string()),
        },
    }
}

/// Get chunk count from database for vector rebuild
pub fn get_chunk_count(db: &crate::db::Database) -> Result<usize, String> {
    db.conn()
        .query_row::<i64, _, _>("SELECT COUNT(*) FROM kb_chunks", [], |r| r.get(0))
        .map(|c| c as usize)
        .map_err(|e| e.to_string())
}

/// Note: Full vector store rebuild should be done through the generate_kb_embeddings command
/// This provides guidance on the repair process
pub fn get_vector_rebuild_guidance() -> RepairResult {
    RepairResult {
        component: "Vector Store".to_string(),
        success: true,
        action_taken: "Guidance provided".to_string(),
        message: Some(
            "To rebuild vectors: 1) Go to Settings > Knowledge Base, \
             2) Click 'Generate Embeddings'. This will re-process all chunks."
                .to_string(),
        ),
    }
}

/// Common failure modes and their solutions
#[derive(Debug, Clone, serde::Serialize)]
pub struct FailureMode {
    /// Problem identifier
    pub id: String,
    /// Human-readable problem description
    pub problem: String,
    /// Symptoms that indicate this failure
    pub symptoms: Vec<String>,
    /// Steps to resolve
    pub resolution_steps: Vec<String>,
    /// Whether auto-repair is available
    pub auto_repair_available: bool,
}

/// Get list of known failure modes and their resolutions
pub fn get_failure_modes() -> Vec<FailureMode> {
    vec![
        FailureMode {
            id: "db_corruption".to_string(),
            problem: "Database corruption".to_string(),
            symptoms: vec![
                "App crashes on startup".to_string(),
                "Error: database disk image is malformed".to_string(),
                "Search returns no results unexpectedly".to_string(),
            ],
            resolution_steps: vec![
                "Use the 'Repair Database' option in diagnostics".to_string(),
                "If repair fails, restore from backup".to_string(),
                "As a last resort, delete the database file and re-index".to_string(),
            ],
            auto_repair_available: true,
        },
        FailureMode {
            id: "model_load_fail".to_string(),
            problem: "Model fails to load".to_string(),
            symptoms: vec![
                "Error: Failed to load model".to_string(),
                "Generation produces no output".to_string(),
                "App freezes when loading model".to_string(),
            ],
            resolution_steps: vec![
                "Check disk space (models need several GB)".to_string(),
                "Verify the model file is not corrupted".to_string(),
                "Try downloading the model again".to_string(),
                "Check that your system meets RAM requirements".to_string(),
            ],
            auto_repair_available: false,
        },
        FailureMode {
            id: "vector_mismatch".to_string(),
            problem: "Vector store out of sync".to_string(),
            symptoms: vec![
                "Semantic search returns stale results".to_string(),
                "New documents don't appear in search".to_string(),
                "Vector count doesn't match document count".to_string(),
            ],
            resolution_steps: vec![
                "Use 'Rebuild Vector Store' in diagnostics".to_string(),
                "Ensure embedding model is loaded".to_string(),
                "Re-run 'Generate Embeddings' from Settings".to_string(),
            ],
            auto_repair_available: true,
        },
        FailureMode {
            id: "kb_index_stale".to_string(),
            problem: "Knowledge base index is stale".to_string(),
            symptoms: vec![
                "Recently modified files not showing in search".to_string(),
                "Document changes not reflected".to_string(),
            ],
            resolution_steps: vec![
                "Re-index the knowledge base folder".to_string(),
                "Check KB folder path is correct".to_string(),
                "Verify file permissions".to_string(),
            ],
            auto_repair_available: true,
        },
        FailureMode {
            id: "keychain_error".to_string(),
            problem: "Keychain/credential storage error".to_string(),
            symptoms: vec![
                "Error: Failed to store/retrieve credential".to_string(),
                "Token storage fails".to_string(),
                "Authentication errors".to_string(),
            ],
            resolution_steps: vec![
                "Check system keychain is accessible".to_string(),
                "Grant app access to keychain if prompted".to_string(),
                "Clear and re-enter credentials".to_string(),
            ],
            auto_repair_available: false,
        },
    ]
}

/// Database maintenance statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct DatabaseStats {
    /// Size of database file in bytes
    pub file_size_bytes: u64,
    /// Number of KB documents
    pub document_count: i64,
    /// Number of KB chunks
    pub chunk_count: i64,
    /// Number of drafts
    pub draft_count: i64,
    /// Number of jobs
    pub job_count: i64,
    /// Page count from SQLite
    pub page_count: i64,
    /// Freelist count (unused pages) from SQLite
    pub freelist_count: i64,
    /// Last vacuum timestamp if stored
    pub last_vacuum: Option<String>,
}

/// Get database statistics for monitoring
pub fn get_database_stats(
    db: &crate::db::Database,
    db_path: &std::path::Path,
) -> Result<DatabaseStats, String> {
    let file_size_bytes = std::fs::metadata(db_path).map(|m| m.len()).unwrap_or(0);

    let document_count: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM kb_documents", [], |r| r.get(0))
        .unwrap_or(0);

    let chunk_count: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM kb_chunks", [], |r| r.get(0))
        .unwrap_or(0);

    let draft_count: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM drafts", [], |r| r.get(0))
        .unwrap_or(0);

    let job_count: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM jobs", [], |r| r.get(0))
        .unwrap_or(0);

    let page_count: i64 = db
        .conn()
        .query_row("PRAGMA page_count", [], |r| r.get(0))
        .unwrap_or(0);

    let freelist_count: i64 = db
        .conn()
        .query_row("PRAGMA freelist_count", [], |r| r.get(0))
        .unwrap_or(0);

    let last_vacuum: Option<String> = db
        .conn()
        .query_row(
            "SELECT value FROM settings WHERE key = 'last_vacuum'",
            [],
            |r| r.get(0),
        )
        .ok();

    Ok(DatabaseStats {
        file_size_bytes,
        document_count,
        chunk_count,
        draft_count,
        job_count,
        page_count,
        freelist_count,
        last_vacuum,
    })
}

/// Run scheduled database maintenance (VACUUM if needed)
pub fn run_database_maintenance(db: &crate::db::Database) -> RepairResult {
    // Check if VACUUM is needed (freelist > 10% of pages)
    let page_count: i64 = db
        .conn()
        .query_row("PRAGMA page_count", [], |r| r.get(0))
        .unwrap_or(1);

    let freelist_count: i64 = db
        .conn()
        .query_row("PRAGMA freelist_count", [], |r| r.get(0))
        .unwrap_or(0);

    let fragmentation_pct = (freelist_count as f64 / page_count as f64) * 100.0;

    if fragmentation_pct < 10.0 {
        // No maintenance needed
        return RepairResult {
            component: "Database".to_string(),
            success: true,
            action_taken: "Checked database - no maintenance needed".to_string(),
            message: Some(format!(
                "Fragmentation at {:.1}% (threshold: 10%)",
                fragmentation_pct
            )),
        };
    }

    // Run VACUUM
    match db.conn().execute("VACUUM", []) {
        Ok(_) => {
            // Record timestamp
            let now = chrono::Utc::now().to_rfc3339();
            let _ = db.conn().execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES ('last_vacuum', ?)",
                [&now],
            );

            RepairResult {
                component: "Database".to_string(),
                success: true,
                action_taken: format!(
                    "VACUUM completed (was {:.1}% fragmented)",
                    fragmentation_pct
                ),
                message: Some("Database optimized successfully".to_string()),
            }
        }
        Err(e) => RepairResult {
            component: "Database".to_string(),
            success: false,
            action_taken: "VACUUM failed".to_string(),
            message: Some(e.to_string()),
        },
    }
}

/// Resource usage metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResourceMetrics {
    /// Process memory usage in bytes (resident)
    pub memory_bytes: u64,
    /// Estimated memory threshold for warnings (based on system)
    pub memory_threshold_bytes: u64,
    /// Whether memory usage is concerning
    pub memory_warning: bool,
}

/// Get process resident memory in bytes using platform-native APIs.
/// Returns 0 on non-macOS platforms or on failure.
#[cfg(target_os = "macos")]
fn get_process_memory_bytes() -> u64 {
    use std::mem::MaybeUninit;

    // mach FFI types
    type MachPort = u32;
    type KernReturn = i32;

    #[repr(C)]
    struct MachTaskBasicInfo {
        virtual_size: u64,
        resident_size: u64,
        resident_size_max: u64,
        user_time: [u32; 2],   // time_value_t
        system_time: [u32; 2], // time_value_t
        policy: i32,
        suspend_count: i32,
    }

    const MACH_TASK_BASIC_INFO: u32 = 20;
    const MACH_TASK_BASIC_INFO_COUNT: u32 =
        (std::mem::size_of::<MachTaskBasicInfo>() / std::mem::size_of::<u32>()) as u32;

    extern "C" {
        fn mach_task_self() -> MachPort;
        fn task_info(
            target_task: MachPort,
            flavor: u32,
            task_info_out: *mut MachTaskBasicInfo,
            task_info_out_cnt: *mut u32,
        ) -> KernReturn;
    }

    unsafe {
        let mut info = MaybeUninit::<MachTaskBasicInfo>::zeroed();
        let mut count = MACH_TASK_BASIC_INFO_COUNT;
        let kr = task_info(
            mach_task_self(),
            MACH_TASK_BASIC_INFO,
            info.as_mut_ptr(),
            &mut count,
        );
        if kr == 0 {
            info.assume_init().resident_size
        } else {
            0
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn get_process_memory_bytes() -> u64 {
    0
}

/// Get current resource usage metrics
pub fn get_resource_metrics() -> ResourceMetrics {
    let memory_bytes = get_process_memory_bytes();

    // Default threshold: 4GB for LLM operations
    let memory_threshold_bytes = 4 * 1024 * 1024 * 1024u64;

    ResourceMetrics {
        memory_bytes,
        memory_threshold_bytes,
        memory_warning: memory_bytes > memory_threshold_bytes,
    }
}

/// LLM resource limits configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LlmResourceLimits {
    /// Maximum memory usage in bytes before warning
    pub max_memory_bytes: u64,
    /// Maximum context tokens to use
    pub max_context_tokens: usize,
    /// Whether to enable watchdog that cancels generation on OOM
    pub enable_watchdog: bool,
    /// Timeout for generation in seconds
    pub generation_timeout_secs: u64,
}

impl Default for LlmResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 8 * 1024 * 1024 * 1024, // 8GB
            max_context_tokens: 4096,
            enable_watchdog: true,
            generation_timeout_secs: 120,
        }
    }
}

/// Vector store maintenance info
#[derive(Debug, Clone, serde::Serialize)]
pub struct VectorMaintenanceInfo {
    /// Current vector count
    pub vector_count: usize,
    /// Estimated storage size
    pub estimated_size_bytes: u64,
    /// Whether compaction is recommended
    pub compaction_recommended: bool,
    /// Last compaction timestamp if available
    pub last_compaction: Option<String>,
}

/// Get vector store maintenance info (async)
pub async fn get_vector_maintenance_info(
    vectors: Option<&crate::kb::vectors::VectorStore>,
) -> Option<VectorMaintenanceInfo> {
    let store = vectors?;

    if !store.is_enabled() {
        return None;
    }

    let vector_count = store.count().await.ok()?;
    let dim = store.embedding_dim();

    // Estimate size: vectors * dimensions * 4 bytes per float
    let estimated_size_bytes = (vector_count * dim * 4) as u64;

    // Recommend compaction if over 100k vectors
    let compaction_recommended = vector_count > 100_000;

    Some(VectorMaintenanceInfo {
        vector_count,
        estimated_size_bytes,
        compaction_recommended,
        last_compaction: None, // Would need to track this
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_worst() {
        assert_eq!(
            HealthStatus::Healthy.worst(HealthStatus::Warning),
            HealthStatus::Warning
        );
        assert_eq!(
            HealthStatus::Warning.worst(HealthStatus::Error),
            HealthStatus::Error
        );
        assert_eq!(
            HealthStatus::Healthy.worst(HealthStatus::Healthy),
            HealthStatus::Healthy
        );
    }

    #[test]
    fn test_component_health_constructors() {
        let healthy = ComponentHealth::healthy("Test", "OK");
        assert_eq!(healthy.status, HealthStatus::Healthy);
        assert!(!healthy.can_repair);

        let warning = ComponentHealth::warning("Test", "Warning", Some("Details"));
        assert_eq!(warning.status, HealthStatus::Warning);
        assert_eq!(warning.details, Some("Details".to_string()));

        let error = ComponentHealth::error("Test", "Error", None, true);
        assert_eq!(error.status, HealthStatus::Error);
        assert!(error.can_repair);
    }

    #[test]
    fn test_failure_modes() {
        let modes = get_failure_modes();
        assert!(!modes.is_empty());

        // Verify all modes have required fields
        for mode in modes {
            assert!(!mode.id.is_empty());
            assert!(!mode.problem.is_empty());
            assert!(!mode.symptoms.is_empty());
            assert!(!mode.resolution_steps.is_empty());
        }
    }

    #[test]
    fn test_filesystem_health() {
        let health = check_filesystem_health();
        // Should at least not panic
        assert!(!health.name.is_empty());
    }
}
