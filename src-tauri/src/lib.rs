mod audit;
mod chunker;
mod commands;
mod db;
mod embedder;
mod error;
mod graph;
mod metrics;
mod migrations;
mod models;
mod ollama;
mod parsers;
mod state;
pub mod utils;
mod vector_index;
mod vector_store;

use std::sync::RwLock;

use state::AppState;
use tauri::Manager;
use vector_index::VectorIndex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            let db_pool = db::create_pool(&app_data_dir)
                .expect("Failed to create database pool");

            app.manage(AppState {
                db_pool: db_pool.clone(),
                vector_index: RwLock::new(VectorIndex::new()),
                metrics: metrics::AppMetrics::new(),
            });

            // Build HNSW indices for all collections
            {
                let conn = db_pool.get().expect("Failed to get connection for index build");
                let mut stmt = conn
                    .prepare("SELECT id FROM collections")
                    .expect("Failed to prepare collections query");
                let collection_ids: Vec<String> = stmt
                    .query_map([], |row| row.get::<_, String>(0))
                    .expect("Failed to query collections")
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap_or_default();

                let app_state: &AppState = &app.state::<AppState>();
                let mut index = app_state
                    .vector_index
                    .write()
                    .expect("Failed to acquire vector index write lock");
                for cid in &collection_ids {
                    if let Err(e) = index.build_collection_index(&conn, cid) {
                        tracing::warn!("Failed to build index for collection {}: {}", cid, e);
                    }
                }
                tracing::info!("Built HNSW indices for {} collections", collection_ids.len());
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::collections::create_collection,
            commands::collections::list_collections,
            commands::collections::get_collection,
            commands::collections::update_collection,
            commands::collections::delete_collection,
            commands::settings::get_settings,
            commands::settings::update_setting,
            commands::ollama::check_ollama_connection,
            commands::ollama::list_ollama_models,
            commands::ollama::test_ollama_connection,
            commands::documents::ingest_files,
            commands::documents::list_documents,
            commands::documents::get_document,
            commands::documents::delete_document,
            commands::documents::get_document_chunks,
            commands::documents::get_stats,
            commands::documents::reingest_document,
            commands::documents::reingest_collection,
            commands::documents::add_document_tag,
            commands::documents::remove_document_tag,
            commands::documents::list_all_tags,
            // Search commands
            commands::search::vector_search,
            commands::search::keyword_search,
            commands::search::hybrid_search,
            commands::search::save_search_query,
            commands::search::get_search_history,
            commands::search::clear_search_history,
            commands::search::find_similar_chunks,
            // Chat commands
            commands::chat::send_chat_message,
            commands::chat::create_conversation,
            commands::chat::list_conversations,
            commands::chat::get_conversation_messages,
            commands::chat::get_message_citations,
            commands::chat::delete_conversation,
            commands::chat::rename_conversation,
            commands::chat::cancel_chat_generation,
            commands::chat::delete_last_assistant_message,
            commands::chat::export_conversation_markdown,
            // Metrics commands
            commands::settings::get_metrics,
            // Graph commands
            commands::graph::build_graph,
            commands::graph::get_graph,
            // Audit commands
            commands::audit::get_audit_log,
            commands::audit::export_audit_log,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running VaultMind");
}
