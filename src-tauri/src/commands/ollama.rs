use tauri::State;

use crate::error::AppError;
use crate::models::OllamaModel;
use crate::ollama;
use crate::state::AppState;

#[tauri::command]
pub async fn check_ollama_connection(
    state: State<'_, AppState>,
) -> Result<(bool, String), AppError> {
    let (host, port) = {
        let db = state.db.lock().map_err(|e| AppError::Database(
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                Some(format!("Mutex lock failed: {}", e)),
            ),
        ))?;

        let host: String = db.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_host'",
            [],
            |row| row.get(0),
        )?;

        let port: String = db.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_port'",
            [],
            |row| row.get(0),
        )?;

        (host, port)
    };

    ollama::health_check(&host, &port).await
}

#[tauri::command]
pub async fn list_ollama_models(
    state: State<'_, AppState>,
) -> Result<Vec<OllamaModel>, AppError> {
    let (host, port) = {
        let db = state.db.lock().map_err(|e| AppError::Database(
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                Some(format!("Mutex lock failed: {}", e)),
            ),
        ))?;

        let host: String = db.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_host'",
            [],
            |row| row.get(0),
        )?;

        let port: String = db.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_port'",
            [],
            |row| row.get(0),
        )?;

        (host, port)
    };

    ollama::list_models(&host, &port).await
}

#[tauri::command]
pub async fn test_ollama_connection(
    host: String,
    port: String,
) -> Result<(bool, String), AppError> {
    ollama::health_check(&host, &port).await
}
