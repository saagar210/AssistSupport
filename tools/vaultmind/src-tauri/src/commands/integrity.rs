use std::collections::HashMap;

use tauri::State;

use crate::error::AppError;
use crate::integrity;
use crate::state::{get_conn, AppState};

#[tauri::command]
pub fn check_db_integrity(
    state: State<'_, AppState>,
) -> Result<integrity::IntegrityReport, AppError> {
    let conn = get_conn(state.inner())?;
    integrity::check_database_integrity(&conn)
}

/// Get basic database statistics without running full integrity check.
#[tauri::command]
pub fn get_db_stats(state: State<'_, AppState>) -> Result<DbStats, AppError> {
    let conn = get_conn(state.inner())?;

    let table_counts = get_table_counts(&conn)?;
    let page_count: i64 = conn.query_row("PRAGMA page_count", [], |row| row.get(0))?;
    let page_size: i64 = conn.query_row("PRAGMA page_size", [], |row| row.get(0))?;

    Ok(DbStats {
        db_size_bytes: (page_count * page_size) as u64,
        table_counts,
    })
}

fn get_table_counts(conn: &rusqlite::Connection) -> Result<HashMap<String, i64>, AppError> {
    let tables = [
        "collections",
        "documents",
        "chunks",
        "chunk_embeddings",
        "graph_edges",
        "conversations",
        "messages",
        "citations",
        "search_history",
        "audit_log",
        "entities",
        "entity_mentions",
        "entity_relationships",
    ];

    let mut counts = HashMap::new();
    for table in tables {
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap_or(0);
        counts.insert(table.to_string(), count);
    }
    Ok(counts)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DbStats {
    pub db_size_bytes: u64,
    pub table_counts: HashMap<String, i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_pool() -> (
        tempfile::TempDir,
        r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let pool = crate::db::create_pool(dir.path()).unwrap();
        (dir, pool)
    }

    #[test]
    fn test_get_db_stats_returns_table_counts() {
        let (_dir, pool) = setup_pool();
        let conn = pool.get().unwrap();
        let counts = get_table_counts(&conn).unwrap();

        let expected_tables = [
            "collections",
            "documents",
            "chunks",
            "chunk_embeddings",
            "graph_edges",
            "conversations",
            "messages",
            "citations",
            "search_history",
            "audit_log",
            "entities",
            "entity_mentions",
            "entity_relationships",
        ];

        for table in expected_tables {
            assert!(
                counts.contains_key(table),
                "table_counts should include '{table}'"
            );
        }

        // General collection is seeded, so collections count should be >= 1
        assert!(
            *counts.get("collections").unwrap_or(&0) >= 1,
            "collections count should be >= 1 (General seeded)"
        );
    }
}
