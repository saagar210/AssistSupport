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
            match db.conn().query_row::<i64, _, _>("SELECT COUNT(*) FROM kb_documents", [], |r| r.get(0)) {
                Ok(count) => {
                    ComponentHealth::healthy(
                        "Database",
                        &format!("OK - {} documents indexed", count)
                    )
                }
                Err(e) => {
                    ComponentHealth::error(
                        "Database",
                        "Query failed",
                        Some(&e.to_string()),
                        false
                    )
                }
            }
        }
        Err(e) => {
            ComponentHealth::error(
                "Database",
                "Integrity check failed",
                Some(&e.to_string()),
                true // Can attempt VACUUM/repair
            )
        }
    }
}

/// Check vector store health (async version needed due to count())
pub fn check_vector_store_health_sync(vectors: Option<&crate::kb::vectors::VectorStore>) -> ComponentHealth {
    match vectors {
        Some(store) => {
            if !store.is_enabled() {
                ComponentHealth::warning(
                    "Vector Store",
                    "Disabled",
                    Some("Enable vector search in Settings to use semantic search")
                )
            } else {
                let dim = store.embedding_dim();
                ComponentHealth::healthy(
                    "Vector Store",
                    &format!("OK - dim={}", dim)
                )
            }
        }
        None => {
            ComponentHealth::unavailable(
                "Vector Store",
                "Not initialized"
            )
        }
    }
}

/// Check vector store health with count (async)
pub async fn check_vector_store_health(vectors: Option<&crate::kb::vectors::VectorStore>) -> ComponentHealth {
    match vectors {
        Some(store) => {
            if !store.is_enabled() {
                return ComponentHealth::warning(
                    "Vector Store",
                    "Disabled",
                    Some("Enable vector search in Settings to use semantic search")
                );
            }

            match store.count().await {
                Ok(count) => {
                    if count == 0 {
                        ComponentHealth::warning(
                            "Vector Store",
                            "No vectors indexed",
                            Some("Run 'Generate Embeddings' to enable semantic search")
                        )
                    } else {
                        ComponentHealth::healthy(
                            "Vector Store",
                            &format!("OK - {} vectors, dim={}", count, store.embedding_dim())
                        )
                    }
                }
                Err(e) => {
                    ComponentHealth::error(
                        "Vector Store",
                        "Count query failed",
                        Some(&e.to_string()),
                        true // Can rebuild
                    )
                }
            }
        }
        None => {
            ComponentHealth::unavailable(
                "Vector Store",
                "Not initialized"
            )
        }
    }
}

/// Check LLM engine health
pub fn check_llm_health(llm: Option<&crate::llm::LlmEngine>) -> ComponentHealth {
    match llm {
        Some(engine) => {
            if engine.is_model_loaded() {
                match engine.model_info() {
                    Some(info) => {
                        ComponentHealth::healthy(
                            "LLM Engine",
                            &format!("OK - {} loaded", info.name)
                        )
                    }
                    None => {
                        ComponentHealth::warning(
                            "LLM Engine",
                            "Model loaded but info unavailable",
                            None
                        )
                    }
                }
            } else {
                ComponentHealth::warning(
                    "LLM Engine",
                    "No model loaded",
                    Some("Load a model from the Settings tab")
                )
            }
        }
        None => {
            ComponentHealth::unavailable(
                "LLM Engine",
                "Not initialized"
            )
        }
    }
}

/// Check embedding model health
pub fn check_embedding_health(embeddings: Option<&crate::kb::embeddings::EmbeddingEngine>) -> ComponentHealth {
    match embeddings {
        Some(engine) => {
            if engine.is_model_loaded() {
                ComponentHealth::healthy(
                    "Embedding Model",
                    "OK - Model loaded"
                )
            } else {
                ComponentHealth::warning(
                    "Embedding Model",
                    "No model loaded",
                    Some("Embedding model will load when generating embeddings")
                )
            }
        }
        None => {
            ComponentHealth::unavailable(
                "Embedding Model",
                "Not initialized"
            )
        }
    }
}

/// Check file system health (data directories)
pub fn check_filesystem_health() -> ComponentHealth {
    let app_data_dir = dirs::data_local_dir()
        .map(|p| p.join("com.assistsupport.app"));

    match app_data_dir {
        Some(path) => {
            if path.exists() {
                // Check if we can write
                let test_path = path.join(".health_check");
                match std::fs::write(&test_path, "test") {
                    Ok(()) => {
                        let _ = std::fs::remove_file(&test_path);

                        ComponentHealth::healthy(
                            "File System",
                            &format!("OK - {}", path.display())
                        )
                    }
                    Err(e) => {
                        ComponentHealth::error(
                            "File System",
                            "Cannot write to data directory",
                            Some(&e.to_string()),
                            false
                        )
                    }
                }
            } else {
                ComponentHealth::warning(
                    "File System",
                    "Data directory does not exist",
                    Some("Will be created on first run")
                )
            }
        }
        None => {
            ComponentHealth::error(
                "File System",
                "Cannot determine data directory",
                None,
                false
            )
        }
    }
}

/// Attempt to repair the database
pub fn repair_database(db: &crate::db::Database) -> RepairResult {
    // Try VACUUM to repair
    match db.conn().execute("VACUUM", []) {
        Ok(_) => {
            // Re-check integrity
            match db.check_integrity() {
                Ok(()) => {
                    RepairResult {
                        component: "Database".to_string(),
                        success: true,
                        action_taken: "Ran VACUUM to compact and repair database".to_string(),
                        message: Some("Database integrity restored".to_string()),
                    }
                }
                Err(e) => {
                    RepairResult {
                        component: "Database".to_string(),
                        success: false,
                        action_taken: "VACUUM completed but integrity check still fails".to_string(),
                        message: Some(format!("Manual intervention may be required: {}", e)),
                    }
                }
            }
        }
        Err(e) => {
            RepairResult {
                component: "Database".to_string(),
                success: false,
                action_taken: "VACUUM failed".to_string(),
                message: Some(e.to_string()),
            }
        }
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
             2) Click 'Generate Embeddings'. This will re-process all chunks.".to_string()
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
