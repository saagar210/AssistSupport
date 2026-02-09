use std::collections::HashMap;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub id: String,
    pub entity_type: String,
    pub retention_days: i64,
    pub auto_delete: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub id: String,
    pub consent_type: String,
    pub granted: bool,
    pub granted_at: String,
    pub revoked_at: Option<String>,
}

/// Export all user data as JSON strings (one per table).
/// Returns a HashMap of table_name -> JSON array string.
pub fn export_all_data(conn: &Connection) -> Result<HashMap<String, String>, AppError> {
    let tables = [
        "collections",
        "documents",
        "conversations",
        "messages",
        "search_history",
        "settings",
        "audit_log",
    ];

    let mut result = HashMap::new();

    for table in &tables {
        let mut stmt = conn.prepare(&format!("SELECT * FROM {}", table))?;
        let column_count = stmt.column_count();
        let column_names: Vec<String> = (0..column_count)
            .map(|i| stmt.column_name(i).unwrap_or("unknown").to_string())
            .collect();

        let rows: Vec<serde_json::Value> = stmt
            .query_map([], |row| {
                let mut obj = serde_json::Map::new();
                for (i, name) in column_names.iter().enumerate() {
                    let val: rusqlite::types::Value = row.get(i)?;
                    let json_val = match val {
                        rusqlite::types::Value::Null => serde_json::Value::Null,
                        rusqlite::types::Value::Integer(n) => {
                            serde_json::Value::Number(serde_json::Number::from(n))
                        }
                        rusqlite::types::Value::Real(f) => serde_json::Number::from_f64(f)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Null),
                        rusqlite::types::Value::Text(s) => serde_json::Value::String(s),
                        rusqlite::types::Value::Blob(b) => {
                            serde_json::Value::String(format!("<blob: {} bytes>", b.len()))
                        }
                    };
                    obj.insert(name.clone(), json_val);
                }
                Ok(serde_json::Value::Object(obj))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let json_str = serde_json::to_string(&rows)
            .map_err(|e| AppError::Parse(format!("Failed to serialize {}: {}", table, e)))?;
        result.insert(table.to_string(), json_str);
    }

    Ok(result)
}

/// Erase a specific document and ALL related data (chunks, embeddings, FTS, graph edges, citations, entity mentions).
pub fn erase_document(conn: &Connection, document_id: &str) -> Result<(), AppError> {
    let tx = conn.unchecked_transaction()?;

    // 1. Delete from chunks_fts WHERE document_id = ?
    tx.execute(
        "DELETE FROM chunks_fts WHERE document_id = ?1",
        rusqlite::params![document_id],
    )?;

    // 2. Delete from chunk_embeddings WHERE document_id = ?
    tx.execute(
        "DELETE FROM chunk_embeddings WHERE document_id = ?1",
        rusqlite::params![document_id],
    )?;

    // 3. Delete from entity_mentions WHERE document_id = ?
    tx.execute(
        "DELETE FROM entity_mentions WHERE document_id = ?1",
        rusqlite::params![document_id],
    )?;

    // 4. Delete graph_edges referencing chunks from this document
    tx.execute(
        "DELETE FROM graph_edges WHERE source_chunk_id IN (SELECT id FROM chunks WHERE document_id = ?1)
         OR target_chunk_id IN (SELECT id FROM chunks WHERE document_id = ?1)",
        rusqlite::params![document_id],
    )?;

    // 5. Delete citations that reference chunks from this document
    tx.execute(
        "DELETE FROM citations WHERE chunk_id IN (SELECT id FROM chunks WHERE document_id = ?1)",
        rusqlite::params![document_id],
    )?;

    // 6. Delete from chunks WHERE document_id = ?
    tx.execute(
        "DELETE FROM chunks WHERE document_id = ?1",
        rusqlite::params![document_id],
    )?;

    // 7. Delete from documents WHERE id = ?
    tx.execute(
        "DELETE FROM documents WHERE id = ?1",
        rusqlite::params![document_id],
    )?;

    tx.commit()?;

    // VACUUM to reclaim space (must be outside transaction)
    conn.execute_batch("VACUUM")?;

    Ok(())
}

/// Erase an entire collection and ALL related data.
pub fn erase_collection(conn: &Connection, collection_id: &str) -> Result<(), AppError> {
    let tx = conn.unchecked_transaction()?;

    // Delete entity_mentions for documents in this collection
    tx.execute(
        "DELETE FROM entity_mentions WHERE document_id IN (SELECT id FROM documents WHERE collection_id = ?1)",
        rusqlite::params![collection_id],
    )?;

    // Delete entity_relationships for this collection
    tx.execute(
        "DELETE FROM entity_relationships WHERE collection_id = ?1",
        rusqlite::params![collection_id],
    )?;

    // Delete entities for this collection
    tx.execute(
        "DELETE FROM entities WHERE collection_id = ?1",
        rusqlite::params![collection_id],
    )?;

    // Delete chunks_fts for documents in this collection
    tx.execute(
        "DELETE FROM chunks_fts WHERE collection_id = ?1",
        rusqlite::params![collection_id],
    )?;

    // Delete chunk_embeddings for this collection
    tx.execute(
        "DELETE FROM chunk_embeddings WHERE collection_id = ?1",
        rusqlite::params![collection_id],
    )?;

    // Delete graph_edges for this collection
    tx.execute(
        "DELETE FROM graph_edges WHERE collection_id = ?1",
        rusqlite::params![collection_id],
    )?;

    // Delete citations via messages in conversations for this collection
    tx.execute(
        "DELETE FROM citations WHERE message_id IN (
            SELECT m.id FROM messages m
            JOIN conversations c ON m.conversation_id = c.id
            WHERE c.collection_id = ?1
        )",
        rusqlite::params![collection_id],
    )?;

    // Delete messages in conversations for this collection
    tx.execute(
        "DELETE FROM messages WHERE conversation_id IN (
            SELECT id FROM conversations WHERE collection_id = ?1
        )",
        rusqlite::params![collection_id],
    )?;

    // Delete conversations for this collection
    tx.execute(
        "DELETE FROM conversations WHERE collection_id = ?1",
        rusqlite::params![collection_id],
    )?;

    // Delete chunks for documents in this collection
    tx.execute(
        "DELETE FROM chunks WHERE collection_id = ?1",
        rusqlite::params![collection_id],
    )?;

    // Delete documents in this collection
    tx.execute(
        "DELETE FROM documents WHERE collection_id = ?1",
        rusqlite::params![collection_id],
    )?;

    // Delete the collection itself
    tx.execute(
        "DELETE FROM collections WHERE id = ?1",
        rusqlite::params![collection_id],
    )?;

    tx.commit()?;

    conn.execute_batch("VACUUM")?;

    Ok(())
}

