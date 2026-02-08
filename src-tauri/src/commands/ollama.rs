use tauri::State;

use crate::error::AppError;
use crate::model_registry::{self, RecommendedModel};
use crate::models::OllamaModel;
use crate::ollama;
use crate::ollama::ModelInfo;
use crate::state::{get_conn, AppState};

#[tauri::command]
pub async fn check_ollama_connection(
    state: State<'_, AppState>,
) -> Result<(bool, String), AppError> {
    let (host, port) = {
        let conn = get_conn(state.inner())?;

        let host: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_host'",
            [],
            |row| row.get(0),
        )?;

        let port: String = conn.query_row(
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
        let conn = get_conn(state.inner())?;

        let host: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_host'",
            [],
            |row| row.get(0),
        )?;

        let port: String = conn.query_row(
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

#[tauri::command]
pub async fn pull_ollama_model(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    model_name: String,
) -> Result<(), AppError> {
    let (host, port) = {
        let conn = get_conn(state.inner())?;

        let host: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_host'",
            [],
            |row| row.get(0),
        )?;

        let port: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_port'",
            [],
            |row| row.get(0),
        )?;

        (host, port)
    };

    ollama::pull_model(&host, &port, &model_name, &app_handle).await
}

#[tauri::command]
pub async fn delete_ollama_model(
    state: State<'_, AppState>,
    model_name: String,
) -> Result<(), AppError> {
    let (host, port) = {
        let conn = get_conn(state.inner())?;

        let host: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_host'",
            [],
            |row| row.get(0),
        )?;

        let port: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_port'",
            [],
            |row| row.get(0),
        )?;

        (host, port)
    };

    ollama::delete_model(&host, &port, &model_name).await
}

#[tauri::command]
pub async fn show_ollama_model(
    state: State<'_, AppState>,
    model_name: String,
) -> Result<ModelInfo, AppError> {
    let (host, port) = {
        let conn = get_conn(state.inner())?;

        let host: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_host'",
            [],
            |row| row.get(0),
        )?;

        let port: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_port'",
            [],
            |row| row.get(0),
        )?;

        (host, port)
    };

    ollama::show_model(&host, &port, &model_name).await
}

#[tauri::command]
pub fn get_recommended_models() -> Vec<RecommendedModel> {
    model_registry::get_recommended_models()
}

#[tauri::command]
pub fn get_models_by_use_case(use_case: String) -> Vec<RecommendedModel> {
    model_registry::get_models_by_use_case(&use_case)
}
