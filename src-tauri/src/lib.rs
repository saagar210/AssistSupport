//! AssistSupport - Self-contained local KB + LLM app for IT support

pub mod backup;
pub mod commands;
pub mod db;
pub mod downloads;
pub mod jira;
pub mod kb;
pub mod llm;
pub mod prompts;
pub mod security;
pub mod validation;

use crate::db::Database;
use crate::kb::embeddings::EmbeddingEngine;
use crate::kb::vectors::VectorStore;
use crate::llm::LlmEngine;
use std::sync::{Arc, Mutex};
use parking_lot::RwLock;
use tokio::sync::RwLock as TokioRwLock;

/// Application state
pub struct AppState {
    pub db: Mutex<Option<Database>>,
    pub llm: Arc<RwLock<Option<LlmEngine>>>,
    pub embeddings: Arc<RwLock<Option<EmbeddingEngine>>>,
    pub vectors: Arc<TokioRwLock<Option<VectorStore>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            db: Mutex::new(None),
            llm: Arc::new(RwLock::new(None)),
            embeddings: Arc::new(RwLock::new(None)),
            vectors: Arc::new(TokioRwLock::new(None)),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::initialize_app,
            commands::check_fts5_enabled,
            commands::check_db_integrity,
            commands::get_vector_consent,
            commands::set_vector_consent,
            commands::check_keychain_available,
            commands::search_kb,
            commands::get_search_context,
            // LLM commands
            commands::init_llm_engine,
            commands::load_model,
            commands::load_custom_model,
            commands::validate_gguf_file,
            commands::unload_model,
            commands::get_model_info,
            commands::is_model_loaded,
            commands::generate_text,
            commands::generate_with_context,
            commands::generate_streaming,
            commands::test_model,
            commands::cancel_generation,
            commands::get_context_window,
            commands::set_context_window,
            // Download commands
            commands::get_recommended_models,
            commands::list_downloaded_models,
            commands::get_models_dir,
            commands::delete_downloaded_model,
            commands::has_hf_token,
            commands::set_hf_token,
            commands::clear_hf_token,
            commands::download_model,
            commands::cancel_download,
            // KB Indexer commands
            commands::set_kb_folder,
            commands::get_kb_folder,
            commands::index_kb,
            commands::get_kb_stats,
            commands::list_kb_documents,
            commands::remove_kb_document,
            commands::generate_kb_embeddings,
            // KB Watcher commands
            commands::start_kb_watcher,
            commands::stop_kb_watcher,
            commands::is_kb_watcher_running,
            // Embedding commands
            commands::init_embedding_engine,
            commands::load_embedding_model,
            commands::unload_embedding_model,
            commands::get_embedding_model_info,
            commands::is_embedding_model_loaded,
            commands::get_embedding_model_path,
            commands::is_embedding_model_downloaded,
            // Vector store commands
            commands::init_vector_store,
            commands::set_vector_enabled,
            commands::is_vector_enabled,
            commands::get_vector_stats,
            // OCR commands
            commands::process_ocr,
            commands::process_ocr_bytes,
            commands::is_ocr_available,
            // Decision Tree commands
            commands::list_decision_trees,
            commands::get_decision_tree,
            // Jira commands
            commands::is_jira_configured,
            commands::get_jira_config,
            commands::configure_jira,
            commands::clear_jira_config,
            commands::get_jira_ticket,
            // Draft & Template commands
            commands::list_drafts,
            commands::search_drafts,
            commands::get_draft,
            commands::save_draft,
            commands::delete_draft,
            commands::list_autosaves,
            commands::cleanup_autosaves,
            commands::get_draft_versions,
            commands::list_templates,
            commands::get_template,
            commands::save_template,
            commands::delete_template,
            // Custom variable commands
            commands::list_custom_variables,
            commands::get_custom_variable,
            commands::save_custom_variable,
            commands::delete_custom_variable,
            // Export commands
            commands::export_draft,
            // Backup/Restore commands
            commands::export_backup,
            commands::preview_backup_import,
            commands::import_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use crate::security::KeychainManager;

    #[test]
    fn test_keychain_available() {
        // This will vary by environment
        let available = KeychainManager::is_available();
        println!("Keychain available: {}", available);
    }
}
