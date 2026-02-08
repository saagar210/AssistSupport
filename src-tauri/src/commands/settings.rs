use std::collections::HashMap;

use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, AppError> {
    let db = crate::state::lock_db(&state)?;

    let mut stmt = db.prepare("SELECT key, value FROM settings")?;

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
    let db = crate::state::lock_db(&state)?;

    db.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::db;

    fn setup_db() -> rusqlite::Connection {
        let dir = tempfile::tempdir().unwrap();
        db::initialize(dir.path()).unwrap()
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
