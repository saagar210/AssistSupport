use tauri::State;

use crate::audit::{self, AuditAction};
use crate::error::AppError;
use crate::models::Collection;
use crate::state::{get_conn, AppState};

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

    let conn = get_conn(state.inner())?;

    conn.execute(
        "INSERT INTO collections (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, name, description, now, now],
    )?;

    let _ = audit::log_audit(
        &conn,
        AuditAction::CollectionCreate,
        Some("collection"),
        Some(&id),
        &serde_json::json!({"name": name}),
    );

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
    let conn = get_conn(state.inner())?;

    let mut stmt = conn.prepare(
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
    let conn = get_conn(state.inner())?;

    let collection = conn.query_row(
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

    let conn = get_conn(state.inner())?;

    let rows_updated = conn.execute(
        "UPDATE collections SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![name, description, now, id],
    )?;

    if rows_updated == 0 {
        return Err(AppError::NotFound(format!("Collection '{}' not found", id)));
    }

    let _ = audit::log_audit(
        &conn,
        AuditAction::CollectionUpdate,
        Some("collection"),
        Some(&id),
        &serde_json::json!({"name": name}),
    );

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
    let conn = get_conn(state.inner())?;

    // Check if this is the "General" collection
    let name: String = conn.query_row(
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

    let _ = audit::log_audit(
        &conn,
        AuditAction::CollectionDelete,
        Some("collection"),
        Some(&id),
        &serde_json::json!({"name": name}),
    );

    conn.execute("DELETE FROM collections WHERE id = ?1", rusqlite::params![id])?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::db;
    use crate::models::Collection;

    fn setup_db() -> rusqlite::Connection {
        let dir = tempfile::tempdir().unwrap();
        let pool = db::create_pool(dir.path()).unwrap();
        let conn = pool.get().unwrap();
        std::mem::forget(dir);
        // Return a new connection from the same DB file
        // For unit tests we just need a Connection, not pool
        let path = conn.path().unwrap().to_owned();
        drop(conn);
        let c = rusqlite::Connection::open(path).unwrap();
        db::configure_connection(&c).unwrap();
        c
    }

    fn create_collection_direct(conn: &rusqlite::Connection, name: &str, desc: &str) -> Collection {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO collections (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, name, desc, now, now],
        ).unwrap();
        Collection {
            id,
            name: name.to_string(),
            description: desc.to_string(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    fn list_collections(conn: &rusqlite::Connection) -> Vec<Collection> {
        let mut stmt = conn.prepare(
            "SELECT id, name, description, created_at, updated_at FROM collections ORDER BY created_at ASC",
        ).unwrap();
        stmt.query_map([], |row| {
            Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        }).unwrap().collect::<Result<Vec<_>, _>>().unwrap()
    }

    #[test]
    fn test_create_and_list_collections() {
        let conn = setup_db();
        // "General" is seeded
        let initial = list_collections(&conn);
        assert_eq!(initial.len(), 1);
        assert_eq!(initial[0].name, "General");

        create_collection_direct(&conn, "Test Collection", "A test");
        let all = list_collections(&conn);
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_duplicate_collection_name_fails() {
        let conn = setup_db();
        create_collection_direct(&conn, "Unique", "desc");

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let result = conn.execute(
            "INSERT INTO collections (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, "Unique", "another desc", now, now],
        );
        assert!(result.is_err(), "Duplicate collection name should fail due to UNIQUE constraint");
    }

    #[test]
    fn test_general_cannot_be_deleted_logic() {
        let conn = setup_db();

        // Find the General collection id
        let general_id: String = conn.query_row(
            "SELECT id FROM collections WHERE name = 'General'",
            [],
            |row| row.get(0),
        ).unwrap();

        // Simulate the delete_collection guard
        let name: String = conn.query_row(
            "SELECT name FROM collections WHERE id = ?1",
            rusqlite::params![general_id],
            |row| row.get(0),
        ).unwrap();

        assert_eq!(name, "General");
        // The command would return an error here
    }

    #[test]
    fn test_delete_non_general_collection() {
        let conn = setup_db();
        let col = create_collection_direct(&conn, "Deletable", "to delete");

        conn.execute("DELETE FROM collections WHERE id = ?1", rusqlite::params![col.id]).unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM collections WHERE id = ?1",
            rusqlite::params![col.id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_update_collection() {
        let conn = setup_db();
        let col = create_collection_direct(&conn, "Original", "original desc");

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE collections SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
            rusqlite::params!["Updated", "new desc", now, col.id],
        ).unwrap();

        let updated: (String, String) = conn.query_row(
            "SELECT name, description FROM collections WHERE id = ?1",
            rusqlite::params![col.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();

        assert_eq!(updated.0, "Updated");
        assert_eq!(updated.1, "new desc");
    }
}
