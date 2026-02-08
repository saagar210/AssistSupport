use tauri::State;

use crate::error::AppError;
use crate::models::Collection;
use crate::state::AppState;

#[tauri::command]
pub fn create_collection(
    state: State<'_, AppState>,
    name: String,
    description: String,
) -> Result<Collection, AppError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Validation("Collection name cannot be empty".into()));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let db = state.db.lock().map_err(|e| AppError::Database(
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
            Some(format!("Mutex lock failed: {}", e)),
        ),
    ))?;

    db.execute(
        "INSERT INTO collections (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, name, description, now, now],
    )?;

    Ok(Collection {
        id,
        name,
        description,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn list_collections(state: State<'_, AppState>) -> Result<Vec<Collection>, AppError> {
    let db = state.db.lock().map_err(|e| AppError::Database(
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
            Some(format!("Mutex lock failed: {}", e)),
        ),
    ))?;

    let mut stmt = db.prepare(
        "SELECT id, name, description, created_at, updated_at FROM collections ORDER BY created_at ASC",
    )?;

    let collections = stmt
        .query_map([], |row| {
            Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(collections)
}

#[tauri::command]
pub fn get_collection(state: State<'_, AppState>, id: String) -> Result<Collection, AppError> {
    let db = state.db.lock().map_err(|e| AppError::Database(
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
            Some(format!("Mutex lock failed: {}", e)),
        ),
    ))?;

    let collection = db.query_row(
        "SELECT id, name, description, created_at, updated_at FROM collections WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        },
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("Collection '{}' not found", id)),
        other => AppError::Database(other),
    })?;

    Ok(collection)
}

#[tauri::command]
pub fn update_collection(
    state: State<'_, AppState>,
    id: String,
    name: String,
    description: String,
) -> Result<Collection, AppError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Validation("Collection name cannot be empty".into()));
    }

    let now = chrono::Utc::now().to_rfc3339();

    let db = state.db.lock().map_err(|e| AppError::Database(
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
            Some(format!("Mutex lock failed: {}", e)),
        ),
    ))?;

    let rows_updated = db.execute(
        "UPDATE collections SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![name, description, now, id],
    )?;

    if rows_updated == 0 {
        return Err(AppError::NotFound(format!("Collection '{}' not found", id)));
    }

    Ok(Collection {
        id,
        name,
        description,
        created_at: String::new(), // Will be fetched if needed
        updated_at: now,
    })
}

#[tauri::command]
pub fn delete_collection(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|e| AppError::Database(
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
            Some(format!("Mutex lock failed: {}", e)),
        ),
    ))?;

    // Check if this is the "General" collection
    let name: String = db.query_row(
        "SELECT name FROM collections WHERE id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("Collection '{}' not found", id)),
        other => AppError::Database(other),
    })?;

    if name == "General" {
        return Err(AppError::Validation("Cannot delete the default 'General' collection".into()));
    }

    db.execute("DELETE FROM collections WHERE id = ?1", rusqlite::params![id])?;

    Ok(())
}
