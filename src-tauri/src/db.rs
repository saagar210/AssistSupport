//! Database module for AssistSupport
//! SQLCipher encrypted database with FTS5 full-text search

use crate::security::{MasterKey, SecurityError};
use rusqlite::{Connection, Result as SqliteResult, params};
use std::path::{Path, PathBuf};
use thiserror::Error;

const CURRENT_SCHEMA_VERSION: i32 = 3;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    #[error("Database not initialized")]
    NotInitialized,
    #[error("Migration failed: {0}")]
    Migration(String),
    #[error("Database corruption detected")]
    Corruption,
    #[error("FTS5 not available in this build")]
    Fts5NotAvailable,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Database manager for AssistSupport
pub struct Database {
    conn: Connection,
    path: PathBuf,
}

impl Database {
    /// Open or create encrypted database
    pub fn open(path: &Path, master_key: &MasterKey) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;

        // Set SQLCipher key (hex-encoded)
        // Using default SQLCipher 4 settings for compatibility
        let key_pragma = format!("PRAGMA key = \"x'{}'\"", master_key.to_hex());
        conn.execute_batch(&key_pragma)?;

        // Verify the key works by reading from the database
        conn.execute_batch("SELECT count(*) FROM sqlite_master;")?;

        let db = Self {
            conn,
            path: path.to_path_buf(),
        };

        // Verify FTS5 is available
        db.verify_fts5()?;