/// Erase ALL user data. Nuclear option.
pub fn erase_all_data(conn: &Connection) -> Result<(), AppError> {
    let tx = conn.unchecked_transaction()?;

    // Delete in FK-safe order
    let tables = [
        "entity_mentions",
        "entity_relationships",
        "entities",
        "citations",
        "messages",
        "conversations",
        "chunks_fts",
        "chunk_embeddings",
        "graph_edges",
        "chunks",
        "documents",
        "collections",
        "search_history",
        "audit_log",
        "data_retention_policies",
        "consent_records",
    ];

    for table in &tables {
        tx.execute(&format!("DELETE FROM {}", table), [])?;
    }

    // Reset settings to defaults only (delete all, re-seed)
    tx.execute("DELETE FROM settings", [])?;
    tx.execute_batch(
        "INSERT INTO settings (key, value) VALUES ('ollama_host', 'localhost');
         INSERT INTO settings (key, value) VALUES ('ollama_port', '11434');
         INSERT INTO settings (key, value) VALUES ('embedding_model', 'nomic-embed-text');
         INSERT INTO settings (key, value) VALUES ('chat_model', 'llama3.2');
         INSERT INTO settings (key, value) VALUES ('chunk_size', '512');
         INSERT INTO settings (key, value) VALUES ('chunk_overlap', '64');
         INSERT INTO settings (key, value) VALUES ('theme', 'system');
         INSERT INTO settings (key, value) VALUES ('vector_top_k', '20');
         INSERT INTO settings (key, value) VALUES ('keyword_top_k', '20');
         INSERT INTO settings (key, value) VALUES ('context_chunks', '5');
         INSERT INTO settings (key, value) VALUES ('similarity_threshold', '0.75');
         INSERT INTO settings (key, value) VALUES ('rrf_k', '60');
         INSERT INTO settings (key, value) VALUES ('context_token_budget', '4096');
         INSERT INTO settings (key, value) VALUES ('history_token_budget', '2048');",
    )?;

    tx.commit()?;

    conn.execute_batch("VACUUM")?;

    Ok(())
}

