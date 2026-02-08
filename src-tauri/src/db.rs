use std::path::Path;
use std::time::Duration;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;

use crate::error::AppError;

/// Configure a SQLite connection with required PRAGMAs.
/// Called for every connection checked out of the pool.
pub fn configure_connection(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch("PRAGMA busy_timeout = 5000;")?;
    Ok(())
}

/// Create and return an r2d2 connection pool for the SQLite database.
pub fn create_pool(app_data_dir: &Path) -> Result<Pool<SqliteConnectionManager>, AppError> {
    std::fs::create_dir_all(app_data_dir)?;

    let db_path = app_data_dir.join("vaultmind.db");
    let manager = SqliteConnectionManager::file(db_path);

    let pool = Pool::builder()
        .max_size(8)
        .min_idle(Some(2))
        .connection_timeout(Duration::from_secs(5))
        .connection_customizer(Box::new(ConnectionCustomizer))
        .build(manager)
        .map_err(|e| AppError::LockFailed(format!("Failed to create connection pool: {}", e)))?;

    // Initialize schema using first connection
    let conn = pool.get().map_err(|e| {
        AppError::LockFailed(format!("Failed to get initial connection: {}", e))
    })?;
    initialize_schema(&conn)?;

    Ok(pool)
}

/// r2d2 connection customizer that applies PRAGMAs on each new connection.
#[derive(Debug)]
struct ConnectionCustomizer;

impl r2d2::CustomizeConnection<Connection, rusqlite::Error> for ConnectionCustomizer {
    fn on_acquire(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        configure_connection(conn)
    }
}

/// Initialize the database schema and seed data.
fn initialize_schema(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS collections (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
            filename TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_type TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            file_hash TEXT NOT NULL,
            title TEXT NOT NULL,
            author TEXT,
            page_count INTEGER,
            word_count INTEGER DEFAULT 0,
            chunk_count INTEGER DEFAULT 0,
            status TEXT DEFAULT 'pending',
            error_message TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS chunks (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            chunk_index INTEGER NOT NULL,
            start_offset INTEGER DEFAULT 0,
            end_offset INTEGER DEFAULT 0,
            page_number INTEGER,
            section_title TEXT,
            token_count INTEGER DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
            content,
            chunk_id,
            document_id,
            collection_id
        );

        CREATE TABLE IF NOT EXISTS graph_edges (
            id TEXT PRIMARY KEY,
            source_chunk_id TEXT NOT NULL,
            target_chunk_id TEXT NOT NULL,
            collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
            weight REAL DEFAULT 0.0,
            relationship_type TEXT DEFAULT 'semantic',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
            title TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS citations (
            id TEXT PRIMARY KEY,
            message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
            chunk_id TEXT NOT NULL,
            document_id TEXT NOT NULL,
            document_title TEXT NOT NULL,
            section_title TEXT,
            page_number INTEGER,
            relevance_score REAL DEFAULT 0.0,
            snippet TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS chunk_embeddings (
            chunk_id TEXT PRIMARY KEY,
            collection_id TEXT NOT NULL,
            document_id TEXT NOT NULL,
            embedding BLOB NOT NULL,
            content_preview TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_collection
            ON chunk_embeddings(collection_id);
        CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_document
            ON chunk_embeddings(document_id);
        ",
    )?;

    // Seed default settings
    conn.execute_batch(
        "
        INSERT OR IGNORE INTO settings (key, value) VALUES ('ollama_host', 'localhost');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('ollama_port', '11434');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('embedding_model', 'nomic-embed-text');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('chat_model', 'llama3.2');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('chunk_size', '512');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('chunk_overlap', '64');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('theme', 'system');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('vector_top_k', '20');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('keyword_top_k', '20');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('context_chunks', '5');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('similarity_threshold', '0.75');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('rrf_k', '60');
        ",
    )?;

    // Seed default "General" collection
    let now = chrono::Utc::now().to_rfc3339();
    let general_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT OR IGNORE INTO collections (id, name, description, created_at, updated_at) VALUES (?1, 'General', '', ?2, ?3)",
        rusqlite::params![general_id, now, now],
    )?;

    // Run pending migrations
    crate::migrations::run_pending(conn)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_db_dir() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        (dir, path)
    }

    #[test]
    fn test_initialize_creates_db_file() {
        let (_dir, path) = temp_db_dir();
        let _pool = create_pool(&path).unwrap();
        assert!(path.join("vaultmind.db").exists());
    }

    #[test]
    fn test_initialize_creates_all_tables() {
        let (_dir, path) = temp_db_dir();
        let pool = create_pool(&path).unwrap();
        let conn = pool.get().unwrap();

        let expected_tables = vec![
            "collections",
            "documents",
            "chunks",
            "graph_edges",
            "conversations",
            "messages",
            "citations",
            "settings",
            "chunk_embeddings",
        ];

        for table in expected_tables {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    rusqlite::params![table],
                    |row| row.get::<_, i64>(0),
                )
                .map(|c| c > 0)
                .unwrap();
            assert!(exists, "Table '{}' should exist", table);
        }

        // Also check FTS virtual table
        let fts_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='chunks_fts'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap();
        assert!(fts_exists, "FTS table 'chunks_fts' should exist");
    }

    #[test]
    fn test_general_collection_seeded() {
        let (_dir, path) = temp_db_dir();
        let pool = create_pool(&path).unwrap();
        let conn = pool.get().unwrap();

        let name: String = conn
            .query_row(
                "SELECT name FROM collections WHERE name = 'General'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(name, "General");
    }

    #[test]
    fn test_default_settings_seeded() {
        let (_dir, path) = temp_db_dir();
        let pool = create_pool(&path).unwrap();
        let conn = pool.get().unwrap();

        let expected_settings = vec![
            ("ollama_host", "localhost"),
            ("ollama_port", "11434"),
            ("embedding_model", "nomic-embed-text"),
            ("chat_model", "llama3.2"),
            ("chunk_size", "512"),
            ("chunk_overlap", "64"),
            ("theme", "system"),
            ("vector_top_k", "20"),
            ("keyword_top_k", "20"),
            ("context_chunks", "5"),
            ("similarity_threshold", "0.75"),
            ("rrf_k", "60"),
        ];

        for (key, expected_value) in expected_settings {
            let value: String = conn
                .query_row(
                    "SELECT value FROM settings WHERE key = ?1",
                    rusqlite::params![key],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(value, expected_value, "Setting '{}' mismatch", key);
        }
    }

    #[test]
    fn test_initialize_idempotent() {
        let (_dir, path) = temp_db_dir();
        let _pool1 = create_pool(&path).unwrap();
        let pool2 = create_pool(&path).unwrap();
        let conn2 = pool2.get().unwrap();

        // Should still have exactly one General collection
        let count: i64 = conn2
            .query_row(
                "SELECT COUNT(*) FROM collections WHERE name = 'General'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_pool_returns_connections() {
        let (_dir, path) = temp_db_dir();
        let pool = create_pool(&path).unwrap();

        // Should be able to get multiple connections
        let _conn1 = pool.get().unwrap();
        let _conn2 = pool.get().unwrap();
        let _conn3 = pool.get().unwrap();
    }

    #[test]
    fn test_connection_has_wal_mode() {
        let (_dir, path) = temp_db_dir();
        let pool = create_pool(&path).unwrap();
        let conn = pool.get().unwrap();

        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode, "wal");
    }

    #[test]
    fn test_connection_has_foreign_keys() {
        let (_dir, path) = temp_db_dir();
        let pool = create_pool(&path).unwrap();
        let conn = pool.get().unwrap();

        let fk: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk, 1);
    }
}