        Ok(db)
    }

    /// Initialize database schema
    pub fn initialize(&self) -> Result<(), DbError> {
        // Run integrity check
        self.check_integrity()?;

        // Get current schema version
        let version = self.get_schema_version()?;

        // Run migrations
        if version < CURRENT_SCHEMA_VERSION {
            self.run_migrations(version)?;
        }

        Ok(())
    }

    /// Verify FTS5 extension is available (release gate)
    pub fn verify_fts5(&self) -> Result<bool, DbError> {
        // Check if FTS5 is compiled in
        let result: SqliteResult<i32> = self.conn.query_row(
            "SELECT 1 WHERE EXISTS (SELECT 1 FROM pragma_compile_options WHERE compile_options = 'ENABLE_FTS5')",
            [],
            |row| row.get(0),
        );

        match result {
            Ok(_) => Ok(true),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Try to create a test FTS5 table as fallback verification
                match self.conn.execute(
                    "CREATE VIRTUAL TABLE IF NOT EXISTS _fts5_test USING fts5(content)",
                    [],
                ) {
                    Ok(_) => {
                        self.conn.execute("DROP TABLE IF EXISTS _fts5_test", [])?;
                        Ok(true)
                    }
                    Err(_) => Err(DbError::Fts5NotAvailable),
                }
            }
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// Check database integrity
    pub fn check_integrity(&self) -> Result<(), DbError> {
        let result: String = self.conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;

        if result != "ok" {
            return Err(DbError::Corruption);
        }

        Ok(())
    }

    /// Get current schema version
    fn get_schema_version(&self) -> Result<i32, DbError> {
        // Create settings table if not exists
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        let version: SqliteResult<String> = self.conn.query_row(
            "SELECT value FROM settings WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        );

        match version {
            Ok(v) => v.parse().map_err(|_| DbError::Migration("Invalid schema version".into())),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// Set schema version
    fn set_schema_version(&self, version: i32) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('schema_version', ?)",
            params![version.to_string()],
        )?;
        Ok(())
    }

    /// Run database migrations
    fn run_migrations(&self, from_version: i32) -> Result<(), DbError> {
        // Backup before migration
        self.backup()?;

        let tx = self.conn.unchecked_transaction()?;

        if from_version < 1 {
            self.migrate_v1()?;
        }

        if from_version < 2 {
            self.migrate_v2()?;
        }

        if from_version < 3 {
            self.migrate_v3()?;
        }

        tx.commit()?;
        self.set_schema_version(CURRENT_SCHEMA_VERSION)?;

        Ok(())
    }

    /// Migration to v1: Initial schema
    fn migrate_v1(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            r#"
            -- Core settings
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- Drafts
            CREATE TABLE IF NOT EXISTS drafts (
                id TEXT PRIMARY KEY,
                input_text TEXT NOT NULL,
                summary_text TEXT,
                diagnosis_json TEXT,
                response_text TEXT,
                ticket_id TEXT,
                kb_sources_json TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                is_autosave INTEGER DEFAULT 0,
                model_name TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_drafts_created ON drafts(created_at);
            CREATE INDEX IF NOT EXISTS idx_drafts_ticket ON drafts(ticket_id);

            -- Follow-ups
            CREATE TABLE IF NOT EXISTS followups (
                id TEXT PRIMARY KEY,
                draft_id TEXT,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (draft_id) REFERENCES drafts(id) ON DELETE SET NULL
            );

            -- Attachments (encrypted at rest)
            CREATE TABLE IF NOT EXISTS attachments (
                id TEXT PRIMARY KEY,
                draft_id TEXT,
                filename TEXT NOT NULL,
                mime_type TEXT,
                encrypted_path TEXT NOT NULL,
                ocr_text TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (draft_id) REFERENCES drafts(id) ON DELETE CASCADE
            );

            -- Knowledge Base Documents
            CREATE TABLE IF NOT EXISTS kb_documents (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL UNIQUE,
                file_hash TEXT NOT NULL,
                title TEXT,
                indexed_at TEXT,
                chunk_count INTEGER,
                ocr_quality TEXT,
                partial_index INTEGER DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_kb_docs_path ON kb_documents(file_path);

            -- Document Chunks (keep rowid for FTS5 joins)
            CREATE TABLE IF NOT EXISTS kb_chunks (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                heading_path TEXT,
                content TEXT NOT NULL,
                word_count INTEGER,
                FOREIGN KEY (document_id) REFERENCES kb_documents(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_kb_chunks_doc ON kb_chunks(document_id);

            -- FTS5 Full-Text Search Index
            CREATE VIRTUAL TABLE IF NOT EXISTS kb_fts USING fts5(
                content, heading_path,
                content='kb_chunks',
                tokenize='porter unicode61'
            );

            -- FTS5 Triggers (sync with kb_chunks via rowid)
            CREATE TRIGGER IF NOT EXISTS kb_chunks_ai AFTER INSERT ON kb_chunks BEGIN
                INSERT INTO kb_fts(rowid, content, heading_path)
                VALUES (new.rowid, new.content, new.heading_path);
            END;

            CREATE TRIGGER IF NOT EXISTS kb_chunks_ad AFTER DELETE ON kb_chunks BEGIN
                INSERT INTO kb_fts(kb_fts, rowid, content, heading_path)
                VALUES ('delete', old.rowid, old.content, old.heading_path);
            END;

            CREATE TRIGGER IF NOT EXISTS kb_chunks_au AFTER UPDATE ON kb_chunks BEGIN
                INSERT INTO kb_fts(kb_fts, rowid, content, heading_path)
                VALUES ('delete', old.rowid, old.content, old.heading_path);
                INSERT INTO kb_fts(rowid, content, heading_path)
                VALUES (new.rowid, new.content, new.heading_path);
            END;

            -- Diagnostic Sessions
            CREATE TABLE IF NOT EXISTS diagnostic_sessions (
                id TEXT PRIMARY KEY,
                draft_id TEXT,
                checklist_json TEXT,
                findings_json TEXT,
                decision_tree_id TEXT,
                tree_path_json TEXT,
                escalation_note TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (draft_id) REFERENCES drafts(id) ON DELETE SET NULL
            );

            -- Decision Trees (built-in + custom)
            CREATE TABLE IF NOT EXISTS decision_trees (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                category TEXT,
                tree_json TEXT NOT NULL,
                source TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Learning Stats (opt-in)
            CREATE TABLE IF NOT EXISTS learning_checklist_stats (
                item_text_hash TEXT PRIMARY KEY,
                times_shown INTEGER DEFAULT 0,
                times_checked INTEGER DEFAULT 0,
                times_led_to_resolution INTEGER DEFAULT 0,
                avg_time_to_check_ms INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS learning_tree_stats (
                tree_id TEXT NOT NULL,
                node_id TEXT NOT NULL,
                times_visited INTEGER DEFAULT 0,
                times_led_to_resolution INTEGER DEFAULT 0,
                PRIMARY KEY (tree_id, node_id)
            );

            -- Vector search consent (LanceDB)
            CREATE TABLE IF NOT EXISTS vector_consent (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                enabled INTEGER NOT NULL DEFAULT 0,
                consented_at TEXT,
                encryption_supported INTEGER
            );
            INSERT OR IGNORE INTO vector_consent (id, enabled) VALUES (1, 0);

            -- Custom template variables
            CREATE TABLE IF NOT EXISTS custom_variables (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                value TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            "#,
        )?;

        Ok(())
    }

    /// Migration to v2: Add index for drafts.updated_at (FollowUps performance)
    fn migrate_v2(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            r#"
            -- Add index for faster draft sorting by updated_at (used in FollowUps tab)
            CREATE INDEX IF NOT EXISTS idx_drafts_updated ON drafts(updated_at DESC);
            "#,
        )?;

        Ok(())
    }

    /// Migration to v3: Add model_name column to drafts (track which model generated response)
    fn migrate_v3(&self) -> Result<(), DbError> {
        // Check if model_name column already exists (may exist if created from fresh schema)
        let has_model_name: bool = self
            .conn
            .prepare("PRAGMA table_info(drafts)")?
            .query_map([], |row| {
                let name: String = row.get(1)?;
                Ok(name)
            })?
            .filter_map(|r| r.ok())
            .any(|name| name == "model_name");

        if !has_model_name {
            self.conn.execute_batch(
                r#"
                -- Add model_name column to track which model generated each response
                ALTER TABLE drafts ADD COLUMN model_name TEXT;
                "#,
            )?;
        }

        Ok(())
    }

    /// Create backup of database
    /// Note: For SQLCipher encrypted databases, we use file copy instead of SQLite backup API
    pub fn backup(&self) -> Result<PathBuf, DbError> {
        let backup_path = self.path.with_extension("db.bak");

        // For SQLCipher, the standard backup API doesn't work with encrypted databases
        // We'll use a file copy approach instead (database must be checkpointed first)
        self.conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;

        // Copy the database file
        std::fs::copy(&self.path, &backup_path)?;

        Ok(backup_path)
    }

    /// Get inner connection reference
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Execute a simple query (for testing)
    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize, DbError> {
        Ok(self.conn.execute(sql, params)?)
    }

    /// FTS5 search for KB chunks
    pub fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<FtsSearchResult>, DbError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                kb_chunks.id,
                kb_chunks.document_id,
                kb_chunks.heading_path,
                snippet(kb_fts, 0, '<mark>', '</mark>', '...', 32) as snippet,
                bm25(kb_fts) as rank
            FROM kb_fts
            JOIN kb_chunks ON kb_fts.rowid = kb_chunks.rowid
            WHERE kb_fts MATCH ?
            ORDER BY rank
            LIMIT ?
            "#,
        )?;

        let results = stmt
            .query_map(params![query, limit as i64], |row| {
                Ok(FtsSearchResult {
                    chunk_id: row.get(0)?,
                    document_id: row.get(1)?,
                    heading_path: row.get(2)?,
                    snippet: row.get(3)?,
                    rank: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /// Get vector consent status
    pub fn get_vector_consent(&self) -> Result<VectorConsent, DbError> {
        let row = self.conn.query_row(
            "SELECT enabled, consented_at, encryption_supported FROM vector_consent WHERE id = 1",
            [],
            |row| {
                Ok(VectorConsent {
                    enabled: row.get::<_, i32>(0)? != 0,
                    consented_at: row.get(1)?,
                    encryption_supported: row.get::<_, Option<i32>>(2)?.map(|v| v != 0),
                })
            },
        )?;
        Ok(row)
    }

    /// Set vector consent
    pub fn set_vector_consent(&self, enabled: bool, encryption_supported: bool) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE vector_consent SET enabled = ?, consented_at = ?, encryption_supported = ? WHERE id = 1",
            params![enabled as i32, now, encryption_supported as i32],
        )?;
        Ok(())
    }

    /// List all decision trees
    pub fn list_decision_trees(&self) -> Result<Vec<DecisionTree>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, category, tree_json, source, created_at, updated_at
             FROM decision_trees ORDER BY name"
        )?;

        let trees = stmt.query_map([], |row| {
            Ok(DecisionTree {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                tree_json: row.get(3)?,
                source: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(trees)
    }

    /// Get a single decision tree by ID
    pub fn get_decision_tree(&self, tree_id: &str) -> Result<DecisionTree, DbError> {
        let tree = self.conn.query_row(
            "SELECT id, name, category, tree_json, source, created_at, updated_at
             FROM decision_trees WHERE id = ?",
            [tree_id],
            |row| Ok(DecisionTree {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                tree_json: row.get(3)?,
                source: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        )?;
        Ok(tree)
    }

    /// Save or update a decision tree
    pub fn save_decision_tree(&self, tree: &DecisionTree) -> Result<String, DbError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO decision_trees
             (id, name, category, tree_json, source, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                &tree.id,
                &tree.name,
                &tree.category,
                &tree.tree_json,
                &tree.source,
                &tree.created_at,
                &tree.updated_at,
            ],
        )?;
        Ok(tree.id.clone())
    }

    /// Ensure response_templates table exists (called during init)
    pub fn ensure_templates_table(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS response_templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                category TEXT,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    // ============================================================================
    // Draft Methods
    // ============================================================================

    /// List recent drafts
    pub fn list_drafts(&self, limit: usize) -> Result<Vec<SavedDraft>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, input_text, summary_text, diagnosis_json, response_text,
                    ticket_id, kb_sources_json, created_at, updated_at, is_autosave, model_name
             FROM drafts
             ORDER BY updated_at DESC
             LIMIT ?"
        )?;

        let drafts = stmt.query_map([limit as i64], |row| {
            Ok(SavedDraft {
                id: row.get(0)?,
                input_text: row.get(1)?,
                summary_text: row.get(2)?,
                diagnosis_json: row.get(3)?,
                response_text: row.get(4)?,
                ticket_id: row.get(5)?,
                kb_sources_json: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                is_autosave: row.get::<_, i32>(9)? != 0,
                model_name: row.get(10)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(drafts)
    }

    /// Search drafts by text content
    pub fn search_drafts(&self, query: &str, limit: usize) -> Result<Vec<SavedDraft>, DbError> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, input_text, summary_text, diagnosis_json, response_text,
                    ticket_id, kb_sources_json, created_at, updated_at, is_autosave, model_name
             FROM drafts
             WHERE is_autosave = 0
               AND (input_text LIKE ?1 OR response_text LIKE ?1 OR ticket_id LIKE ?1)
             ORDER BY updated_at DESC
             LIMIT ?2"
        )?;

        let drafts = stmt.query_map(params![pattern, limit as i64], |row| {
            Ok(SavedDraft {
                id: row.get(0)?,
                input_text: row.get(1)?,
                summary_text: row.get(2)?,
                diagnosis_json: row.get(3)?,
                response_text: row.get(4)?,
                ticket_id: row.get(5)?,
                kb_sources_json: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                is_autosave: row.get::<_, i32>(9)? != 0,
                model_name: row.get(10)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(drafts)
    }

    /// Get a single draft by ID
    pub fn get_draft(&self, draft_id: &str) -> Result<SavedDraft, DbError> {
        let draft = self.conn.query_row(
            "SELECT id, input_text, summary_text, diagnosis_json, response_text,
                    ticket_id, kb_sources_json, created_at, updated_at, is_autosave, model_name
             FROM drafts WHERE id = ?",
            [draft_id],
            |row| Ok(SavedDraft {
                id: row.get(0)?,
                input_text: row.get(1)?,
                summary_text: row.get(2)?,
                diagnosis_json: row.get(3)?,
                response_text: row.get(4)?,
                ticket_id: row.get(5)?,
                kb_sources_json: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                is_autosave: row.get::<_, i32>(9)? != 0,
                model_name: row.get(10)?,
            })
        )?;
        Ok(draft)
    }

    /// Save a draft (insert or update)
    pub fn save_draft(&self, draft: &SavedDraft) -> Result<String, DbError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO drafts
             (id, input_text, summary_text, diagnosis_json, response_text,
              ticket_id, kb_sources_json, created_at, updated_at, is_autosave, model_name)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &draft.id,
                &draft.input_text,
                &draft.summary_text,
                &draft.diagnosis_json,
                &draft.response_text,
                &draft.ticket_id,
                &draft.kb_sources_json,
                &draft.created_at,
                &draft.updated_at,
                draft.is_autosave as i32,
                &draft.model_name,
            ],
        )?;
        Ok(draft.id.clone())
    }

    /// Delete a draft
    pub fn delete_draft(&self, draft_id: &str) -> Result<(), DbError> {
        self.conn.execute("DELETE FROM drafts WHERE id = ?", [draft_id])?;
        Ok(())
    }

    /// Cleanup old autosaves, keeping only the most recent ones
    pub fn cleanup_autosaves(&self, keep_count: usize) -> Result<usize, DbError> {
        // Delete old autosaves, keeping only the most recent `keep_count`
        let deleted = self.conn.execute(
            "DELETE FROM drafts WHERE is_autosave = 1 AND id NOT IN (
                SELECT id FROM drafts WHERE is_autosave = 1
                ORDER BY created_at DESC LIMIT ?
            )",
            [keep_count],
        )?;
        Ok(deleted)
    }

    /// List autosave drafts (most recent first)
    pub fn list_autosaves(&self, limit: usize) -> Result<Vec<SavedDraft>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, input_text, summary_text, diagnosis_json, response_text,
                    ticket_id, kb_sources_json, created_at, updated_at, is_autosave, model_name
             FROM drafts
             WHERE is_autosave = 1
             ORDER BY created_at DESC
             LIMIT ?"
        )?;

        let drafts = stmt.query_map([limit], |row| {
            Ok(SavedDraft {
                id: row.get(0)?,
                input_text: row.get(1)?,
                summary_text: row.get(2)?,
                diagnosis_json: row.get(3)?,
                response_text: row.get(4)?,
                ticket_id: row.get(5)?,
                kb_sources_json: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                is_autosave: row.get::<_, i32>(9)? != 0,
                model_name: row.get(10)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(drafts)
    }

    /// Get draft versions by input hash (autosaves with matching input_text hash)
    /// The hash is computed as SHA256(input_text)[0:16]
    pub fn get_draft_versions(&self, input_hash: &str) -> Result<Vec<SavedDraft>, DbError> {
        use sha2::{Sha256, Digest};

        // Get all autosaves and filter by input hash
        let all_autosaves = self.list_autosaves(100)?; // Get more to search through

        let matching: Vec<SavedDraft> = all_autosaves
            .into_iter()
            .filter(|draft| {
                let mut hasher = Sha256::new();
                hasher.update(draft.input_text.as_bytes());
                let hash = hex::encode(hasher.finalize());
                hash[..16] == *input_hash
            })
            .collect();

        Ok(matching)
    }

    // ============================================================================
    // Response Template Methods
    // ============================================================================

    /// List all response templates
    pub fn list_templates(&self) -> Result<Vec<ResponseTemplate>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, category, content, created_at, updated_at
             FROM response_templates
             ORDER BY name"
        )?;

        let templates = stmt.query_map([], |row| {
            Ok(ResponseTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(templates)
    }

    /// Get a single template by ID
    pub fn get_template(&self, template_id: &str) -> Result<ResponseTemplate, DbError> {
        let template = self.conn.query_row(
            "SELECT id, name, category, content, created_at, updated_at
             FROM response_templates WHERE id = ?",
            [template_id],
            |row| Ok(ResponseTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        )?;
        Ok(template)
    }

    /// Save a template (insert or update)
    pub fn save_template(&self, template: &ResponseTemplate) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO response_templates
             (id, name, category, content, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                &template.id,
                &template.name,
                &template.category,
                &template.content,
                &template.created_at,
                &template.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Delete a template
    pub fn delete_template(&self, template_id: &str) -> Result<(), DbError> {
        self.conn.execute("DELETE FROM response_templates WHERE id = ?", [template_id])?;
        Ok(())
    }

    // ============================================================================
    // Custom Variable Methods
    // ============================================================================

    /// Ensure custom_variables table exists (for existing databases)
    pub fn ensure_custom_variables_table(&self) -> Result<(), DbError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS custom_variables (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                value TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    /// List all custom variables
    pub fn list_custom_variables(&self) -> Result<Vec<CustomVariable>, DbError> {
        // Ensure table exists for older databases
        self.ensure_custom_variables_table()?;

        let mut stmt = self.conn.prepare(
            "SELECT id, name, value, created_at
             FROM custom_variables
             ORDER BY name"
        )?;

        let variables = stmt.query_map([], |row| {
            Ok(CustomVariable {
                id: row.get(0)?,
                name: row.get(1)?,
                value: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(variables)
    }

    /// Get a single custom variable by ID
    pub fn get_custom_variable(&self, variable_id: &str) -> Result<CustomVariable, DbError> {
        let variable = self.conn.query_row(
            "SELECT id, name, value, created_at
             FROM custom_variables WHERE id = ?",
            [variable_id],
            |row| Ok(CustomVariable {
                id: row.get(0)?,
                name: row.get(1)?,
                value: row.get(2)?,
                created_at: row.get(3)?,
            })
        )?;
        Ok(variable)
    }

    /// Save a custom variable (insert or update)
    pub fn save_custom_variable(&self, variable: &CustomVariable) -> Result<(), DbError> {
        // Ensure table exists
        self.ensure_custom_variables_table()?;

        self.conn.execute(
            "INSERT INTO custom_variables (id, name, value, created_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                value = excluded.value",
            params![
                variable.id,
                variable.name,
                variable.value,
                variable.created_at,
            ],
        )?;
        Ok(())
    }

    /// Delete a custom variable
    pub fn delete_custom_variable(&self, variable_id: &str) -> Result<(), DbError> {
        self.conn.execute("DELETE FROM custom_variables WHERE id = ?", [variable_id])?;
        Ok(())
    }

    /// Seed built-in decision trees (called on first run)
    pub fn seed_builtin_trees(&self) -> Result<(), DbError> {
        // Check if already seeded
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM decision_trees WHERE source = 'builtin'",
            [],
            |row| row.get(0)
        )?;

        if count > 0 {
            return Ok(());
        }

        let now = chrono::Utc::now().to_rfc3339();

        // Insert 4 core built-in trees
        for tree in BUILTIN_TREES.iter() {
            self.conn.execute(
                "INSERT INTO decision_trees (id, name, category, tree_json, source, created_at, updated_at)
                 VALUES (?, ?, ?, ?, 'builtin', ?, ?)",
                params![tree.0, tree.1, tree.2, tree.3, &now, &now],
            )?;
        }

        Ok(())
    }

    /// Get all chunk IDs and content for embedding generation
    pub fn get_all_chunks_for_embedding(&self) -> Result<Vec<(String, String)>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content FROM kb_chunks ORDER BY document_id, chunk_index"
        )?;

        let chunks = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(chunks)
    }

    /// Get chunk content by ID
    pub fn get_chunk_content(&self, chunk_id: &str) -> Result<String, DbError> {
        self.conn.query_row(
            "SELECT content FROM kb_chunks WHERE id = ?",
            [chunk_id],
            |row| row.get(0),
        ).map_err(DbError::Sqlite)
    }
}

/// FTS5 search result
#[derive(Debug, Clone, serde::Serialize)]
pub struct FtsSearchResult {
    pub chunk_id: String,
    pub document_id: String,
    pub heading_path: Option<String>,
    pub snippet: String,
    pub rank: f64,
}

/// Vector consent status
#[derive(Debug, Clone, serde::Serialize)]
pub struct VectorConsent {
    pub enabled: bool,
    pub consented_at: Option<String>,
    pub encryption_supported: Option<bool>,
}

/// Decision tree from database
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionTree {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    pub tree_json: String,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Saved draft from database
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedDraft {
    pub id: String,
    pub input_text: String,
    pub summary_text: Option<String>,
    pub diagnosis_json: Option<String>,
    pub response_text: Option<String>,
    pub ticket_id: Option<String>,
    pub kb_sources_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub is_autosave: bool,
    /// Name of the model that generated this response (e.g., "Llama 3.2 3B Instruct")
    #[serde(default)]
    pub model_name: Option<String>,
}

/// Response template from database
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseTemplate {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Custom template variable
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomVariable {
    pub id: String,
    pub name: String,
    pub value: String,
    pub created_at: String,
}

/// Built-in decision trees: (id, name, category, tree_json)
const BUILTIN_TREES: &[(&str, &str, &str, &str)] = &[
    ("auth-issues", "Authentication Issues", "Security", include_str!("trees/auth.json")),
    ("vpn-connectivity", "VPN Connectivity", "Network", include_str!("trees/vpn.json")),
    ("email-calendar", "Email & Calendar", "Productivity", include_str!("trees/email.json")),
    ("password-reset", "Password Reset", "Security", include_str!("trees/password.json")),
];

/// Get the application data directory
pub fn get_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AssistSupport")
}

/// Get database path
pub fn get_db_path() -> PathBuf {
    get_app_data_dir().join("assistsupport.db")
}

/// Get attachments directory
pub fn get_attachments_dir() -> PathBuf {
    get_app_data_dir().join("attachments")
}

/// Get models directory
pub fn get_models_dir() -> PathBuf {
    get_app_data_dir().join("models")
}

/// Get vectors directory (LanceDB)
pub fn get_vectors_dir() -> PathBuf {
    get_app_data_dir().join("vectors")
}

/// Get downloads directory
pub fn get_downloads_dir() -> PathBuf {
    get_app_data_dir().join("downloads")
}

/// Get logs directory
pub fn get_logs_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Logs")
        .join("AssistSupport")
}

/// Get cache directory
pub fn get_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AssistSupport")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_db() -> (Database, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let key = MasterKey::generate();
        let db = Database::open(&db_path, &key).unwrap();
        db.initialize().unwrap();
        (db, dir)
    }

    #[test]
    fn test_database_creation() {
        let (db, _dir) = create_test_db();
        assert!(db.check_integrity().is_ok());
    }

    #[test]
    fn test_fts5_available() {
        let (db, _dir) = create_test_db();
        assert!(db.verify_fts5().unwrap());
    }

    #[test]
    fn test_sqlcipher_encryption() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("encrypted.db");
        let key = MasterKey::generate();

        // Create and write to encrypted database
        {
            let db = Database::open(&db_path, &key).unwrap();
            db.initialize().unwrap();
            db.conn()
                .execute(
                    "INSERT INTO settings (key, value) VALUES ('test', 'secret_data')",
                    [],
                )
                .unwrap();
        }

        // Verify file is encrypted (raw read should not contain plaintext)
        let raw_content = std::fs::read(&db_path).unwrap();
        let content_str = String::from_utf8_lossy(&raw_content);
        assert!(
            !content_str.contains("secret_data"),
            "Database file should be encrypted - plaintext found!"
        );
        assert!(
            !content_str.contains("SQLite format"),
            "Database should be encrypted - SQLite header found!"
        );

        // Verify we can still read with correct key
        let db = Database::open(&db_path, &key).unwrap();
        let value: String = db
            .conn()
            .query_row("SELECT value FROM settings WHERE key = 'test'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(value, "secret_data");
    }

    #[test]
    fn test_wrong_key_fails() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("encrypted.db");
        let key1 = MasterKey::generate();
        let key2 = MasterKey::generate();

        // Create database with key1
        {
            let db = Database::open(&db_path, &key1).unwrap();
            db.initialize().unwrap();
        }

        // Opening with wrong key should fail
        let result = Database::open(&db_path, &key2);
        assert!(result.is_err(), "Should fail to open with wrong key");
    }

    #[test]
    fn test_fts5_indexing() {
        let (db, _dir) = create_test_db();

        // Insert a test document
        db.conn()
            .execute(
                "INSERT INTO kb_documents (id, file_path, file_hash, indexed_at) VALUES (?, ?, ?, ?)",
                params!["doc1", "/test/doc.md", "abc123", "2024-01-01"],
            )
            .unwrap();

        // Insert a test chunk (trigger should update FTS5)
        db.conn()
            .execute(
                "INSERT INTO kb_chunks (id, document_id, chunk_index, heading_path, content, word_count) VALUES (?, ?, ?, ?, ?, ?)",
                params!["chunk1", "doc1", 0, "Test > Heading", "This is a test chunk about authentication errors", 8],
            )
            .unwrap();

        // Search should find the chunk
        let results = db.fts_search("authentication", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chunk_id, "chunk1");
    }

    #[test]
    fn test_fts5_triggers() {
        let (db, _dir) = create_test_db();

        // Insert document
        db.conn()
            .execute(
                "INSERT INTO kb_documents (id, file_path, file_hash, indexed_at) VALUES (?, ?, ?, ?)",
                params!["doc1", "/test/doc.md", "abc123", "2024-01-01"],
            )
            .unwrap();

        // Insert chunk
        db.conn()
            .execute(
                "INSERT INTO kb_chunks (id, document_id, chunk_index, heading_path, content, word_count) VALUES (?, ?, ?, ?, ?, ?)",
                params!["chunk1", "doc1", 0, "Heading", "VPN connection troubleshooting guide", 4],
            )
            .unwrap();

        // Verify FTS5 has the content
        let results = db.fts_search("VPN", 10).unwrap();
        assert_eq!(results.len(), 1);

        // Update chunk
        db.conn()
            .execute(
                "UPDATE kb_chunks SET content = 'Password reset instructions' WHERE id = 'chunk1'",
                [],
            )
            .unwrap();

        // Old content should not be found
        let results = db.fts_search("VPN", 10).unwrap();
        assert_eq!(results.len(), 0);

        // New content should be found
        let results = db.fts_search("Password", 10).unwrap();
        assert_eq!(results.len(), 1);

        // Delete chunk
        db.conn()
            .execute("DELETE FROM kb_chunks WHERE id = 'chunk1'", [])
            .unwrap();

        // Should not be found after delete
        let results = db.fts_search("Password", 10).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_vector_consent() {
        let (db, _dir) = create_test_db();

        // Default should be disabled
        let consent = db.get_vector_consent().unwrap();
        assert!(!consent.enabled);

        // Enable with encryption support
        db.set_vector_consent(true, true).unwrap();
        let consent = db.get_vector_consent().unwrap();
        assert!(consent.enabled);
        assert_eq!(consent.encryption_supported, Some(true));
    }
}