/// Enforce retention policies - delete data older than retention period.
/// Returns count of rows deleted.
pub fn enforce_retention_policies(conn: &Connection) -> Result<usize, AppError> {
    let mut stmt = conn.prepare(
        "SELECT entity_type, retention_days FROM data_retention_policies WHERE auto_delete = 1 AND retention_days > 0",
    )?;

    let policies: Vec<(String, i64)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(AppError::Database)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::Database)?;

    let mut total_deleted: usize = 0;

    for (entity_type, retention_days) in &policies {
        let cutoff = chrono::Utc::now()
            - chrono::Duration::days(*retention_days);
        let cutoff_str = cutoff.to_rfc3339();

        let deleted = match entity_type.as_str() {
            "search_history" => conn.execute(
                "DELETE FROM search_history WHERE created_at < ?1",
                rusqlite::params![cutoff_str],
            )?,
            "audit_log" => conn.execute(
                "DELETE FROM audit_log WHERE timestamp < ?1",
                rusqlite::params![cutoff_str],
            )?,
            _ => 0,
        };

        total_deleted += deleted;
    }

    Ok(total_deleted)
}

/// Get all retention policies.
pub fn get_retention_policies(conn: &Connection) -> Result<Vec<RetentionPolicy>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, entity_type, retention_days, auto_delete, created_at, updated_at FROM data_retention_policies ORDER BY entity_type",
    )?;

    let policies = stmt
        .query_map([], |row| {
            Ok(RetentionPolicy {
                id: row.get(0)?,
                entity_type: row.get(1)?,
                retention_days: row.get(2)?,
                auto_delete: row.get::<_, i64>(3)? != 0,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(policies)
}

/// Update a retention policy.
pub fn update_retention_policy(
    conn: &Connection,
    id: &str,
    retention_days: i64,
    auto_delete: bool,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    let rows_affected = conn.execute(
        "UPDATE data_retention_policies SET retention_days = ?1, auto_delete = ?2, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![retention_days, auto_delete as i64, now, id],
    )?;

    if rows_affected == 0 {
        return Err(AppError::NotFound(format!(
            "Retention policy not found: {}",
            id
        )));
    }

    Ok(())
}

/// Record user consent.
pub fn record_consent(
    conn: &Connection,
    consent_type: &str,
    granted: bool,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();

    if granted {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO consent_records (id, consent_type, granted, granted_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id, consent_type, granted as i64, now],
        )?;
    } else {
        // Revoke: update the most recent granted record for this type
        conn.execute(
            "UPDATE consent_records SET granted = 0, revoked_at = ?1 WHERE consent_type = ?2 AND granted = 1 AND revoked_at IS NULL",
            rusqlite::params![now, consent_type],
        )?;
    }

    Ok(())
}

/// Get all consent records.
pub fn get_consent_records(conn: &Connection) -> Result<Vec<ConsentRecord>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, consent_type, granted, granted_at, revoked_at FROM consent_records ORDER BY granted_at DESC",
    )?;

    let records = stmt
        .query_map([], |row| {
            Ok(ConsentRecord {
                id: row.get(0)?,
                consent_type: row.get(1)?,
                granted: row.get::<_, i64>(2)? != 0,
                granted_at: row.get(3)?,
                revoked_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// Create an in-memory SQLite DB with the full schema (all tables up to v6).
    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("Failed to open in-memory DB");
        conn.execute_batch("PRAGMA foreign_keys = ON;").expect("PRAGMA failed");

        conn.execute_batch(
            "CREATE TABLE collections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE documents (
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
                tags TEXT DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE chunks (
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

            CREATE VIRTUAL TABLE chunks_fts USING fts5(
                content,
                chunk_id,
                document_id,
                collection_id
            );

            CREATE TABLE graph_edges (
                id TEXT PRIMARY KEY,
                source_chunk_id TEXT NOT NULL,
                target_chunk_id TEXT NOT NULL,
                collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
                weight REAL DEFAULT 0.0,
                relationship_type TEXT DEFAULT 'semantic',
                created_at TEXT NOT NULL
            );

            CREATE TABLE conversations (
                id TEXT PRIMARY KEY,
                collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
                title TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE citations (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
                chunk_id TEXT NOT NULL,
                document_id TEXT NOT NULL,
                document_title TEXT NOT NULL,
                section_title TEXT,
                page_number INTEGER,
                relevance_score REAL DEFAULT 0.0,
                snippet TEXT NOT NULL,
                start_char INTEGER DEFAULT 0,
                end_char INTEGER DEFAULT 0,
                confidence REAL DEFAULT 0.0,
                hop_distance INTEGER DEFAULT 0
            );

            CREATE TABLE settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE chunk_embeddings (
                chunk_id TEXT PRIMARY KEY,
                collection_id TEXT NOT NULL,
                document_id TEXT NOT NULL,
                embedding BLOB NOT NULL,
                content_preview TEXT
            );

            CREATE TABLE search_history (
                id TEXT PRIMARY KEY,
                collection_id TEXT NOT NULL,
                query TEXT NOT NULL,
                result_count INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
            );

            CREATE TABLE audit_log (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                action TEXT NOT NULL,
                entity_type TEXT,
                entity_id TEXT,
                details TEXT DEFAULT '{}',
                ip_address TEXT,
                user_agent TEXT DEFAULT 'desktop'
            );

            CREATE TABLE entities (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
                first_seen_at TEXT NOT NULL,
                mention_count INTEGER DEFAULT 1,
                metadata TEXT DEFAULT '{}'
            );

            CREATE TABLE entity_mentions (
                id TEXT PRIMARY KEY,
                entity_id TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
                chunk_id TEXT NOT NULL REFERENCES chunks(id) ON DELETE CASCADE,
                document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
                start_offset INTEGER DEFAULT 0,
                end_offset INTEGER DEFAULT 0,
                context TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE entity_relationships (
                id TEXT PRIMARY KEY,
                source_entity_id TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
                target_entity_id TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
                relationship_type TEXT NOT NULL,
                confidence REAL DEFAULT 0.0,
                evidence_chunk_id TEXT REFERENCES chunks(id) ON DELETE SET NULL,
                collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
                created_at TEXT NOT NULL
            );

            CREATE TABLE data_retention_policies (
                id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                retention_days INTEGER NOT NULL,
                auto_delete INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE consent_records (
                id TEXT PRIMARY KEY,
                consent_type TEXT NOT NULL,
                granted INTEGER NOT NULL,
                granted_at TEXT NOT NULL,
                revoked_at TEXT
            );",
        )
        .expect("Failed to create schema");

        // Seed default settings
        conn.execute_batch(
            "INSERT INTO settings (key, value) VALUES ('ollama_host', 'localhost');
             INSERT INTO settings (key, value) VALUES ('theme', 'system');",
        )
        .expect("Failed to seed settings");

        conn
    }

    /// Helper: insert a collection and return its id.
    fn insert_collection(conn: &Connection, name: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO collections (id, name, description, created_at, updated_at) VALUES (?1, ?2, '', ?3, ?4)",
            rusqlite::params![id, name, now, now],
        )
        .expect("insert collection");
        id
    }

    /// Helper: insert a document and return its id.
    fn insert_document(conn: &Connection, collection_id: &str, filename: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO documents (id, collection_id, filename, file_path, file_type, file_size, file_hash, title, created_at, updated_at)
             VALUES (?1, ?2, ?3, '/tmp/test', 'txt', 100, 'abc123', ?3, ?4, ?5)",
            rusqlite::params![id, collection_id, filename, now, now],
        )
        .expect("insert document");
        id
    }

    /// Helper: insert a chunk and return its id.
    fn insert_chunk(conn: &Connection, document_id: &str, collection_id: &str, content: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO chunks (id, document_id, collection_id, content, chunk_index, created_at)
             VALUES (?1, ?2, ?3, ?4, 0, ?5)",
            rusqlite::params![id, document_id, collection_id, content, now],
        )
        .expect("insert chunk");

        // Also insert into FTS
        conn.execute(
            "INSERT INTO chunks_fts (content, chunk_id, document_id, collection_id)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![content, id, document_id, collection_id],
        )
        .expect("insert fts");

        // Also insert embedding
        conn.execute(
            "INSERT INTO chunk_embeddings (chunk_id, collection_id, document_id, embedding, content_preview)
             VALUES (?1, ?2, ?3, X'00', ?4)",
            rusqlite::params![id, collection_id, document_id, content],
        )
        .expect("insert embedding");

        id
    }

    #[test]
    fn test_export_all_data_returns_valid_json() {
        let conn = setup_test_db();
        let coll_id = insert_collection(&conn, "Export Test");
        insert_document(&conn, &coll_id, "test.txt");

        let exported = export_all_data(&conn).expect("export failed");

        // Should have all expected tables
        assert!(exported.contains_key("collections"));
        assert!(exported.contains_key("documents"));
        assert!(exported.contains_key("settings"));

        // Each value should be valid JSON
        for (table, json_str) in &exported {
            let parsed: serde_json::Value =
                serde_json::from_str(json_str).unwrap_or_else(|e| panic!("Invalid JSON for {}: {}", table, e));
            assert!(parsed.is_array(), "Expected array for table {}", table);
        }

        // Collections should have at least 1 entry
        let collections: serde_json::Value =
            serde_json::from_str(exported.get("collections").expect("missing collections"))
                .expect("parse collections");
        assert!(
            collections.as_array().expect("not array").len() >= 1,
            "Should have at least 1 collection"
        );
    }

    #[test]
    fn test_erase_document_removes_all_related() {
        let conn = setup_test_db();
        let coll_id = insert_collection(&conn, "Erase Doc Test");
        let doc_id = insert_document(&conn, &coll_id, "to_erase.txt");
        let chunk_id = insert_chunk(&conn, &doc_id, &coll_id, "some content");

        // Insert a graph edge referencing the chunk
        conn.execute(
            "INSERT INTO graph_edges (id, source_chunk_id, target_chunk_id, collection_id, created_at) VALUES ('ge1', ?1, ?1, ?2, '2025-01-01')",
            rusqlite::params![chunk_id, coll_id],
        ).expect("insert edge");

        // Verify data exists before erase
        let chunk_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM chunks WHERE document_id = ?1", rusqlite::params![doc_id], |row| row.get(0))
            .expect("count chunks");
        assert_eq!(chunk_count, 1);

        erase_document(&conn, &doc_id).expect("erase_document failed");

        // Verify everything is gone
        let doc_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM documents WHERE id = ?1", rusqlite::params![doc_id], |row| row.get(0))
            .expect("count docs");
        assert_eq!(doc_count, 0);

        let chunk_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM chunks WHERE document_id = ?1", rusqlite::params![doc_id], |row| row.get(0))
            .expect("count chunks");
        assert_eq!(chunk_count, 0);

        let emb_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM chunk_embeddings WHERE document_id = ?1", rusqlite::params![doc_id], |row| row.get(0))
            .expect("count embeddings");
        assert_eq!(emb_count, 0);

        let edge_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM graph_edges WHERE id = 'ge1'", [], |row| row.get(0))
            .expect("count edges");
        assert_eq!(edge_count, 0);
    }

    #[test]
    fn test_erase_collection_cascades() {
        let conn = setup_test_db();
        let coll_id = insert_collection(&conn, "Erase Coll Test");
        let doc_id = insert_document(&conn, &coll_id, "doc1.txt");
        insert_chunk(&conn, &doc_id, &coll_id, "chunk content");

        // Insert a conversation with a message
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO conversations (id, collection_id, title, created_at, updated_at) VALUES ('conv1', ?1, 'Test', ?2, ?3)",
            rusqlite::params![coll_id, now, now],
        ).expect("insert conversation");
        conn.execute(
            "INSERT INTO messages (id, conversation_id, role, content, created_at) VALUES ('msg1', 'conv1', 'user', 'hello', ?1)",
            rusqlite::params![now],
        ).expect("insert message");

        erase_collection(&conn, &coll_id).expect("erase_collection failed");

        // Everything should be gone
        let coll_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM collections WHERE id = ?1", rusqlite::params![coll_id], |row| row.get(0))
            .expect("count coll");
        assert_eq!(coll_count, 0);

        let doc_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM documents WHERE collection_id = ?1", rusqlite::params![coll_id], |row| row.get(0))
            .expect("count docs");
        assert_eq!(doc_count, 0);

        let chunk_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM chunks WHERE collection_id = ?1", rusqlite::params![coll_id], |row| row.get(0))
            .expect("count chunks");
        assert_eq!(chunk_count, 0);

        let conv_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM conversations WHERE collection_id = ?1", rusqlite::params![coll_id], |row| row.get(0))
            .expect("count convs");
        assert_eq!(conv_count, 0);

        let msg_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM messages WHERE conversation_id = 'conv1'", [], |row| row.get(0))
            .expect("count msgs");
        assert_eq!(msg_count, 0);
    }

    #[test]
    fn test_erase_all_data_empties_everything() {
        let conn = setup_test_db();
        let coll_id = insert_collection(&conn, "Nuke Test");
        insert_document(&conn, &coll_id, "nuked.txt");

        conn.execute(
            "INSERT INTO search_history (id, collection_id, query, created_at) VALUES ('sh1', ?1, 'test query', '2025-01-01')",
            rusqlite::params![coll_id],
        ).expect("insert search history");

        erase_all_data(&conn).expect("erase_all_data failed");

        let tables_to_check = [
            "collections",
            "documents",
            "chunks",
            "conversations",
            "messages",
            "citations",
            "search_history",
            "audit_log",
        ];

        for table in &tables_to_check {
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| row.get(0))
                .expect("count");
            assert_eq!(count, 0, "Table {} should be empty after erase_all_data", table);
        }

        // Settings should still have defaults
        let settings_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))
            .expect("count settings");
        assert!(settings_count > 0, "Settings should be re-seeded with defaults");
    }

    #[test]
    fn test_retention_policy_deletes_expired() {
        let conn = setup_test_db();
        let coll_id = insert_collection(&conn, "Retention Test");

        // Insert a retention policy: search_history, 30 days, auto_delete=1
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO data_retention_policies (id, entity_type, retention_days, auto_delete, created_at, updated_at)
             VALUES ('rp1', 'search_history', 30, 1, ?1, ?2)",
            rusqlite::params![now, now],
        ).expect("insert policy");

        // Insert old search history (60 days ago)
        let old_date = (chrono::Utc::now() - chrono::Duration::days(60)).to_rfc3339();
        conn.execute(
            "INSERT INTO search_history (id, collection_id, query, created_at) VALUES ('old1', ?1, 'old query', ?2)",
            rusqlite::params![coll_id, old_date],
        ).expect("insert old history");

        // Insert recent search history (1 day ago)
        let recent_date = (chrono::Utc::now() - chrono::Duration::days(1)).to_rfc3339();
        conn.execute(
            "INSERT INTO search_history (id, collection_id, query, created_at) VALUES ('new1', ?1, 'new query', ?2)",
            rusqlite::params![coll_id, recent_date],
        ).expect("insert new history");

        let deleted = enforce_retention_policies(&conn).expect("enforce failed");
        assert_eq!(deleted, 1, "Should have deleted 1 expired row");

        // Old one should be gone, new one should remain
        let remaining: i64 = conn
            .query_row("SELECT COUNT(*) FROM search_history", [], |row| row.get(0))
            .expect("count");
        assert_eq!(remaining, 1);

        let remaining_id: String = conn
            .query_row("SELECT id FROM search_history", [], |row| row.get(0))
            .expect("get id");
        assert_eq!(remaining_id, "new1");
    }

    #[test]
    fn test_consent_record_grant_and_revoke() {
        let conn = setup_test_db();

        // Grant consent
        record_consent(&conn, "data_processing", true).expect("grant failed");

        let records = get_consent_records(&conn).expect("get records failed");
        assert_eq!(records.len(), 1);
        assert!(records[0].granted);
        assert_eq!(records[0].consent_type, "data_processing");
        assert!(records[0].revoked_at.is_none());

        // Revoke consent
        record_consent(&conn, "data_processing", false).expect("revoke failed");

        let records = get_consent_records(&conn).expect("get records failed");
        assert_eq!(records.len(), 1);
        assert!(!records[0].granted);
        assert!(records[0].revoked_at.is_some());
    }
}
