use rusqlite::Connection;

use crate::error::AppError;

/// Migration v3: Add entities and entity_mentions tables for Named Entity Recognition.
fn migrate_v3(conn: &Connection) -> Result<(), AppError> {
    let tx = conn.unchecked_transaction()?;
    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS entities (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
            first_seen_at TEXT NOT NULL,
            mention_count INTEGER DEFAULT 1,
            metadata TEXT DEFAULT '{}'
        );
        CREATE INDEX IF NOT EXISTS idx_entities_collection ON entities(collection_id);
        CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(entity_type);
        CREATE INDEX IF NOT EXISTS idx_entities_name ON entities(name);
        CREATE TABLE IF NOT EXISTS entity_mentions (
            id TEXT PRIMARY KEY,
            entity_id TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
            chunk_id TEXT NOT NULL REFERENCES chunks(id) ON DELETE CASCADE,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            start_offset INTEGER DEFAULT 0,
            end_offset INTEGER DEFAULT 0,
            context TEXT,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_entity_mentions_entity ON entity_mentions(entity_id);
        CREATE INDEX IF NOT EXISTS idx_entity_mentions_chunk ON entity_mentions(chunk_id);",
    )?;
    set_schema_version(&tx, 3)?;
    tx.commit()?;
    tracing::info!("Migration v3 applied (entities, entity_mentions tables)");
    Ok(())
}

/// Migration v4: Add entity_relationships table for relationship type detection.
fn migrate_v4(conn: &Connection) -> Result<(), AppError> {
    let tx = conn.unchecked_transaction()?;
    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS entity_relationships (
            id TEXT PRIMARY KEY,
            source_entity_id TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
            target_entity_id TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
            relationship_type TEXT NOT NULL,
            confidence REAL DEFAULT 0.0,
            evidence_chunk_id TEXT REFERENCES chunks(id) ON DELETE SET NULL,
            collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_entity_rel_source ON entity_relationships(source_entity_id);
        CREATE INDEX IF NOT EXISTS idx_entity_rel_target ON entity_relationships(target_entity_id);
        CREATE INDEX IF NOT EXISTS idx_entity_rel_collection ON entity_relationships(collection_id);",
    )?;
    set_schema_version(&tx, 4)?;
    tx.commit()?;
    tracing::info!("Migration v4 applied (entity_relationships table)");
    Ok(())
}

/// Migration v5: Add enhanced citation columns for multi-hop RAG.
fn migrate_v5(conn: &Connection) -> Result<(), AppError> {
    let tx = conn.unchecked_transaction()?;

    // Each ALTER TABLE uses .ok() for idempotent re-runs (won't fail if column exists)
    tx.execute_batch("ALTER TABLE citations ADD COLUMN start_char INTEGER DEFAULT 0;").ok();
    tx.execute_batch("ALTER TABLE citations ADD COLUMN end_char INTEGER DEFAULT 0;").ok();
    tx.execute_batch("ALTER TABLE citations ADD COLUMN confidence REAL DEFAULT 0.0;").ok();
    tx.execute_batch("ALTER TABLE citations ADD COLUMN hop_distance INTEGER DEFAULT 0;").ok();

    set_schema_version(&tx, 5)?;
    tx.commit()?;
    tracing::info!("Migration v5 applied (enhanced citation columns for multi-hop RAG)");
    Ok(())
}

#[cfg(test)]
const CURRENT_VERSION: i64 = 5;

