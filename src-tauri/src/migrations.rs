use rusqlite::Connection;

use crate::error::AppError;

/// Current schema version. Increment this when adding new migrations.
#[cfg(test)]
const CURRENT_VERSION: i64 = 1;

/// Run all pending migrations. Called after initial schema creation.
pub fn run_pending(conn: &Connection) -> Result<(), AppError> {
    let version = get_schema_version(conn)?;

    if version < 1 {
        migrate_v1(conn)?;
    }

    Ok(())
}

fn get_schema_version(conn: &Connection) -> Result<i64, AppError> {
    let result = conn.query_row(
        "SELECT value FROM settings WHERE key = 'schema_version'",
        [],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(val) => val.parse::<i64>().map_err(|_| {
            AppError::Validation("Invalid schema_version value".to_string())
        }),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
        Err(e) => Err(AppError::Database(e)),
    }
}

fn set_schema_version(conn: &Connection, version: i64) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('schema_version', ?1)",
        rusqlite::params![version.to_string()],
    )?;
    Ok(())
}

/// Migration v1: Add tags, search_history, and performance indexes.
fn migrate_v1(conn: &Connection) -> Result<(), AppError> {
    let tx = conn.unchecked_transaction()?;

    // Tags support on documents
    tx.execute_batch(
        "ALTER TABLE documents ADD COLUMN tags TEXT DEFAULT '[]';",
    ).ok(); // OK if column already exists (idempotent re-run)

    // Search history table
    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS search_history (
            id TEXT PRIMARY KEY,
            collection_id TEXT NOT NULL,
            query TEXT NOT NULL,
            result_count INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
        );",
    )?;

    // Performance indexes
    tx.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_chunks_document_id ON chunks(document_id);
         CREATE INDEX IF NOT EXISTS idx_messages_conversation_id ON messages(conversation_id);
         CREATE INDEX IF NOT EXISTS idx_citations_message_id ON citations(message_id);
         CREATE INDEX IF NOT EXISTS idx_graph_edges_collection ON graph_edges(collection_id);
         CREATE INDEX IF NOT EXISTS idx_documents_collection ON documents(collection_id);",
    )?;

    set_schema_version(&tx, 1)?;
    tx.commit()?;

    tracing::info!("Migration v1 applied (tags, search_history, indexes)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup_db() -> Connection {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let conn = db::initialize(&path).unwrap();
        // Leak dir to keep temp directory alive
        std::mem::forget(dir);
        conn
    }

    #[test]
    fn test_migration_v1_applies_correctly() {
        let conn = setup_db();

        // db::initialize calls run_pending, so migration should already be at v1
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, 1);

        // search_history table should exist
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='search_history'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap();
        assert!(exists, "search_history table should exist after migration v1");

        // tags column should exist on documents
        let has_tags: bool = conn
            .prepare("SELECT tags FROM documents LIMIT 0")
            .is_ok();
        assert!(has_tags, "documents.tags column should exist after migration v1");
    }

    #[test]
    fn test_migration_idempotent_on_rerun() {
        let conn = setup_db();

        // Already at v1 from initialize. Run again — should not error.
        let version1 = get_schema_version(&conn).unwrap();
        run_pending(&conn).unwrap();
        let version2 = get_schema_version(&conn).unwrap();

        assert_eq!(version1, version2);
        assert_eq!(version2, CURRENT_VERSION);
    }

    #[test]
    fn test_schema_version_updates() {
        let conn = setup_db();
        // initialize already ran migrations, so version should be at CURRENT_VERSION
        assert_eq!(get_schema_version(&conn).unwrap(), CURRENT_VERSION);
    }
}
