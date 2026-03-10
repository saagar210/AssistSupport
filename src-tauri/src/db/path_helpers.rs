use std::path::PathBuf;

/// Get the application data directory
pub fn get_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .expect("Platform data directory must be available")
        .join("AssistSupport")
}

/// Get database path
pub fn get_db_path() -> PathBuf {
    get_app_data_dir().join("assistsupport.db")
}

/// Get attachments directory
pub fn get_attachments_dir() -> PathBuf {
    get_app_data_dir().join("attachments")
}

/// Get models directory
pub fn get_models_dir() -> PathBuf {
    get_app_data_dir().join("models")
}

/// Get vectors directory (LanceDB)
pub fn get_vectors_dir() -> PathBuf {
    get_app_data_dir().join("vectors")
}

/// Get downloads directory
pub fn get_downloads_dir() -> PathBuf {
    get_app_data_dir().join("downloads")
}

/// Get logs directory
pub fn get_logs_dir() -> PathBuf {
    dirs::data_dir()
        .expect("Platform data directory must be available")
        .join("Logs")
        .join("AssistSupport")
}

/// Get cache directory
pub fn get_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .expect("Platform cache directory must be available")
        .join("AssistSupport")
}