/// Run all pending migrations. Called after initial schema creation.
pub fn run_pending(conn: &Connection) -> Result<(), AppError> {
    let version = get_schema_version(conn)?;

    if version < 1 {
        migrate_v1(conn)?;
    }
    if version < 2 {
        migrate_v2(conn)?;
    }
    if version < 3 {
        migrate_v3(conn)?;
    }
    if version < 4 {
        migrate_v4(conn)?;
    }
    if version < 5 {
        migrate_v5(conn)?;
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

/// Migration v2: Add audit_log table for SOC 2 / GDPR compliance.
fn migrate_v2(conn: &Connection) -> Result<(), AppError> {
    let tx = conn.unchecked_transaction()?;

    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS audit_log (
            id TEXT PRIMARY KEY,
            timestamp TEXT NOT NULL,
            action TEXT NOT NULL,
            entity_type TEXT,
            entity_id TEXT,
            details TEXT DEFAULT '{}',
            ip_address TEXT,
            user_agent TEXT DEFAULT 'desktop'
        );
        CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp);
        CREATE INDEX IF NOT EXISTS idx_audit_log_action ON audit_log(action);
        CREATE INDEX IF NOT EXISTS idx_audit_log_entity ON audit_log(entity_type, entity_id);",
    )?;

    set_schema_version(&tx, 2)?;
    tx.commit()?;

    tracing::info!("Migration v2 applied (audit_log table)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup_db() -> Connection {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let pool = db::create_pool(&path).unwrap();
        let conn = pool.get().unwrap();
        // We need to return an owned Connection for tests, so open a new one
        std::mem::forget(dir);
        let conn2 = Connection::open(path.join("vaultmind.db")).unwrap();
        db::configure_connection(&conn2).unwrap();
        drop(conn);
        conn2
    }

    #[test]
    fn test_migration_v1_applies_correctly() {
        let conn = setup_db();

        let version = get_schema_version(&conn).unwrap();
        assert!(version >= 1);

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
    fn test_migration_v2_creates_audit_log() {
        let conn = setup_db();

        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, CURRENT_VERSION);

        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='audit_log'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap();
        assert!(exists, "audit_log table should exist after migration v2");
    }

    #[test]
    fn test_migration_v3_creates_entities_tables() {
        let conn = setup_db();
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, CURRENT_VERSION);
        let entities_exist: bool = conn
            .query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entities'", [], |row| row.get::<_, i64>(0))
            .map(|c| c > 0).unwrap();
        assert!(entities_exist, "entities table should exist after migration v3");
        let mentions_exist: bool = conn
            .query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entity_mentions'", [], |row| row.get::<_, i64>(0))
            .map(|c| c > 0).unwrap();
        assert!(mentions_exist, "entity_mentions table should exist after migration v3");
        let idx_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_entit%'", [], |row| row.get(0))
            .unwrap();
        assert!(idx_count >= 5, "Should have at least 5 entity-related indexes");
    }

    #[test]
    fn test_migration_v4_creates_entity_relationships_table() {
        let conn = setup_db();
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, CURRENT_VERSION);

        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entity_relationships'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap();
        assert!(exists, "entity_relationships table should exist after migration v4");

        // Verify indexes exist
        let idx_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_entity_rel%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(idx_count >= 3, "Should have at least 3 entity_rel indexes");
    }

    #[test]
    fn test_migration_v5_adds_citation_columns() {
        let conn = setup_db();
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, CURRENT_VERSION);

        // Verify all four new columns exist on citations table
        let has_start_char: bool = conn
            .prepare("SELECT start_char FROM citations LIMIT 0")
            .is_ok();
        assert!(has_start_char, "citations.start_char column should exist after migration v5");

        let has_end_char: bool = conn
            .prepare("SELECT end_char FROM citations LIMIT 0")
            .is_ok();
        assert!(has_end_char, "citations.end_char column should exist after migration v5");

        let has_confidence: bool = conn
            .prepare("SELECT confidence FROM citations LIMIT 0")
            .is_ok();
        assert!(has_confidence, "citations.confidence column should exist after migration v5");

        let has_hop_distance: bool = conn
            .prepare("SELECT hop_distance FROM citations LIMIT 0")
            .is_ok();
        assert!(has_hop_distance, "citations.hop_distance column should exist after migration v5");
    }

    #[test]
    fn test_migration_idempotent_on_rerun() {
        let conn = setup_db();

        let version1 = get_schema_version(&conn).unwrap();
        run_pending(&conn).unwrap();
        let version2 = get_schema_version(&conn).unwrap();

        assert_eq!(version1, version2);
        assert_eq!(version2, CURRENT_VERSION);
    }

    #[test]
    fn test_schema_version_updates() {
        let conn = setup_db();
        assert_eq!(get_schema_version(&conn).unwrap(), CURRENT_VERSION);
    }
}
