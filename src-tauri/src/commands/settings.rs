use std::collections::HashMap;

use tauri::State;

use crate::error::AppError;
use crate::state::{get_conn, AppState};

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, AppError> {
    let conn = get_conn(state.inner())?;

    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;

    let settings = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<HashMap<String, String>, _>>()?;

    Ok(settings)
}

#[tauri::command]
pub fn update_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), AppError> {
    let conn = get_conn(state.inner())?;

    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![&key, &value],
    )?;

    let _ = crate::audit::log_audit(
        &conn,
        crate::audit::AuditAction::SettingUpdate,
        Some("setting"),
        Some(&key),
        &serde_json::json!({"key": key, "value": value}),
    );

    Ok(())
}

#[tauri::command]
pub fn get_metrics(
    state: State<'_, AppState>,
) -> Result<crate::metrics::MetricsSnapshot, AppError> {
    Ok(state.inner().metrics.snapshot())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::db;

    fn setup_db() -> rusqlite::Connection {
        let dir = tempfile::tempdir().unwrap();
        let pool = db::create_pool(dir.path()).unwrap();
        let conn = pool.get().unwrap();
        std::mem::forget(dir);
        let path = conn.path().unwrap().to_owned();
        drop(conn);
        let c = rusqlite::Connection::open(path).unwrap();
        db::configure_connection(&c).unwrap();
        c
    }

    fn get_all_settings(conn: &rusqlite::Connection) -> HashMap<String, String> {
        let mut stmt = conn.prepare("SELECT key, value FROM settings").unwrap();
        stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .unwrap()
        .collect::<Result<HashMap<String, String>, _>>()
        .unwrap()
    }

    #[test]
    fn test_get_settings_returns_seeded_defaults() {
        let conn = setup_db();
        let settings = get_all_settings(&conn);

        assert_eq!(settings.get("ollama_host").unwrap(), "localhost");
        assert_eq!(settings.get("ollama_port").unwrap(), "11434");
        assert_eq!(settings.get("embedding_model").unwrap(), "nomic-embed-text");
        assert_eq!(settings.get("chat_model").unwrap(), "llama3.2");
        assert_eq!(settings.get("chunk_size").unwrap(), "512");
        assert_eq!(settings.get("theme").unwrap(), "system");
    }

    #[test]
    fn test_update_setting_persists() {
        let conn = setup_db();

        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params!["theme", "dark"],
        ).unwrap();

        let settings = get_all_settings(&conn);
        assert_eq!(settings.get("theme").unwrap(), "dark");
    }

    #[test]
    fn test_update_setting_creates_new_key() {
        let conn = setup_db();

        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params!["custom_key", "custom_value"],
        ).unwrap();

        let settings = get_all_settings(&conn);
        assert_eq!(settings.get("custom_key").unwrap(), "custom_value");
    }
}
