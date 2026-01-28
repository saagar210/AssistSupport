//! Database module for AssistSupport
//! SQLCipher encrypted database with FTS5 full-text search

pub mod executor;

pub use executor::{DbExecutor, DbExecutorError};

use crate::jobs::{Job, JobLog, JobStatus, JobType, LogLevel};
use crate::security::{MasterKey, SecurityError};
use crate::validation::{normalize_and_validate_namespace_id, ValidationError};
use chrono::Utc;
use rusqlite::{params, Connection, Result as SqliteResult};
use std::path::{Path, PathBuf};
use thiserror::Error;
use zeroize::Zeroize;

const CURRENT_SCHEMA_VERSION: i32 = 8;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
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
        let mut hex_key = master_key.to_hex();
        let mut key_pragma = format!("PRAGMA key = \"x'{}'\"", hex_key);
        hex_key.zeroize();
        let pragma_result = conn.execute_batch(&key_pragma);
        key_pragma.zeroize();
        pragma_result?;

        // Verify the key works by reading from the database
        conn.execute_batch("SELECT count(*) FROM sqlite_master;")?;

        // Enable foreign key enforcement (required for ON DELETE CASCADE)
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Set busy timeout (5 seconds) to avoid SQLITE_BUSY errors
        conn.execute_batch("PRAGMA busy_timeout = 5000;")?;

        // Use WAL journal mode for better concurrent read performance
        // Note: WAL works with SQLCipher but plaintext_header_size may need adjustment
        // for older SQLCipher versions. Default behavior is safe.
        let _ = conn.execute_batch("PRAGMA journal_mode = WAL;");

        // Set secure delete to overwrite deleted content
        conn.execute_batch("PRAGMA secure_delete = ON;")?;

        let db = Self {
            conn,
            path: path.to_path_buf(),
        };

        // Set secure file permissions on database file
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
        }

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
        let result: String = self
            .conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))?;

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
            Ok(v) => v
                .parse()
                .map_err(|_| DbError::Migration("Invalid schema version".into())),
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

        if from_version < 4 {
            self.migrate_v4()?;
        }

        if from_version < 5 {
            self.migrate_v5()?;
        }

        if from_version < 6 {
            self.migrate_v6()?;
        }

        if from_version < 7 {
            self.migrate_v7()?;
        }

        if from_version < 8 {
            self.migrate_v8()?;
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

    /// Migration to v4: Add namespaces, ingest sources, and update kb tables
    fn migrate_v4(&self) -> Result<(), DbError> {
        // Create namespaces table
        self.conn.execute_batch(
            r#"
            -- Namespaces for organizing content
            CREATE TABLE IF NOT EXISTS namespaces (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                color TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Insert default namespace
            INSERT OR IGNORE INTO namespaces (id, name, description, color, created_at, updated_at)
            VALUES ('default', 'Default', 'Default namespace for all content', '#6366f1', datetime('now'), datetime('now'));

            -- Ingest sources (web URLs, YouTube videos, GitHub repos)
            CREATE TABLE IF NOT EXISTS ingest_sources (
                id TEXT PRIMARY KEY,
                source_type TEXT NOT NULL CHECK(source_type IN ('web', 'youtube', 'github', 'file')),
                source_uri TEXT NOT NULL,
                namespace_id TEXT NOT NULL DEFAULT 'default',
                title TEXT,
                etag TEXT,
                last_modified TEXT,
                content_hash TEXT,
                last_ingested_at TEXT,
                status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'active', 'stale', 'error', 'removed')),
                error_message TEXT,
                metadata_json TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (namespace_id) REFERENCES namespaces(id) ON DELETE CASCADE,
                UNIQUE(source_type, source_uri, namespace_id)
            );
            CREATE INDEX IF NOT EXISTS idx_ingest_sources_namespace ON ingest_sources(namespace_id);
            CREATE INDEX IF NOT EXISTS idx_ingest_sources_type ON ingest_sources(source_type);
            CREATE INDEX IF NOT EXISTS idx_ingest_sources_status ON ingest_sources(status);

            -- Ingest runs (track ingest operations)
            CREATE TABLE IF NOT EXISTS ingest_runs (
                id TEXT PRIMARY KEY,
                source_id TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                status TEXT NOT NULL DEFAULT 'running' CHECK(status IN ('running', 'completed', 'failed', 'cancelled')),
                documents_added INTEGER DEFAULT 0,
                documents_updated INTEGER DEFAULT 0,
                documents_removed INTEGER DEFAULT 0,
                chunks_added INTEGER DEFAULT 0,
                error_message TEXT,
                FOREIGN KEY (source_id) REFERENCES ingest_sources(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_ingest_runs_source ON ingest_runs(source_id);
            CREATE INDEX IF NOT EXISTS idx_ingest_runs_started ON ingest_runs(started_at DESC);

            -- GitHub tokens (encrypted, stored separately for security)
            CREATE TABLE IF NOT EXISTS github_tokens (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                encrypted_token BLOB,
                token_name TEXT,
                created_at TEXT,
                last_used_at TEXT
            );
            INSERT OR IGNORE INTO github_tokens (id) VALUES (1);

            -- Network allowlist for SSRF protection override
            CREATE TABLE IF NOT EXISTS network_allowlist (
                id TEXT PRIMARY KEY,
                host_pattern TEXT NOT NULL UNIQUE,
                reason TEXT,
                created_at TEXT NOT NULL
            );
            "#,
        )?;

        // Check if namespace column already exists in kb_documents
        let has_namespace_col: bool = self
            .conn
            .prepare("PRAGMA table_info(kb_documents)")?
            .query_map([], |row| {
                let name: String = row.get(1)?;
                Ok(name)
            })?
            .filter_map(|r| r.ok())
            .any(|name| name == "namespace_id");

        if !has_namespace_col {
            // Add new columns to kb_documents
            self.conn.execute_batch(
                r#"
                -- Add namespace and source columns to kb_documents
                ALTER TABLE kb_documents ADD COLUMN namespace_id TEXT NOT NULL DEFAULT 'default';
                ALTER TABLE kb_documents ADD COLUMN source_type TEXT NOT NULL DEFAULT 'file';
                ALTER TABLE kb_documents ADD COLUMN source_id TEXT;

                -- Update existing documents to have default namespace
                UPDATE kb_documents SET namespace_id = 'default' WHERE namespace_id = 'default';

                -- Create unique index on (namespace_id, file_path) replacing file_path UNIQUE
                DROP INDEX IF EXISTS idx_kb_docs_path;
                CREATE UNIQUE INDEX idx_kb_docs_namespace_path ON kb_documents(namespace_id, file_path);
                CREATE INDEX idx_kb_docs_source ON kb_documents(source_id);
                "#,
            )?;
        }

        // Check if namespace column already exists in kb_chunks
        let has_chunk_namespace: bool = self
            .conn
            .prepare("PRAGMA table_info(kb_chunks)")?
            .query_map([], |row| {
                let name: String = row.get(1)?;
                Ok(name)
            })?
            .filter_map(|r| r.ok())
            .any(|name| name == "namespace_id");

        if !has_chunk_namespace {
            // Add namespace column to kb_chunks
            self.conn.execute_batch(
                r#"
                -- Add namespace column to kb_chunks for faster filtering
                ALTER TABLE kb_chunks ADD COLUMN namespace_id TEXT NOT NULL DEFAULT 'default';

                -- Update chunks with namespace from their parent documents
                UPDATE kb_chunks SET namespace_id = (
                    SELECT namespace_id FROM kb_documents WHERE kb_documents.id = kb_chunks.document_id
                );

                -- Create index for namespace filtering
                CREATE INDEX IF NOT EXISTS idx_kb_chunks_namespace ON kb_chunks(namespace_id);
                "#,
            )?;
        }

        Ok(())
    }

    /// Migration to v5: Add jobs and job_logs tables for background task management
    fn migrate_v5(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            r#"
            -- Jobs table for background task management
            CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                job_type TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'queued' CHECK(status IN ('queued', 'running', 'succeeded', 'failed', 'cancelled')),
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                started_at TEXT,
                completed_at TEXT,
                progress REAL DEFAULT 0.0,
                progress_message TEXT,
                error TEXT,
                metadata_json TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
            CREATE INDEX IF NOT EXISTS idx_jobs_created ON jobs(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_jobs_type ON jobs(job_type);

            -- Job logs for detailed progress tracking
            CREATE TABLE IF NOT EXISTS job_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                level TEXT NOT NULL DEFAULT 'info' CHECK(level IN ('debug', 'info', 'warning', 'error')),
                message TEXT NOT NULL,
                FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_job_logs_job ON job_logs(job_id);
            CREATE INDEX IF NOT EXISTS idx_job_logs_timestamp ON job_logs(timestamp DESC);
            "#,
        )?;

        Ok(())
    }

    /// Migration to v6: Document versioning and source trust (Phase 14)
    fn migrate_v6(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            r#"
            -- Document versions for rollback support
            CREATE TABLE IF NOT EXISTS document_versions (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL,
                version_number INTEGER NOT NULL,
                file_hash TEXT NOT NULL,
                content_snapshot TEXT,
                chunks_json TEXT,
                created_at TEXT NOT NULL,
                change_reason TEXT,
                FOREIGN KEY (document_id) REFERENCES kb_documents(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_doc_versions_doc ON document_versions(document_id);
            CREATE INDEX IF NOT EXISTS idx_doc_versions_num ON document_versions(document_id, version_number DESC);

            -- Source trust and curation metadata
            ALTER TABLE ingest_sources ADD COLUMN trust_score REAL DEFAULT 0.5;
            ALTER TABLE ingest_sources ADD COLUMN is_pinned INTEGER DEFAULT 0;
            ALTER TABLE ingest_sources ADD COLUMN owner TEXT;
            ALTER TABLE ingest_sources ADD COLUMN review_status TEXT DEFAULT 'pending'
                CHECK(review_status IN ('pending', 'approved', 'rejected', 'needs_review'));
            ALTER TABLE ingest_sources ADD COLUMN tags_json TEXT;
            ALTER TABLE ingest_sources ADD COLUMN stale_at TEXT;

            -- Document curation metadata
            ALTER TABLE kb_documents ADD COLUMN review_status TEXT DEFAULT 'auto_approved'
                CHECK(review_status IN ('pending', 'approved', 'rejected', 'auto_approved'));
            ALTER TABLE kb_documents ADD COLUMN is_pinned INTEGER DEFAULT 0;
            ALTER TABLE kb_documents ADD COLUMN tags_json TEXT;
            ALTER TABLE kb_documents ADD COLUMN owner TEXT;

            -- Namespace ingestion rules (allowlist/denylist)
            CREATE TABLE IF NOT EXISTS namespace_rules (
                id TEXT PRIMARY KEY,
                namespace_id TEXT NOT NULL,
                rule_type TEXT NOT NULL CHECK(rule_type IN ('allow', 'deny')),
                pattern_type TEXT NOT NULL CHECK(pattern_type IN ('domain', 'file_pattern', 'url_pattern')),
                pattern TEXT NOT NULL,
                reason TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (namespace_id) REFERENCES namespaces(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_ns_rules_ns ON namespace_rules(namespace_id);
            CREATE INDEX IF NOT EXISTS idx_ns_rules_type ON namespace_rules(rule_type, pattern_type);

            -- Trust score defaults based on source type
            UPDATE ingest_sources SET trust_score = 0.8 WHERE source_type = 'file' AND trust_score = 0.5;
            UPDATE ingest_sources SET trust_score = 0.6 WHERE source_type = 'web' AND trust_score = 0.5;
            UPDATE ingest_sources SET trust_score = 0.5 WHERE source_type = 'youtube' AND trust_score = 0.5;
            "#,
        )?;

        Ok(())
    }

    /// Migration to v7: IT Support workflow enhancements (Phase 17)
    /// - Case intake fields for drafts
    /// - Draft versioning and finalization
    /// - Playbooks for common workflows
    fn migrate_v7(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            r#"
            -- Add case intake and workflow fields to drafts
            ALTER TABLE drafts ADD COLUMN case_intake_json TEXT;
            ALTER TABLE drafts ADD COLUMN status TEXT DEFAULT 'draft'
                CHECK(status IN ('draft', 'finalized', 'archived'));
            ALTER TABLE drafts ADD COLUMN handoff_summary TEXT;
            ALTER TABLE drafts ADD COLUMN finalized_at TEXT;
            ALTER TABLE drafts ADD COLUMN finalized_by TEXT;

            -- Draft versions for history/diff view
            CREATE TABLE IF NOT EXISTS draft_versions (
                id TEXT PRIMARY KEY,
                draft_id TEXT NOT NULL,
                version_number INTEGER NOT NULL,
                input_text TEXT,
                summary_text TEXT,
                response_text TEXT,
                case_intake_json TEXT,
                kb_sources_json TEXT,
                created_at TEXT NOT NULL,
                change_reason TEXT,
                FOREIGN KEY (draft_id) REFERENCES drafts(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_draft_versions_draft ON draft_versions(draft_id);
            CREATE INDEX IF NOT EXISTS idx_draft_versions_num ON draft_versions(draft_id, version_number DESC);

            -- Playbooks: curated workflows tied to decision trees
            CREATE TABLE IF NOT EXISTS playbooks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                category TEXT,
                decision_tree_id TEXT,
                steps_json TEXT NOT NULL,
                template_id TEXT,
                shortcuts_json TEXT,
                is_active INTEGER DEFAULT 1,
                usage_count INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (template_id) REFERENCES response_templates(id) ON DELETE SET NULL
            );
            CREATE INDEX IF NOT EXISTS idx_playbooks_category ON playbooks(category);
            CREATE INDEX IF NOT EXISTS idx_playbooks_active ON playbooks(is_active);
            CREATE INDEX IF NOT EXISTS idx_playbooks_tree ON playbooks(decision_tree_id);

            -- Action shortcuts: one-click sequences
            CREATE TABLE IF NOT EXISTS action_shortcuts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                shortcut_key TEXT,
                action_type TEXT NOT NULL CHECK(action_type IN ('template', 'clarify', 'request_logs', 'summarize', 'custom')),
                action_data_json TEXT NOT NULL,
                category TEXT,
                sort_order INTEGER DEFAULT 0,
                is_active INTEGER DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_shortcuts_type ON action_shortcuts(action_type);
            CREATE INDEX IF NOT EXISTS idx_shortcuts_active ON action_shortcuts(is_active, sort_order);

            -- Insert default action shortcuts
            INSERT OR IGNORE INTO action_shortcuts (id, name, action_type, action_data_json, category, sort_order, created_at, updated_at)
            VALUES
                ('clarify_default', 'Request Clarification', 'clarify', '{"prompt": "To help resolve this issue, could you please provide:\n\n1. When did this issue first occur?\n2. Have you tried any troubleshooting steps?\n3. Are other users affected?"}', 'intake', 1, datetime('now'), datetime('now')),
                ('request_logs', 'Request Logs', 'request_logs', '{"prompt": "To investigate further, please share:\n\n- Screenshots of any error messages\n- Relevant log files\n- Steps to reproduce the issue"}', 'intake', 2, datetime('now'), datetime('now')),
                ('summarize_steps', 'Summarize Resolution', 'summarize', '{"prompt": "Resolution Summary\n\nIssue: [Brief description]\n\nRoot Cause: [What caused it]\n\nResolution: [Steps taken]\n\nPrevention: [How to avoid in future]"}', 'resolution', 3, datetime('now'), datetime('now'));
            "#,
        )?;

        Ok(())
    }

    /// Migration to v8: Response ratings and analytics events
    fn migrate_v8(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            r#"
            -- Phase 4: Response ratings
            CREATE TABLE IF NOT EXISTS response_ratings (
                id TEXT PRIMARY KEY,
                draft_id TEXT NOT NULL,
                rating INTEGER NOT NULL CHECK(rating BETWEEN 1 AND 5),
                feedback_text TEXT,
                feedback_category TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (draft_id) REFERENCES drafts(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_response_ratings_draft ON response_ratings(draft_id);
            CREATE INDEX IF NOT EXISTS idx_response_ratings_created ON response_ratings(created_at DESC);

            -- Phase 2: Analytics events
            CREATE TABLE IF NOT EXISTS analytics_events (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                event_data_json TEXT,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_analytics_events_type ON analytics_events(event_type);
            CREATE INDEX IF NOT EXISTS idx_analytics_events_created ON analytics_events(created_at DESC);
            "#,
        )?;

        Ok(())
    }

    /// Create backup of database
    /// Note: For SQLCipher encrypted databases, we use file copy instead of SQLite backup API
    pub fn backup(&self) -> Result<PathBuf, DbError> {
        let backup_path = self.path.with_extension("db.bak");

        // For SQLCipher, the standard backup API doesn't work with encrypted databases
        // We'll use a file copy approach instead (database must be checkpointed first)
        self.conn
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;

        // Copy the database file
        std::fs::copy(&self.path, &backup_path)?;

        // Set secure file permissions on backup file
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&backup_path, std::fs::Permissions::from_mode(0o600));
        }

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
    pub fn set_vector_consent(
        &self,
        enabled: bool,
        encryption_supported: bool,
    ) -> Result<(), DbError> {
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
             FROM decision_trees ORDER BY name",
        )?;

        let trees = stmt
            .query_map([], |row| {
                Ok(DecisionTree {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    tree_json: row.get(3)?,
                    source: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(trees)
    }

    /// Get a single decision tree by ID
    pub fn get_decision_tree(&self, tree_id: &str) -> Result<DecisionTree, DbError> {
        let tree = self.conn.query_row(
            "SELECT id, name, category, tree_json, source, created_at, updated_at
             FROM decision_trees WHERE id = ?",
            [tree_id],
            |row| {
                Ok(DecisionTree {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    tree_json: row.get(3)?,
                    source: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
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
                    ticket_id, kb_sources_json, created_at, updated_at, is_autosave, model_name,
                    case_intake_json, status, handoff_summary, finalized_at, finalized_by
             FROM drafts
             ORDER BY updated_at DESC
             LIMIT ?",
        )?;

        let drafts = stmt
            .query_map([limit as i64], |row| {
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
                    case_intake_json: row.get(11)?,
                    status: row
                        .get::<_, Option<String>>(12)?
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_default(),
                    handoff_summary: row.get(13)?,
                    finalized_at: row.get(14)?,
                    finalized_by: row.get(15)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(drafts)
    }

    /// Search drafts by text content
    pub fn search_drafts(&self, query: &str, limit: usize) -> Result<Vec<SavedDraft>, DbError> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, input_text, summary_text, diagnosis_json, response_text,
                    ticket_id, kb_sources_json, created_at, updated_at, is_autosave, model_name,
                    case_intake_json, status, handoff_summary, finalized_at, finalized_by
             FROM drafts
             WHERE is_autosave = 0
               AND (input_text LIKE ?1 OR response_text LIKE ?1 OR ticket_id LIKE ?1)
             ORDER BY updated_at DESC
             LIMIT ?2",
        )?;

        let drafts = stmt
            .query_map(params![pattern, limit as i64], |row| {
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
                    case_intake_json: row.get(11)?,
                    status: row
                        .get::<_, Option<String>>(12)?
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_default(),
                    handoff_summary: row.get(13)?,
                    finalized_at: row.get(14)?,
                    finalized_by: row.get(15)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(drafts)
    }

    /// Get a single draft by ID
    pub fn get_draft(&self, draft_id: &str) -> Result<SavedDraft, DbError> {
        let draft = self.conn.query_row(
            "SELECT id, input_text, summary_text, diagnosis_json, response_text,
                    ticket_id, kb_sources_json, created_at, updated_at, is_autosave, model_name,
                    case_intake_json, status, handoff_summary, finalized_at, finalized_by
             FROM drafts WHERE id = ?",
            [draft_id],
            |row| {
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
                    case_intake_json: row.get(11)?,
                    status: row
                        .get::<_, Option<String>>(12)?
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_default(),
                    handoff_summary: row.get(13)?,
                    finalized_at: row.get(14)?,
                    finalized_by: row.get(15)?,
                })
            },
        )?;
        Ok(draft)
    }

    /// Save a draft (insert or update)
    pub fn save_draft(&self, draft: &SavedDraft) -> Result<String, DbError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO drafts
             (id, input_text, summary_text, diagnosis_json, response_text,
              ticket_id, kb_sources_json, created_at, updated_at, is_autosave, model_name,
              case_intake_json, status, handoff_summary, finalized_at, finalized_by)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
                &draft.case_intake_json,
                draft.status.to_string(),
                &draft.handoff_summary,
                &draft.finalized_at,
                &draft.finalized_by,
            ],
        )?;
        Ok(draft.id.clone())
    }

    /// Delete a draft
    pub fn delete_draft(&self, draft_id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM drafts WHERE id = ?", [draft_id])?;
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
                    ticket_id, kb_sources_json, created_at, updated_at, is_autosave, model_name,
                    case_intake_json, status, handoff_summary, finalized_at, finalized_by
             FROM drafts
             WHERE is_autosave = 1
             ORDER BY created_at DESC
             LIMIT ?",
        )?;

        let drafts = stmt
            .query_map([limit], |row| {
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
                    case_intake_json: row.get(11)?,
                    status: row
                        .get::<_, Option<String>>(12)?
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_default(),
                    handoff_summary: row.get(13)?,
                    finalized_at: row.get(14)?,
                    finalized_by: row.get(15)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(drafts)
    }

    /// Get draft versions by input hash (autosaves with matching input_text hash)
    /// The hash is computed as SHA256(input_text)[0:16]
    pub fn get_draft_versions(&self, input_hash: &str) -> Result<Vec<SavedDraft>, DbError> {
        use sha2::{Digest, Sha256};

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
    // Draft Versioning Methods (Phase 17)
    // ============================================================================

    /// Create a draft version snapshot
    pub fn create_draft_version(
        &self,
        draft_id: &str,
        change_reason: Option<&str>,
    ) -> Result<String, DbError> {
        // Get current draft state
        let draft = self.get_draft(draft_id)?;

        // Get next version number
        let version_number: i32 = self.conn.query_row(
            "SELECT COALESCE(MAX(version_number), 0) + 1 FROM draft_versions WHERE draft_id = ?",
            [draft_id],
            |row| row.get(0),
        )?;

        let version_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO draft_versions
             (id, draft_id, version_number, input_text, summary_text, response_text,
              case_intake_json, kb_sources_json, created_at, change_reason)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &version_id,
                draft_id,
                version_number,
                &draft.input_text,
                &draft.summary_text,
                &draft.response_text,
                &draft.case_intake_json,
                &draft.kb_sources_json,
                &now,
                change_reason,
            ],
        )?;

        Ok(version_id)
    }

    /// List draft versions
    pub fn list_draft_versions(&self, draft_id: &str) -> Result<Vec<DraftVersion>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, draft_id, version_number, input_text, summary_text, response_text,
                    case_intake_json, kb_sources_json, created_at, change_reason
             FROM draft_versions
             WHERE draft_id = ?
             ORDER BY version_number DESC",
        )?;

        let versions = stmt
            .query_map([draft_id], |row| {
                Ok(DraftVersion {
                    id: row.get(0)?,
                    draft_id: row.get(1)?,
                    version_number: row.get(2)?,
                    input_text: row.get(3)?,
                    summary_text: row.get(4)?,
                    response_text: row.get(5)?,
                    case_intake_json: row.get(6)?,
                    kb_sources_json: row.get(7)?,
                    created_at: row.get(8)?,
                    change_reason: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(versions)
    }

    /// Finalize a draft (lock it and mark as read-only)
    pub fn finalize_draft(
        &self,
        draft_id: &str,
        finalized_by: Option<&str>,
    ) -> Result<(), DbError> {
        // Create a version snapshot before finalizing
        self.create_draft_version(draft_id, Some("Pre-finalization snapshot"))?;

        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE drafts SET status = 'finalized', finalized_at = ?, finalized_by = ?, updated_at = ?
             WHERE id = ?",
            params![&now, finalized_by, &now, draft_id],
        )?;
        Ok(())
    }

    /// Archive a draft
    pub fn archive_draft(&self, draft_id: &str) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE drafts SET status = 'archived', updated_at = ? WHERE id = ?",
            params![&now, draft_id],
        )?;
        Ok(())
    }

    /// Update draft handoff summary
    pub fn update_draft_handoff(
        &self,
        draft_id: &str,
        handoff_summary: &str,
    ) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE drafts SET handoff_summary = ?, updated_at = ? WHERE id = ?",
            params![handoff_summary, &now, draft_id],
        )?;
        Ok(())
    }

    // ============================================================================
    // Playbook Methods (Phase 17)
    // ============================================================================

    /// List all active playbooks
    pub fn list_playbooks(&self, category: Option<&str>) -> Result<Vec<Playbook>, DbError> {
        let query = match category {
            Some(_) => "SELECT id, name, description, category, decision_tree_id, steps_json,
                               template_id, shortcuts_json, is_active, usage_count, created_at, updated_at
                        FROM playbooks WHERE is_active = 1 AND category = ? ORDER BY usage_count DESC",
            None => "SELECT id, name, description, category, decision_tree_id, steps_json,
                            template_id, shortcuts_json, is_active, usage_count, created_at, updated_at
                     FROM playbooks WHERE is_active = 1 ORDER BY usage_count DESC",
        };

        let mut stmt = self.conn.prepare(query)?;
        let playbooks = if let Some(cat) = category {
            stmt.query_map([cat], Self::row_to_playbook)?
        } else {
            stmt.query_map([], Self::row_to_playbook)?
        }
        .collect::<Result<Vec<_>, _>>()?;

        Ok(playbooks)
    }

    fn row_to_playbook(row: &rusqlite::Row) -> Result<Playbook, rusqlite::Error> {
        Ok(Playbook {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            category: row.get(3)?,
            decision_tree_id: row.get(4)?,
            steps_json: row.get(5)?,
            template_id: row.get(6)?,
            shortcuts_json: row.get(7)?,
            is_active: row.get::<_, i32>(8)? != 0,
            usage_count: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    }

    /// Get a playbook by ID
    pub fn get_playbook(&self, playbook_id: &str) -> Result<Playbook, DbError> {
        let playbook = self.conn.query_row(
            "SELECT id, name, description, category, decision_tree_id, steps_json,
                    template_id, shortcuts_json, is_active, usage_count, created_at, updated_at
             FROM playbooks WHERE id = ?",
            [playbook_id],
            Self::row_to_playbook,
        )?;
        Ok(playbook)
    }

    /// Save a playbook (insert or update)
    pub fn save_playbook(&self, playbook: &Playbook) -> Result<String, DbError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO playbooks
             (id, name, description, category, decision_tree_id, steps_json,
              template_id, shortcuts_json, is_active, usage_count, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &playbook.id,
                &playbook.name,
                &playbook.description,
                &playbook.category,
                &playbook.decision_tree_id,
                &playbook.steps_json,
                &playbook.template_id,
                &playbook.shortcuts_json,
                playbook.is_active as i32,
                playbook.usage_count,
                &playbook.created_at,
                &playbook.updated_at,
            ],
        )?;
        Ok(playbook.id.clone())
    }

    /// Increment playbook usage count
    pub fn increment_playbook_usage(&self, playbook_id: &str) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE playbooks SET usage_count = usage_count + 1 WHERE id = ?",
            [playbook_id],
        )?;
        Ok(())
    }

    /// Delete a playbook
    pub fn delete_playbook(&self, playbook_id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM playbooks WHERE id = ?", [playbook_id])?;
        Ok(())
    }

    // ============================================================================
    // Action Shortcut Methods (Phase 17)
    // ============================================================================

    /// List all active action shortcuts
    pub fn list_action_shortcuts(
        &self,
        category: Option<&str>,
    ) -> Result<Vec<ActionShortcut>, DbError> {
        let query = match category {
            Some(_) => "SELECT id, name, shortcut_key, action_type, action_data_json,
                               category, sort_order, is_active, created_at, updated_at
                        FROM action_shortcuts WHERE is_active = 1 AND category = ? ORDER BY sort_order",
            None => "SELECT id, name, shortcut_key, action_type, action_data_json,
                            category, sort_order, is_active, created_at, updated_at
                     FROM action_shortcuts WHERE is_active = 1 ORDER BY sort_order",
        };

        let mut stmt = self.conn.prepare(query)?;
        let shortcuts = if let Some(cat) = category {
            stmt.query_map([cat], Self::row_to_action_shortcut)?
        } else {
            stmt.query_map([], Self::row_to_action_shortcut)?
        }
        .collect::<Result<Vec<_>, _>>()?;

        Ok(shortcuts)
    }

    fn row_to_action_shortcut(row: &rusqlite::Row) -> Result<ActionShortcut, rusqlite::Error> {
        Ok(ActionShortcut {
            id: row.get(0)?,
            name: row.get(1)?,
            shortcut_key: row.get(2)?,
            action_type: row.get(3)?,
            action_data_json: row.get(4)?,
            category: row.get(5)?,
            sort_order: row.get(6)?,
            is_active: row.get::<_, i32>(7)? != 0,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }

    /// Get an action shortcut by ID
    pub fn get_action_shortcut(&self, shortcut_id: &str) -> Result<ActionShortcut, DbError> {
        let shortcut = self.conn.query_row(
            "SELECT id, name, shortcut_key, action_type, action_data_json,
                    category, sort_order, is_active, created_at, updated_at
             FROM action_shortcuts WHERE id = ?",
            [shortcut_id],
            Self::row_to_action_shortcut,
        )?;
        Ok(shortcut)
    }

    /// Save an action shortcut (insert or update)
    pub fn save_action_shortcut(&self, shortcut: &ActionShortcut) -> Result<String, DbError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO action_shortcuts
             (id, name, shortcut_key, action_type, action_data_json,
              category, sort_order, is_active, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &shortcut.id,
                &shortcut.name,
                &shortcut.shortcut_key,
                &shortcut.action_type,
                &shortcut.action_data_json,
                &shortcut.category,
                shortcut.sort_order,
                shortcut.is_active as i32,
                &shortcut.created_at,
                &shortcut.updated_at,
            ],
        )?;
        Ok(shortcut.id.clone())
    }

    /// Delete an action shortcut
    pub fn delete_action_shortcut(&self, shortcut_id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM action_shortcuts WHERE id = ?", [shortcut_id])?;
        Ok(())
    }

    // ============================================================================
    // Response Template Methods
    // ============================================================================

    /// List all response templates
    pub fn list_templates(&self) -> Result<Vec<ResponseTemplate>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, category, content, created_at, updated_at
             FROM response_templates
             ORDER BY name",
        )?;

        let templates = stmt
            .query_map([], |row| {
                Ok(ResponseTemplate {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(templates)
    }

    /// Get a single template by ID
    pub fn get_template(&self, template_id: &str) -> Result<ResponseTemplate, DbError> {
        let template = self.conn.query_row(
            "SELECT id, name, category, content, created_at, updated_at
             FROM response_templates WHERE id = ?",
            [template_id],
            |row| {
                Ok(ResponseTemplate {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
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
        self.conn
            .execute("DELETE FROM response_templates WHERE id = ?", [template_id])?;
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
             ORDER BY name",
        )?;

        let variables = stmt
            .query_map([], |row| {
                Ok(CustomVariable {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    value: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(variables)
    }

    /// Get a single custom variable by ID
    pub fn get_custom_variable(&self, variable_id: &str) -> Result<CustomVariable, DbError> {
        let variable = self.conn.query_row(
            "SELECT id, name, value, created_at
             FROM custom_variables WHERE id = ?",
            [variable_id],
            |row| {
                Ok(CustomVariable {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    value: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
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
        self.conn
            .execute("DELETE FROM custom_variables WHERE id = ?", [variable_id])?;
        Ok(())
    }

    /// Seed built-in decision trees (called on first run)
    pub fn seed_builtin_trees(&self) -> Result<(), DbError> {
        // Check if already seeded
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM decision_trees WHERE source = 'builtin'",
            [],
            |row| row.get(0),
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
        let mut stmt = self
            .conn
            .prepare("SELECT id, content FROM kb_chunks ORDER BY document_id, chunk_index")?;

        let chunks = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(chunks)
    }

    /// Get chunk content by ID
    pub fn get_chunk_content(&self, chunk_id: &str) -> Result<String, DbError> {
        self.conn
            .query_row(
                "SELECT content FROM kb_chunks WHERE id = ?",
                [chunk_id],
                |row| row.get(0),
            )
            .map_err(DbError::Sqlite)
    }

    // ============================================================================
    // Namespace Methods
    // ============================================================================

    /// List all namespaces
    pub fn list_namespaces(&self) -> Result<Vec<Namespace>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, color, created_at, updated_at
             FROM namespaces ORDER BY name",
        )?;

        let namespaces = stmt
            .query_map([], |row| {
                Ok(Namespace {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    color: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(namespaces)
    }

    /// List all namespaces with document and source counts (optimized single query)
    pub fn list_namespaces_with_counts(&self) -> Result<Vec<NamespaceWithCounts>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT
                n.id, n.name, n.description, n.color, n.created_at, n.updated_at,
                COALESCE(d.doc_count, 0) as document_count,
                COALESCE(s.source_count, 0) as source_count
             FROM namespaces n
             LEFT JOIN (
                 SELECT namespace_id, COUNT(*) as doc_count
                 FROM kb_documents
                 GROUP BY namespace_id
             ) d ON d.namespace_id = n.id
             LEFT JOIN (
                 SELECT namespace_id, COUNT(*) as source_count
                 FROM ingest_sources
                 GROUP BY namespace_id
             ) s ON s.namespace_id = n.id
             ORDER BY n.name",
        )?;

        let namespaces = stmt
            .query_map([], |row| {
                Ok(NamespaceWithCounts {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    color: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                    document_count: row.get(6)?,
                    source_count: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(namespaces)
    }

    /// Get a namespace by ID
    pub fn get_namespace(&self, namespace_id: &str) -> Result<Namespace, DbError> {
        self.conn
            .query_row(
                "SELECT id, name, description, color, created_at, updated_at
             FROM namespaces WHERE id = ?",
                [namespace_id],
                |row| {
                    Ok(Namespace {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        color: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                },
            )
            .map_err(DbError::Sqlite)
    }

    /// Create or update a namespace
    pub fn save_namespace(&self, namespace: &Namespace) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO namespaces (id, name, description, color, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                color = excluded.color,
                updated_at = excluded.updated_at",
            params![
                namespace.id,
                namespace.name,
                namespace.description,
                namespace.color,
                namespace.created_at,
                namespace.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Delete a namespace (and all its content)
    pub fn delete_namespace(&self, namespace_id: &str) -> Result<(), DbError> {
        if namespace_id == "default" {
            return Err(DbError::Migration("Cannot delete default namespace".into()));
        }
        // Cascade delete: documents -> chunks are handled by ON DELETE CASCADE
        // Delete documents first
        self.conn.execute(
            "DELETE FROM kb_documents WHERE namespace_id = ?",
            [namespace_id],
        )?;
        // Delete ingest sources
        self.conn.execute(
            "DELETE FROM ingest_sources WHERE namespace_id = ?",
            [namespace_id],
        )?;
        // Delete namespace
        self.conn
            .execute("DELETE FROM namespaces WHERE id = ?", [namespace_id])?;
        Ok(())
    }

    /// Create a new namespace with name, description, and color
    ///
    /// The namespace ID is normalized using the centralized validation rules:
    /// - Converted to lowercase
    /// - Spaces and underscores become hyphens
    /// - Special characters removed
    /// - Multiple hyphens collapsed
    /// - Max length 64 characters
    pub fn create_namespace(
        &self,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<Namespace, DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        // Use centralized normalization for consistency
        let id = normalize_and_validate_namespace_id(name)?;

        let namespace = Namespace {
            id: id.clone(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            color: color.map(|s| s.to_string()),
            created_at: now.clone(),
            updated_at: now,
        };

        self.save_namespace(&namespace)?;
        Ok(namespace)
    }

    /// Ensure a namespace exists, creating it if necessary
    pub fn ensure_namespace_exists(&self, namespace_id: &str) -> Result<(), DbError> {
        // Check if exists
        let exists: bool = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM namespaces WHERE id = ?)",
            [namespace_id],
            |row| row.get(0),
        )?;

        if !exists {
            let now = chrono::Utc::now().to_rfc3339();
            self.conn.execute(
                "INSERT INTO namespaces (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)",
                params![namespace_id, namespace_id, now, now],
            )?;
        }

        Ok(())
    }

    /// Rename a namespace (updates all references)
    ///
    /// Uses centralized namespace ID normalization for consistency.
    pub fn rename_namespace(&self, old_id: &str, new_id: &str) -> Result<(), DbError> {
        if old_id == "default" {
            return Err(DbError::Migration("Cannot rename default namespace".into()));
        }

        let now = chrono::Utc::now().to_rfc3339();
        // Use centralized normalization for consistency
        let new_id_normalized = normalize_and_validate_namespace_id(new_id)?;

        // Update namespace
        self.conn.execute(
            "UPDATE namespaces SET id = ?, name = ?, updated_at = ? WHERE id = ?",
            params![new_id_normalized, new_id, now, old_id],
        )?;

        // Update references in documents
        self.conn.execute(
            "UPDATE kb_documents SET namespace_id = ? WHERE namespace_id = ?",
            params![new_id_normalized, old_id],
        )?;

        // Update references in chunks
        self.conn.execute(
            "UPDATE kb_chunks SET namespace_id = ? WHERE namespace_id = ?",
            params![new_id_normalized, old_id],
        )?;

        // Update references in ingest sources
        self.conn.execute(
            "UPDATE ingest_sources SET namespace_id = ? WHERE namespace_id = ?",
            params![new_id_normalized, old_id],
        )?;

        Ok(())
    }

    /// Migrate existing namespace IDs to the canonical normalized form
    ///
    /// This function scans all namespaces and normalizes their IDs using
    /// the centralized validation rules. It updates all references (documents,
    /// chunks, ingest sources) to use the new canonical ID.
    ///
    /// Returns a list of (old_id, new_id) pairs for namespaces that were migrated.
    pub fn migrate_namespace_ids(&self) -> Result<Vec<(String, String)>, DbError> {
        use crate::validation::normalize_namespace_id;

        let mut migrated = Vec::new();

        // Get all namespaces
        let mut stmt = self.conn.prepare("SELECT id, name FROM namespaces")?;
        let namespaces: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        for (old_id, name) in namespaces {
            // Compute canonical ID
            let canonical_id = normalize_namespace_id(&name);

            // Skip if already canonical
            if old_id == canonical_id {
                continue;
            }

            // Skip if canonical is empty (shouldn't happen, but be safe)
            if canonical_id.is_empty() {
                tracing::warn!("Skipping namespace '{}' - normalized ID is empty", old_id);
                continue;
            }

            // Check if canonical ID already exists (collision)
            let exists: bool = self.conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM namespaces WHERE id = ?)",
                [&canonical_id],
                |row| row.get(0),
            )?;

            if exists && old_id != canonical_id {
                tracing::warn!(
                    "Skipping namespace '{}' - canonical ID '{}' already exists",
                    old_id,
                    canonical_id
                );
                continue;
            }

            let now = chrono::Utc::now().to_rfc3339();

            // Update namespace ID
            self.conn.execute(
                "UPDATE namespaces SET id = ?, updated_at = ? WHERE id = ?",
                params![canonical_id, now, old_id],
            )?;

            // Update references in documents
            self.conn.execute(
                "UPDATE kb_documents SET namespace_id = ? WHERE namespace_id = ?",
                params![canonical_id, old_id],
            )?;

            // Update references in chunks
            self.conn.execute(
                "UPDATE kb_chunks SET namespace_id = ? WHERE namespace_id = ?",
                params![canonical_id, old_id],
            )?;

            // Update references in ingest sources
            self.conn.execute(
                "UPDATE ingest_sources SET namespace_id = ? WHERE namespace_id = ?",
                params![canonical_id, old_id],
            )?;

            tracing::info!("Migrated namespace ID '{}' -> '{}'", old_id, canonical_id);
            migrated.push((old_id, canonical_id));
        }

        Ok(migrated)
    }

    // ============================================================================
    // Ingest Source Methods
    // ============================================================================

    /// List ingest sources, optionally filtered by namespace
    pub fn list_ingest_sources(
        &self,
        namespace_id: Option<&str>,
    ) -> Result<Vec<IngestSource>, DbError> {
        let map_row = |row: &rusqlite::Row| -> rusqlite::Result<IngestSource> {
            Ok(IngestSource {
                id: row.get(0)?,
                source_type: row.get(1)?,
                source_uri: row.get(2)?,
                namespace_id: row.get(3)?,
                title: row.get(4)?,
                etag: row.get(5)?,
                last_modified: row.get(6)?,
                content_hash: row.get(7)?,
                last_ingested_at: row.get(8)?,
                status: row.get(9)?,
                error_message: row.get(10)?,
                metadata_json: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        };

        let sources: Vec<IngestSource> = match namespace_id {
            Some(ns) => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, source_type, source_uri, namespace_id, title, etag, last_modified,
                            content_hash, last_ingested_at, status, error_message, metadata_json,
                            created_at, updated_at
                     FROM ingest_sources WHERE namespace_id = ? ORDER BY created_at DESC",
                )?;
                let result: Vec<IngestSource> = stmt
                    .query_map([ns], map_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                result
            }
            None => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, source_type, source_uri, namespace_id, title, etag, last_modified,
                            content_hash, last_ingested_at, status, error_message, metadata_json,
                            created_at, updated_at
                     FROM ingest_sources ORDER BY created_at DESC",
                )?;
                let result: Vec<IngestSource> = stmt
                    .query_map([], map_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                result
            }
        };

        Ok(sources)
    }

    /// Get an ingest source by ID
    pub fn get_ingest_source(&self, source_id: &str) -> Result<IngestSource, DbError> {
        self.conn
            .query_row(
                "SELECT id, source_type, source_uri, namespace_id, title, etag, last_modified,
                    content_hash, last_ingested_at, status, error_message, metadata_json,
                    created_at, updated_at
             FROM ingest_sources WHERE id = ?",
                [source_id],
                |row| {
                    Ok(IngestSource {
                        id: row.get(0)?,
                        source_type: row.get(1)?,
                        source_uri: row.get(2)?,
                        namespace_id: row.get(3)?,
                        title: row.get(4)?,
                        etag: row.get(5)?,
                        last_modified: row.get(6)?,
                        content_hash: row.get(7)?,
                        last_ingested_at: row.get(8)?,
                        status: row.get(9)?,
                        error_message: row.get(10)?,
                        metadata_json: row.get(11)?,
                        created_at: row.get(12)?,
                        updated_at: row.get(13)?,
                    })
                },
            )
            .map_err(DbError::Sqlite)
    }

    /// Find an ingest source by URI and namespace
    pub fn find_ingest_source(
        &self,
        source_type: &str,
        source_uri: &str,
        namespace_id: &str,
    ) -> Result<Option<IngestSource>, DbError> {
        match self.conn.query_row(
            "SELECT id, source_type, source_uri, namespace_id, title, etag, last_modified,
                    content_hash, last_ingested_at, status, error_message, metadata_json,
                    created_at, updated_at
             FROM ingest_sources WHERE source_type = ? AND source_uri = ? AND namespace_id = ?",
            params![source_type, source_uri, namespace_id],
            |row| {
                Ok(IngestSource {
                    id: row.get(0)?,
                    source_type: row.get(1)?,
                    source_uri: row.get(2)?,
                    namespace_id: row.get(3)?,
                    title: row.get(4)?,
                    etag: row.get(5)?,
                    last_modified: row.get(6)?,
                    content_hash: row.get(7)?,
                    last_ingested_at: row.get(8)?,
                    status: row.get(9)?,
                    error_message: row.get(10)?,
                    metadata_json: row.get(11)?,
                    created_at: row.get(12)?,
                    updated_at: row.get(13)?,
                })
            },
        ) {
            Ok(source) => Ok(Some(source)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// Save an ingest source
    pub fn save_ingest_source(&self, source: &IngestSource) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO ingest_sources (id, source_type, source_uri, namespace_id, title, etag,
                    last_modified, content_hash, last_ingested_at, status, error_message,
                    metadata_json, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                etag = excluded.etag,
                last_modified = excluded.last_modified,
                content_hash = excluded.content_hash,
                last_ingested_at = excluded.last_ingested_at,
                status = excluded.status,
                error_message = excluded.error_message,
                metadata_json = excluded.metadata_json,
                updated_at = excluded.updated_at",
            params![
                source.id,
                source.source_type,
                source.source_uri,
                source.namespace_id,
                source.title,
                source.etag,
                source.last_modified,
                source.content_hash,
                source.last_ingested_at,
                source.status,
                source.error_message,
                source.metadata_json,
                source.created_at,
                source.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Delete an ingest source
    pub fn delete_ingest_source(&self, source_id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM ingest_sources WHERE id = ?", [source_id])?;
        Ok(())
    }

    /// Update ingest source status
    pub fn update_ingest_source_status(
        &self,
        source_id: &str,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE ingest_sources SET status = ?, error_message = ?, updated_at = ? WHERE id = ?",
            params![status, error_message, now, source_id],
        )?;
        Ok(())
    }

    // ============================================================================
    // Ingest Run Methods
    // ============================================================================

    /// Create an ingest run
    pub fn create_ingest_run(&self, source_id: &str) -> Result<String, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO ingest_runs (id, source_id, started_at, status)
             VALUES (?, ?, ?, 'running')",
            params![id, source_id, now],
        )?;
        Ok(id)
    }

    /// Complete an ingest run
    pub fn complete_ingest_run(&self, completion: IngestRunCompletion<'_>) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE ingest_runs SET completed_at = ?, status = ?, documents_added = ?,
                    documents_updated = ?, documents_removed = ?, chunks_added = ?, error_message = ?
             WHERE id = ?",
            params![now, completion.status, completion.docs_added, completion.docs_updated, completion.docs_removed, completion.chunks_added, completion.error_message, completion.run_id],
        )?;
        Ok(())
    }

    /// Get recent ingest runs for a source
    pub fn get_ingest_runs(
        &self,
        source_id: &str,
        limit: usize,
    ) -> Result<Vec<IngestRun>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source_id, started_at, completed_at, status, documents_added,
                    documents_updated, documents_removed, chunks_added, error_message
             FROM ingest_runs WHERE source_id = ? ORDER BY started_at DESC LIMIT ?",
        )?;

        let runs = stmt
            .query_map(params![source_id, limit as i64], |row| {
                Ok(IngestRun {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    started_at: row.get(2)?,
                    completed_at: row.get(3)?,
                    status: row.get(4)?,
                    documents_added: row.get(5)?,
                    documents_updated: row.get(6)?,
                    documents_removed: row.get(7)?,
                    chunks_added: row.get(8)?,
                    error_message: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(runs)
    }

    // ============================================================================
    // FTS Search with Namespace Support
    // ============================================================================

    /// FTS5 search for KB chunks with namespace filtering
    pub fn fts_search_in_namespace(
        &self,
        query: &str,
        namespace_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<FtsSearchResult>, DbError> {
        let map_row = |row: &rusqlite::Row| -> rusqlite::Result<FtsSearchResult> {
            Ok(FtsSearchResult {
                chunk_id: row.get(0)?,
                document_id: row.get(1)?,
                heading_path: row.get(2)?,
                snippet: row.get(3)?,
                rank: row.get(4)?,
            })
        };

        let results: Vec<FtsSearchResult> = match namespace_id {
            Some(ns) => {
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
                    WHERE kb_fts MATCH ?1 AND kb_chunks.namespace_id = ?2
                    ORDER BY rank
                    LIMIT ?3
                    "#,
                )?;
                let result: Vec<FtsSearchResult> = stmt
                    .query_map(params![query, ns, limit as i64], map_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                result
            }
            None => {
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
                    WHERE kb_fts MATCH ?1
                    ORDER BY rank
                    LIMIT ?2
                    "#,
                )?;
                let result: Vec<FtsSearchResult> = stmt
                    .query_map(params![query, limit as i64], map_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                result
            }
        };

        Ok(results)
    }

    // ============================================================================
    // Network Allowlist Methods (SSRF Protection Override)
    // ============================================================================

    /// Check if a host is in the allowlist
    pub fn is_host_allowed(&self, host: &str) -> Result<bool, DbError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM network_allowlist WHERE ? GLOB host_pattern OR ? = host_pattern",
            params![host, host],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Add a host to the allowlist
    pub fn add_to_allowlist(&self, host_pattern: &str, reason: &str) -> Result<(), DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO network_allowlist (id, host_pattern, reason, created_at)
             VALUES (?, ?, ?, ?)",
            params![id, host_pattern, reason, now],
        )?;
        Ok(())
    }

    /// Remove a host from the allowlist
    pub fn remove_from_allowlist(&self, host_pattern: &str) -> Result<(), DbError> {
        self.conn.execute(
            "DELETE FROM network_allowlist WHERE host_pattern = ?",
            [host_pattern],
        )?;
        Ok(())
    }

    /// List all allowlist entries
    pub fn list_allowlist(&self) -> Result<Vec<AllowlistEntry>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, host_pattern, reason, created_at FROM network_allowlist ORDER BY created_at"
        )?;

        let entries = stmt
            .query_map([], |row| {
                Ok(AllowlistEntry {
                    id: row.get(0)?,
                    host_pattern: row.get(1)?,
                    reason: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    // ============================================================================
    // Job Methods
    // ============================================================================

    /// Create a new job
    pub fn create_job(&self, job: &Job) -> Result<(), DbError> {
        let metadata_json = job.metadata.as_ref().map(|m| m.to_string());
        self.conn.execute(
            "INSERT INTO jobs (id, job_type, status, created_at, updated_at, started_at, completed_at,
                    progress, progress_message, error, metadata_json)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                job.id,
                job.job_type.to_string(),
                job.status.to_string(),
                job.created_at.to_rfc3339(),
                job.updated_at.to_rfc3339(),
                job.started_at.map(|t| t.to_rfc3339()),
                job.completed_at.map(|t| t.to_rfc3339()),
                job.progress,
                job.progress_message,
                job.error,
                metadata_json,
            ],
        )?;
        Ok(())
    }

    /// Get a job by ID
    pub fn get_job(&self, job_id: &str) -> Result<Option<Job>, DbError> {
        match self.conn.query_row(
            "SELECT id, job_type, status, created_at, updated_at, started_at, completed_at,
                    progress, progress_message, error, metadata_json
             FROM jobs WHERE id = ?",
            [job_id],
            |row| {
                let status_str: String = row.get(2)?;
                let metadata_json: Option<String> = row.get(10)?;
                Ok(Job {
                    id: row.get(0)?,
                    job_type: row
                        .get::<_, String>(1)?
                        .parse::<JobType>()
                        .unwrap_or(JobType::Custom("unknown".into())),
                    status: status_str.parse::<JobStatus>().unwrap_or(JobStatus::Queued),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map(|t| t.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map(|t| t.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    started_at: row
                        .get::<_, Option<String>>(5)?
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|t| t.with_timezone(&Utc)),
                    completed_at: row
                        .get::<_, Option<String>>(6)?
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|t| t.with_timezone(&Utc)),
                    progress: row.get(7)?,
                    progress_message: row.get(8)?,
                    error: row.get(9)?,
                    metadata: metadata_json.and_then(|s| serde_json::from_str(&s).ok()),
                })
            },
        ) {
            Ok(job) => Ok(Some(job)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// List jobs, optionally filtered by status
    pub fn list_jobs(&self, status: Option<JobStatus>, limit: usize) -> Result<Vec<Job>, DbError> {
        let map_row = |row: &rusqlite::Row| -> rusqlite::Result<Job> {
            let status_str: String = row.get(2)?;
            let metadata_json: Option<String> = row.get(10)?;
            Ok(Job {
                id: row.get(0)?,
                job_type: row
                    .get::<_, String>(1)?
                    .parse::<JobType>()
                    .unwrap_or(JobType::Custom("unknown".into())),
                status: status_str.parse::<JobStatus>().unwrap_or(JobStatus::Queued),
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map(|t| t.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map(|t| t.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                started_at: row
                    .get::<_, Option<String>>(5)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|t| t.with_timezone(&Utc)),
                completed_at: row
                    .get::<_, Option<String>>(6)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|t| t.with_timezone(&Utc)),
                progress: row.get(7)?,
                progress_message: row.get(8)?,
                error: row.get(9)?,
                metadata: metadata_json.and_then(|s| serde_json::from_str(&s).ok()),
            })
        };

        let jobs: Vec<Job> = match status {
            Some(s) => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, job_type, status, created_at, updated_at, started_at, completed_at,
                            progress, progress_message, error, metadata_json
                     FROM jobs WHERE status = ? ORDER BY created_at DESC LIMIT ?",
                )?;
                let result: Vec<Job> = stmt
                    .query_map(params![s.to_string(), limit as i64], map_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                result
            }
            None => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, job_type, status, created_at, updated_at, started_at, completed_at,
                            progress, progress_message, error, metadata_json
                     FROM jobs ORDER BY created_at DESC LIMIT ?",
                )?;
                let result: Vec<Job> = stmt
                    .query_map(params![limit as i64], map_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                result
            }
        };

        Ok(jobs)
    }

    /// Update job status
    pub fn update_job_status(
        &self,
        job_id: &str,
        status: JobStatus,
        error: Option<&str>,
    ) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        let started_at = if status == JobStatus::Running {
            Some(now.clone())
        } else {
            None
        };
        let completed_at = if status.is_terminal() {
            Some(now.clone())
        } else {
            None
        };

        self.conn.execute(
            "UPDATE jobs SET status = ?, updated_at = ?, started_at = COALESCE(?, started_at),
                    completed_at = COALESCE(?, completed_at), error = COALESCE(?, error)
             WHERE id = ?",
            params![
                status.to_string(),
                now,
                started_at,
                completed_at,
                error,
                job_id
            ],
        )?;
        Ok(())
    }

    /// Update job progress
    pub fn update_job_progress(
        &self,
        job_id: &str,
        progress: f32,
        message: Option<&str>,
    ) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE jobs SET progress = ?, progress_message = ?, updated_at = ? WHERE id = ?",
            params![progress, message, now, job_id],
        )?;
        Ok(())
    }

    /// Add a log entry for a job
    pub fn add_job_log(
        &self,
        job_id: &str,
        level: LogLevel,
        message: &str,
    ) -> Result<i64, DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO job_logs (job_id, timestamp, level, message) VALUES (?, ?, ?, ?)",
            params![job_id, now, level.to_string(), message],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get logs for a job
    pub fn get_job_logs(&self, job_id: &str, limit: usize) -> Result<Vec<JobLog>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, job_id, timestamp, level, message
             FROM job_logs WHERE job_id = ? ORDER BY timestamp DESC LIMIT ?",
        )?;

        let logs = stmt
            .query_map(params![job_id, limit as i64], |row| {
                let level_str: String = row.get(3)?;
                let level = match level_str.as_str() {
                    "debug" => LogLevel::Debug,
                    "info" => LogLevel::Info,
                    "warning" => LogLevel::Warning,
                    "error" => LogLevel::Error,
                    _ => LogLevel::Info,
                };
                Ok(JobLog {
                    id: row.get(0)?,
                    job_id: row.get(1)?,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .map(|t| t.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    level,
                    message: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    /// Delete old completed jobs
    pub fn cleanup_old_jobs(&self, keep_days: i64) -> Result<usize, DbError> {
        let cutoff = (Utc::now() - chrono::Duration::days(keep_days)).to_rfc3339();
        let deleted = self.conn.execute(
            "DELETE FROM jobs WHERE status IN ('succeeded', 'failed', 'cancelled')
             AND completed_at < ?",
            params![cutoff],
        )?;
        Ok(deleted)
    }

    /// Get count of jobs by status
    pub fn get_job_counts(&self) -> Result<Vec<(String, i64)>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT status, COUNT(*) FROM jobs GROUP BY status")?;

        let counts = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(counts)
    }

    // ============================================================================
    // Document Versioning Methods (Phase 14)
    // ============================================================================

    /// Create a version snapshot of a document before updating it
    pub fn create_document_version(
        &self,
        document_id: &str,
        change_reason: Option<&str>,
    ) -> Result<String, DbError> {
        // Get current document state
        let (file_hash,): (String,) = self.conn.query_row(
            "SELECT file_hash FROM kb_documents WHERE id = ?",
            [document_id],
            |row| Ok((row.get(0)?,)),
        )?;

        // Get current chunks as JSON
        let mut stmt = self.conn.prepare(
            "SELECT id, chunk_index, heading_path, content, word_count
             FROM kb_chunks WHERE document_id = ? ORDER BY chunk_index",
        )?;

        let chunks: Vec<serde_json::Value> = stmt
            .query_map([document_id], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "chunk_index": row.get::<_, i32>(1)?,
                    "heading_path": row.get::<_, Option<String>>(2)?,
                    "content": row.get::<_, String>(3)?,
                    "word_count": row.get::<_, Option<i32>>(4)?,
                }))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let chunks_json = serde_json::to_string(&chunks)
            .map_err(|e| DbError::Sqlite(rusqlite::Error::InvalidParameterName(e.to_string())))?;

        // Get next version number
        let version_number: i32 = self.conn
            .query_row(
                "SELECT COALESCE(MAX(version_number), 0) + 1 FROM document_versions WHERE document_id = ?",
                [document_id],
                |row| row.get(0),
            )
            .unwrap_or(1);

        // Insert version
        let version_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO document_versions (id, document_id, version_number, file_hash, chunks_json, created_at, change_reason)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![version_id, document_id, version_number, file_hash, chunks_json, now, change_reason],
        )?;

        Ok(version_id)
    }

    /// List versions of a document
    pub fn list_document_versions(
        &self,
        document_id: &str,
    ) -> Result<Vec<DocumentVersion>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, document_id, version_number, file_hash, created_at, change_reason
             FROM document_versions WHERE document_id = ? ORDER BY version_number DESC",
        )?;

        let versions = stmt
            .query_map([document_id], |row| {
                Ok(DocumentVersion {
                    id: row.get(0)?,
                    document_id: row.get(1)?,
                    version_number: row.get(2)?,
                    file_hash: row.get(3)?,
                    created_at: row.get(4)?,
                    change_reason: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(versions)
    }

    /// Rollback a document to a previous version
    pub fn rollback_document(&self, document_id: &str, version_id: &str) -> Result<(), DbError> {
        // Get the version
        let (chunks_json, file_hash, _version_number): (String, String, i32) = self.conn.query_row(
            "SELECT chunks_json, file_hash, version_number FROM document_versions WHERE id = ? AND document_id = ?",
            params![version_id, document_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

        // Create a new version of current state before rollback
        let _ = self.create_document_version(document_id, Some("Pre-rollback snapshot"));

        // Delete current chunks
        self.conn
            .execute("DELETE FROM kb_chunks WHERE document_id = ?", [document_id])?;

        // Parse and restore chunks
        let chunks: Vec<serde_json::Value> = serde_json::from_str(&chunks_json)
            .map_err(|e| DbError::Sqlite(rusqlite::Error::InvalidParameterName(e.to_string())))?;

        let namespace_id: String = self.conn.query_row(
            "SELECT namespace_id FROM kb_documents WHERE id = ?",
            [document_id],
            |row| row.get(0),
        )?;

        for chunk in chunks {
            let chunk_id = uuid::Uuid::new_v4().to_string();
            self.conn.execute(
                "INSERT INTO kb_chunks (id, document_id, chunk_index, heading_path, content, word_count, namespace_id)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![
                    chunk_id,
                    document_id,
                    chunk["chunk_index"].as_i64().unwrap_or(0) as i32,
                    chunk["heading_path"].as_str(),
                    chunk["content"].as_str().unwrap_or(""),
                    chunk["word_count"].as_i64().map(|v| v as i32),
                    namespace_id,
                ],
            )?;
        }

        // Update document hash
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE kb_documents SET file_hash = ?, indexed_at = ? WHERE id = ?",
            params![file_hash, now, document_id],
        )?;

        Ok(())
    }

    // ============================================================================
    // Source Trust and Curation Methods (Phase 14)
    // ============================================================================

    /// Update trust score for a source
    pub fn update_source_trust(&self, source_id: &str, trust_score: f64) -> Result<(), DbError> {
        let score = trust_score.clamp(0.0, 1.0);
        self.conn.execute(
            "UPDATE ingest_sources SET trust_score = ? WHERE id = ?",
            params![score, source_id],
        )?;
        Ok(())
    }

    /// Pin/unpin a source (boosts search results)
    pub fn set_source_pinned(&self, source_id: &str, pinned: bool) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE ingest_sources SET is_pinned = ? WHERE id = ?",
            params![pinned as i32, source_id],
        )?;
        Ok(())
    }

    /// Update review status for a source
    pub fn set_source_review_status(&self, source_id: &str, status: &str) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE ingest_sources SET review_status = ? WHERE id = ?",
            params![status, source_id],
        )?;
        Ok(())
    }

    /// Mark sources as stale based on threshold
    pub fn mark_stale_sources(&self, days_threshold: i64) -> Result<usize, DbError> {
        let now = Utc::now();
        let cutoff = (now - chrono::Duration::days(days_threshold)).to_rfc3339();
        let stale_at = now.to_rfc3339();

        let count = self.conn.execute(
            "UPDATE ingest_sources SET status = 'stale', stale_at = ?
             WHERE status = 'active'
             AND last_ingested_at IS NOT NULL
             AND last_ingested_at < ?",
            params![stale_at, cutoff],
        )?;

        Ok(count)
    }

    /// Get stale sources for review
    pub fn get_stale_sources(
        &self,
        namespace_id: Option<&str>,
    ) -> Result<Vec<IngestSource>, DbError> {
        let map_row = |row: &rusqlite::Row| -> rusqlite::Result<IngestSource> {
            Ok(IngestSource {
                id: row.get(0)?,
                source_type: row.get(1)?,
                source_uri: row.get(2)?,
                namespace_id: row.get(3)?,
                title: row.get(4)?,
                etag: row.get(5)?,
                last_modified: row.get(6)?,
                content_hash: row.get(7)?,
                last_ingested_at: row.get(8)?,
                status: row.get(9)?,
                error_message: row.get(10)?,
                metadata_json: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        };

        let sources: Vec<IngestSource> = match namespace_id {
            Some(ns) => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, source_type, source_uri, namespace_id, title, etag, last_modified,
                            content_hash, last_ingested_at, status, error_message, metadata_json, created_at, updated_at
                     FROM ingest_sources WHERE status = 'stale' AND namespace_id = ? ORDER BY stale_at"
                )?;
                let result: Vec<IngestSource> = stmt
                    .query_map([ns], map_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                result
            }
            None => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, source_type, source_uri, namespace_id, title, etag, last_modified,
                            content_hash, last_ingested_at, status, error_message, metadata_json, created_at, updated_at
                     FROM ingest_sources WHERE status = 'stale' ORDER BY stale_at"
                )?;
                let result: Vec<IngestSource> = stmt
                    .query_map([], map_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                result
            }
        };

        Ok(sources)
    }

    // ============================================================================
    // Namespace Rules Methods (Phase 14)
    // ============================================================================

    /// Add a namespace ingestion rule
    pub fn add_namespace_rule(
        &self,
        namespace_id: &str,
        rule_type: &str,
        pattern_type: &str,
        pattern: &str,
        reason: Option<&str>,
    ) -> Result<String, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO namespace_rules (id, namespace_id, rule_type, pattern_type, pattern, reason, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![id, namespace_id, rule_type, pattern_type, pattern, reason, now],
        )?;

        Ok(id)
    }

    /// Delete a namespace rule
    pub fn delete_namespace_rule(&self, rule_id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM namespace_rules WHERE id = ?", [rule_id])?;
        Ok(())
    }

    /// List rules for a namespace
    pub fn list_namespace_rules(&self, namespace_id: &str) -> Result<Vec<NamespaceRule>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, namespace_id, rule_type, pattern_type, pattern, reason, created_at
             FROM namespace_rules WHERE namespace_id = ? ORDER BY created_at",
        )?;

        let rules = stmt
            .query_map([namespace_id], |row| {
                Ok(NamespaceRule {
                    id: row.get(0)?,
                    namespace_id: row.get(1)?,
                    rule_type: row.get(2)?,
                    pattern_type: row.get(3)?,
                    pattern: row.get(4)?,
                    reason: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rules)
    }

    /// Check if a URL/path is allowed by namespace rules
    pub fn check_namespace_rules(
        &self,
        namespace_id: &str,
        url_or_path: &str,
    ) -> Result<bool, DbError> {
        let rules = self.list_namespace_rules(namespace_id)?;

        for rule in rules {
            let matches = match rule.pattern_type.as_str() {
                "domain" => {
                    if let Ok(parsed) = url::Url::parse(url_or_path) {
                        parsed
                            .host_str()
                            .map(|h| h.contains(&rule.pattern))
                            .unwrap_or(false)
                    } else {
                        false
                    }
                }
                "file_pattern" | "url_pattern" => {
                    // Simple glob-like pattern matching
                    let pattern = rule.pattern.replace("*", "");
                    url_or_path.contains(&pattern)
                }
                _ => false,
            };

            if matches {
                return Ok(rule.rule_type == "allow");
            }
        }

        // No matching rule = allowed by default
        Ok(true)
    }

    // ============================================================================
    // KB Document Methods with Namespace Support
    // ============================================================================

    /// Get documents, optionally filtered by namespace and/or source
    pub fn list_kb_documents(
        &self,
        namespace_id: Option<&str>,
        source_id: Option<&str>,
    ) -> Result<Vec<KbDocument>, DbError> {
        let map_row = |row: &rusqlite::Row| -> rusqlite::Result<KbDocument> {
            Ok(KbDocument {
                id: row.get(0)?,
                file_path: row.get(1)?,
                file_hash: row.get(2)?,
                title: row.get(3)?,
                indexed_at: row.get(4)?,
                chunk_count: row.get(5)?,
                ocr_quality: row.get(6)?,
                partial_index: row.get::<_, Option<i32>>(7)?.map(|v| v != 0),
                namespace_id: row.get(8)?,
                source_type: row.get(9)?,
                source_id: row.get(10)?,
            })
        };

        let docs: Vec<KbDocument> = match (namespace_id, source_id) {
            (Some(ns), Some(src)) => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, file_path, file_hash, title, indexed_at, chunk_count, ocr_quality,
                            partial_index, namespace_id, source_type, source_id
                     FROM kb_documents WHERE namespace_id = ? AND source_id = ? ORDER BY indexed_at DESC"
                )?;
                let result: Vec<KbDocument> = stmt
                    .query_map(params![ns, src], map_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                result
            }
            (Some(ns), None) => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, file_path, file_hash, title, indexed_at, chunk_count, ocr_quality,
                            partial_index, namespace_id, source_type, source_id
                     FROM kb_documents WHERE namespace_id = ? ORDER BY indexed_at DESC",
                )?;
                let result: Vec<KbDocument> = stmt
                    .query_map(params![ns], map_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                result
            }
            (None, Some(src)) => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, file_path, file_hash, title, indexed_at, chunk_count, ocr_quality,
                            partial_index, namespace_id, source_type, source_id
                     FROM kb_documents WHERE source_id = ? ORDER BY indexed_at DESC",
                )?;
                let result: Vec<KbDocument> = stmt
                    .query_map(params![src], map_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                result
            }
            (None, None) => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, file_path, file_hash, title, indexed_at, chunk_count, ocr_quality,
                            partial_index, namespace_id, source_type, source_id
                     FROM kb_documents ORDER BY indexed_at DESC",
                )?;
                let result: Vec<KbDocument> = stmt
                    .query_map([], map_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                result
            }
        };

        Ok(docs)
    }

    /// Delete all documents for a source
    pub fn delete_documents_for_source(&self, source_id: &str) -> Result<usize, DbError> {
        let deleted = self
            .conn
            .execute("DELETE FROM kb_documents WHERE source_id = ?", [source_id])?;
        Ok(deleted)
    }

    /// Get document count by namespace
    pub fn get_document_count_by_namespace(&self) -> Result<Vec<(String, i64)>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT namespace_id, COUNT(*) FROM kb_documents GROUP BY namespace_id")?;

        let counts = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(counts)
    }

    // ========================================================================
    // Phase 4: Response Ratings
    // ========================================================================

    /// Save or update a response rating for a draft
    pub fn save_response_rating(
        &self,
        id: &str,
        draft_id: &str,
        rating: i32,
        feedback_text: Option<&str>,
        feedback_category: Option<&str>,
    ) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO response_ratings (id, draft_id, rating, feedback_text, feedback_category, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![id, draft_id, rating, feedback_text, feedback_category, &now],
        )?;
        Ok(())
    }

    /// Get the rating for a specific draft
    pub fn get_draft_rating(&self, draft_id: &str) -> Result<Option<ResponseRating>, DbError> {
        let result = self.conn.query_row(
            "SELECT id, draft_id, rating, feedback_text, feedback_category, created_at
             FROM response_ratings WHERE draft_id = ? LIMIT 1",
            [draft_id],
            |row| {
                Ok(ResponseRating {
                    id: row.get(0)?,
                    draft_id: row.get(1)?,
                    rating: row.get(2)?,
                    feedback_text: row.get(3)?,
                    feedback_category: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        );

        match result {
            Ok(r) => Ok(Some(r)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// Get aggregate rating statistics
    pub fn get_rating_stats(&self) -> Result<RatingStats, DbError> {
        let total_ratings: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM response_ratings", [], |row| {
                    row.get(0)
                })?;

        let average_rating: f64 = if total_ratings > 0 {
            self.conn.query_row(
                "SELECT AVG(CAST(rating AS REAL)) FROM response_ratings",
                [],
                |row| row.get(0),
            )?
        } else {
            0.0
        };

        let mut distribution = vec![0i64; 5];
        let mut stmt = self.conn.prepare(
            "SELECT rating, COUNT(*) FROM response_ratings GROUP BY rating ORDER BY rating",
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, i32>(0)?, row.get::<_, i64>(1)?)))?;

        for row in rows {
            let (rating, count) = row?;
            if (1..=5).contains(&rating) {
                distribution[(rating - 1) as usize] = count;
            }
        }

        Ok(RatingStats {
            total_ratings,
            average_rating,
            distribution,
        })
    }

    // ========================================================================
    // Phase 2: Analytics Events
    // ========================================================================

    /// Log an analytics event
    pub fn log_analytics_event(
        &self,
        id: &str,
        event_type: &str,
        event_data_json: Option<&str>,
    ) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO analytics_events (id, event_type, event_data_json, created_at)
             VALUES (?, ?, ?, ?)",
            params![id, event_type, event_data_json, &now],
        )?;
        Ok(())
    }

    /// Get analytics summary for a given period (None = all time)
    pub fn get_analytics_summary(
        &self,
        period_days: Option<i64>,
    ) -> Result<AnalyticsSummary, DbError> {
        let date_filter = period_days
            .map(|d| format!("AND created_at >= datetime('now', '-{} days')", d))
            .unwrap_or_default();

        let total_events: i64 = self.conn.query_row(
            &format!(
                "SELECT COUNT(*) FROM analytics_events WHERE 1=1 {}",
                date_filter
            ),
            [],
            |row| row.get(0),
        )?;

        let responses_generated: i64 = self.conn.query_row(
            &format!(
                "SELECT COUNT(*) FROM analytics_events WHERE event_type = 'response_generated' {}",
                date_filter
            ),
            [],
            |row| row.get(0),
        )?;

        let searches_performed: i64 = self.conn.query_row(
            &format!(
                "SELECT COUNT(*) FROM analytics_events WHERE event_type = 'search_performed' {}",
                date_filter
            ),
            [],
            |row| row.get(0),
        )?;

        let drafts_saved: i64 = self.conn.query_row(
            &format!(
                "SELECT COUNT(*) FROM analytics_events WHERE event_type = 'draft_saved' {}",
                date_filter
            ),
            [],
            |row| row.get(0),
        )?;

        // Daily counts for the period
        let daily_query = format!(
            "SELECT DATE(created_at) as day, COUNT(*) FROM analytics_events
             WHERE 1=1 {}
             GROUP BY day ORDER BY day DESC LIMIT 30",
            date_filter
        );
        let mut stmt = self.conn.prepare(&daily_query)?;
        let daily_counts = stmt
            .query_map([], |row| {
                Ok(DailyCount {
                    date: row.get(0)?,
                    count: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Rating stats for the period
        let rating_date_filter = period_days
            .map(|d| format!("WHERE created_at >= datetime('now', '-{} days')", d))
            .unwrap_or_default();

        let total_ratings: i64 = self.conn.query_row(
            &format!(
                "SELECT COUNT(*) FROM response_ratings {}",
                rating_date_filter
            ),
            [],
            |row| row.get(0),
        )?;

        let average_rating: f64 = if total_ratings > 0 {
            self.conn.query_row(
                &format!(
                    "SELECT AVG(CAST(rating AS REAL)) FROM response_ratings {}",
                    rating_date_filter
                ),
                [],
                |row| row.get(0),
            )?
        } else {
            0.0
        };

        Ok(AnalyticsSummary {
            total_events,
            responses_generated,
            searches_performed,
            drafts_saved,
            daily_counts,
            average_rating,
            total_ratings,
        })
    }

    /// Get KB article usage stats from analytics events
    pub fn get_kb_usage_stats(
        &self,
        period_days: Option<i64>,
    ) -> Result<Vec<ArticleUsage>, DbError> {
        let date_filter = period_days
            .map(|d| format!("AND ae.created_at >= datetime('now', '-{} days')", d))
            .unwrap_or_default();

        // Parse event_data_json to extract document_id from kb_article_used events
        let query = format!(
            "SELECT
                json_extract(ae.event_data_json, '$.document_id') as doc_id,
                COALESCE(json_extract(ae.event_data_json, '$.title'), 'Unknown') as title,
                COUNT(*) as usage_count
             FROM analytics_events ae
             WHERE ae.event_type = 'kb_article_used'
               AND json_extract(ae.event_data_json, '$.document_id') IS NOT NULL
               {}
             GROUP BY doc_id
             ORDER BY usage_count DESC
             LIMIT 50",
            date_filter
        );

        let mut stmt = self.conn.prepare(&query)?;
        let results = stmt
            .query_map([], |row| {
                Ok(ArticleUsage {
                    document_id: row.get(0)?,
                    title: row.get(1)?,
                    usage_count: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    // ========================================================================
    // Phase 10: KB Management
    // ========================================================================

    /// Update the content of a KB chunk
    pub fn update_chunk_content(&self, chunk_id: &str, content: &str) -> Result<(), DbError> {
        let word_count = content.split_whitespace().count() as i32;
        let rows = self.conn.execute(
            "UPDATE kb_chunks SET content = ?, word_count = ? WHERE id = ?",
            params![content, word_count, chunk_id],
        )?;
        if rows == 0 {
            return Err(DbError::Migration(format!("Chunk not found: {}", chunk_id)));
        }
        Ok(())
    }

    /// Get KB health statistics
    pub fn get_kb_health_stats(&self) -> Result<KbHealthStats, DbError> {
        let total_documents: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM kb_documents", [], |row| row.get(0))?;

        let total_chunks: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM kb_chunks", [], |row| row.get(0))?;

        let stale_documents: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM kb_documents
             WHERE indexed_at < datetime('now', '-30 days')
                OR indexed_at IS NULL",
            [],
            |row| row.get(0),
        )?;

        let mut stmt = self.conn.prepare(
            "SELECT n.id, n.name,
                    COUNT(DISTINCT d.id) as doc_count,
                    COUNT(c.id) as chunk_count
             FROM namespaces n
             LEFT JOIN kb_documents d ON d.namespace_id = n.id
             LEFT JOIN kb_chunks c ON c.document_id = d.id
             GROUP BY n.id
             ORDER BY n.name",
        )?;

        let namespace_distribution = stmt
            .query_map([], |row| {
                Ok(NamespaceDistribution {
                    namespace_id: row.get(0)?,
                    namespace_name: row.get(1)?,
                    document_count: row.get(2)?,
                    chunk_count: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(KbHealthStats {
            total_documents,
            total_chunks,
            stale_documents,
            namespace_distribution,
        })
    }

    // ========================================================================
    // Phase 6: Draft Version Restore
    // ========================================================================

    /// Restore a draft to a previous version
    pub fn restore_draft_version(&self, draft_id: &str, version_id: &str) -> Result<(), DbError> {
        // First, create a snapshot of the current state before restoring
        self.create_draft_version(draft_id, Some("Pre-restore snapshot"))?;

        // Get the version data to restore
        let version = self.conn.query_row(
            "SELECT input_text, summary_text, response_text, case_intake_json, kb_sources_json
             FROM draft_versions WHERE id = ? AND draft_id = ?",
            params![version_id, draft_id],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            },
        )?;

        let now = Utc::now().to_rfc3339();

        // Update the draft with the version's data
        self.conn.execute(
            "UPDATE drafts SET
                input_text = COALESCE(?, input_text),
                summary_text = ?,
                response_text = ?,
                case_intake_json = ?,
                kb_sources_json = ?,
                updated_at = ?
             WHERE id = ?",
            params![version.0, version.1, version.2, version.3, version.4, &now, draft_id,],
        )?;

        // Create a new version snapshot after restoring
        self.create_draft_version(draft_id, Some("Restored from version"))?;

        Ok(())
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
/// Draft status for workflow lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DraftStatus {
    #[default]
    Draft,
    Finalized,
    Archived,
}

impl std::fmt::Display for DraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DraftStatus::Draft => write!(f, "draft"),
            DraftStatus::Finalized => write!(f, "finalized"),
            DraftStatus::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for DraftStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(DraftStatus::Draft),
            "finalized" => Ok(DraftStatus::Finalized),
            "archived" => Ok(DraftStatus::Archived),
            _ => Err(format!("Unknown draft status: {}", s)),
        }
    }
}

/// Case intake data for structured IT support workflow
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CaseIntake {
    /// Affected user name or ID
    pub user: Option<String>,
    /// Device type/model
    pub device: Option<String>,
    /// Operating system and version
    pub os: Option<String>,
    /// Urgency level: low, medium, high, critical
    pub urgency: Option<String>,
    /// Symptom description
    pub symptoms: Option<String>,
    /// Steps to reproduce
    pub reproduction: Option<String>,
    /// Relevant log snippets
    pub logs: Option<String>,
    /// Additional context fields (custom)
    #[serde(default)]
    pub custom_fields: std::collections::HashMap<String, String>,
}

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
    /// Structured case intake data (Phase 17)
    #[serde(default)]
    pub case_intake_json: Option<String>,
    /// Draft lifecycle status
    #[serde(default)]
    pub status: DraftStatus,
    /// Handoff summary for escalations
    #[serde(default)]
    pub handoff_summary: Option<String>,
    /// When the draft was finalized
    #[serde(default)]
    pub finalized_at: Option<String>,
    /// Who finalized the draft
    #[serde(default)]
    pub finalized_by: Option<String>,
}

/// Draft version for history/diff view
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DraftVersion {
    pub id: String,
    pub draft_id: String,
    pub version_number: i32,
    pub input_text: Option<String>,
    pub summary_text: Option<String>,
    pub response_text: Option<String>,
    pub case_intake_json: Option<String>,
    pub kb_sources_json: Option<String>,
    pub created_at: String,
    pub change_reason: Option<String>,
}

/// Playbook: curated workflow tied to decision trees
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Playbook {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub decision_tree_id: Option<String>,
    pub steps_json: String,
    pub template_id: Option<String>,
    pub shortcuts_json: Option<String>,
    pub is_active: bool,
    pub usage_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Action shortcut for one-click sequences
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActionShortcut {
    pub id: String,
    pub name: String,
    pub shortcut_key: Option<String>,
    pub action_type: String,
    pub action_data_json: String,
    pub category: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
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

/// Namespace for organizing content
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Namespace {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Namespace with document and source counts (optimized query result)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NamespaceWithCounts {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub document_count: i64,
    pub source_count: i64,
}

/// Ingest source (web URL, YouTube video, GitHub repo, etc.)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IngestSource {
    pub id: String,
    pub source_type: String,
    pub source_uri: String,
    pub namespace_id: String,
    pub title: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub content_hash: Option<String>,
    pub last_ingested_at: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub metadata_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Document version for rollback support (Phase 14)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentVersion {
    pub id: String,
    pub document_id: String,
    pub version_number: i32,
    pub file_hash: String,
    pub created_at: String,
    pub change_reason: Option<String>,
}

/// Namespace ingestion rule (Phase 14)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NamespaceRule {
    pub id: String,
    pub namespace_id: String,
    pub rule_type: String,
    pub pattern_type: String,
    pub pattern: String,
    pub reason: Option<String>,
    pub created_at: String,
}

/// Ingest run (tracks a single ingest operation)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IngestRun {
    pub id: String,
    pub source_id: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub status: String,
    pub documents_added: Option<i32>,
    pub documents_updated: Option<i32>,
    pub documents_removed: Option<i32>,
    pub chunks_added: Option<i32>,
    pub error_message: Option<String>,
}

/// Parameters for completing an ingest run (avoids too-many-arguments)
pub struct IngestRunCompletion<'a> {
    pub run_id: &'a str,
    pub status: &'a str,
    pub docs_added: i32,
    pub docs_updated: i32,
    pub docs_removed: i32,
    pub chunks_added: i32,
    pub error_message: Option<&'a str>,
}

/// Network allowlist entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AllowlistEntry {
    pub id: String,
    pub host_pattern: String,
    pub reason: Option<String>,
    pub created_at: String,
}

/// KB Document with namespace support
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KbDocument {
    pub id: String,
    pub file_path: String,
    pub file_hash: String,
    pub title: Option<String>,
    pub indexed_at: Option<String>,
    pub chunk_count: Option<i32>,
    pub ocr_quality: Option<String>,
    pub partial_index: Option<bool>,
    pub namespace_id: String,
    pub source_type: String,
    pub source_id: Option<String>,
}

/// Response rating for a draft
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseRating {
    pub id: String,
    pub draft_id: String,
    pub rating: i32,
    pub feedback_text: Option<String>,
    pub feedback_category: Option<String>,
    pub created_at: String,
}

/// Aggregate rating statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RatingStats {
    pub total_ratings: i64,
    pub average_rating: f64,
    pub distribution: Vec<i64>,
}

/// Analytics summary for a time period
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalyticsSummary {
    pub total_events: i64,
    pub responses_generated: i64,
    pub searches_performed: i64,
    pub drafts_saved: i64,
    pub daily_counts: Vec<DailyCount>,
    pub average_rating: f64,
    pub total_ratings: i64,
}

/// Daily event count
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DailyCount {
    pub date: String,
    pub count: i64,
}

/// KB article usage statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArticleUsage {
    pub document_id: String,
    pub title: String,
    pub usage_count: i64,
}

/// KB health statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KbHealthStats {
    pub total_documents: i64,
    pub total_chunks: i64,
    pub stale_documents: i64,
    pub namespace_distribution: Vec<NamespaceDistribution>,
}

/// Namespace distribution in KB health
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NamespaceDistribution {
    pub namespace_id: String,
    pub namespace_name: String,
    pub document_count: i64,
    pub chunk_count: i64,
}

/// Built-in decision trees: (id, name, category, tree_json)
const BUILTIN_TREES: &[(&str, &str, &str, &str)] = &[
    (
        "auth-issues",
        "Authentication Issues",
        "Security",
        include_str!("../trees/auth.json"),
    ),
    (
        "vpn-connectivity",
        "VPN Connectivity",
        "Network",
        include_str!("../trees/vpn.json"),
    ),
    (
        "email-calendar",
        "Email & Calendar",
        "Productivity",
        include_str!("../trees/email.json"),
    ),
    (
        "password-reset",
        "Password Reset",
        "Security",
        include_str!("../trees/password.json"),
    ),
];

/// Get the application data directory
pub fn get_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .expect("Platform data directory must be available")
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
        .expect("Platform data directory must be available")
        .join("Logs")
        .join("AssistSupport")
}

/// Get cache directory
pub fn get_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .expect("Platform cache directory must be available")
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

    #[test]
    fn test_job_crud() {
        let (db, _dir) = create_test_db();

        // Create a job
        let job = Job::new(JobType::IngestWeb);
        let job_id = job.id.clone();
        db.create_job(&job).unwrap();

        // Get the job
        let retrieved = db.get_job(&job_id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, job_id);
        assert_eq!(retrieved.status, JobStatus::Queued);

        // Update status to running
        db.update_job_status(&job_id, JobStatus::Running, None)
            .unwrap();
        let retrieved = db.get_job(&job_id).unwrap().unwrap();
        assert_eq!(retrieved.status, JobStatus::Running);
        assert!(retrieved.started_at.is_some());

        // Update progress
        db.update_job_progress(&job_id, 0.5, Some("Halfway done"))
            .unwrap();
        let retrieved = db.get_job(&job_id).unwrap().unwrap();
        assert_eq!(retrieved.progress, 0.5);
        assert_eq!(retrieved.progress_message, Some("Halfway done".to_string()));

        // Complete the job
        db.update_job_status(&job_id, JobStatus::Succeeded, None)
            .unwrap();
        let retrieved = db.get_job(&job_id).unwrap().unwrap();
        assert_eq!(retrieved.status, JobStatus::Succeeded);
        assert!(retrieved.completed_at.is_some());
    }

    #[test]
    fn test_job_logs() {
        let (db, _dir) = create_test_db();

        // Create a job
        let job = Job::new(JobType::IngestYoutube);
        let job_id = job.id.clone();
        db.create_job(&job).unwrap();

        // Add logs
        db.add_job_log(&job_id, LogLevel::Info, "Starting ingestion")
            .unwrap();
        db.add_job_log(&job_id, LogLevel::Debug, "Fetching content")
            .unwrap();
        db.add_job_log(&job_id, LogLevel::Warning, "Content is large")
            .unwrap();
        db.add_job_log(&job_id, LogLevel::Info, "Completed")
            .unwrap();

        // Get logs
        let logs = db.get_job_logs(&job_id, 10).unwrap();
        assert_eq!(logs.len(), 4);

        // Logs should be in reverse order (most recent first)
        assert_eq!(logs[0].message, "Completed");
        assert_eq!(logs[3].message, "Starting ingestion");
    }

    #[test]
    fn test_list_jobs_by_status() {
        let (db, _dir) = create_test_db();

        // Create jobs with different statuses
        let job1 = Job::new(JobType::IngestWeb);
        let job2 = Job::new(JobType::IngestYoutube);
        let job3 = Job::new(JobType::IndexKb);

        db.create_job(&job1).unwrap();
        db.create_job(&job2).unwrap();
        db.create_job(&job3).unwrap();

        // Update statuses
        db.update_job_status(&job1.id, JobStatus::Running, None)
            .unwrap();
        db.update_job_status(&job2.id, JobStatus::Succeeded, None)
            .unwrap();

        // List all jobs
        let all_jobs = db.list_jobs(None, 10).unwrap();
        assert_eq!(all_jobs.len(), 3);

        // List only queued jobs
        let queued = db.list_jobs(Some(JobStatus::Queued), 10).unwrap();
        assert_eq!(queued.len(), 1);

        // List only running jobs
        let running = db.list_jobs(Some(JobStatus::Running), 10).unwrap();
        assert_eq!(running.len(), 1);

        // List only succeeded jobs
        let succeeded = db.list_jobs(Some(JobStatus::Succeeded), 10).unwrap();
        assert_eq!(succeeded.len(), 1);
    }

    #[test]
    fn test_job_counts() {
        let (db, _dir) = create_test_db();

        // Create jobs
        let job1 = Job::new(JobType::IngestWeb);
        let job2 = Job::new(JobType::IngestYoutube);
        let job3 = Job::new(JobType::IndexKb);

        db.create_job(&job1).unwrap();
        db.create_job(&job2).unwrap();
        db.create_job(&job3).unwrap();

        db.update_job_status(&job1.id, JobStatus::Succeeded, None)
            .unwrap();
        db.update_job_status(&job2.id, JobStatus::Failed, Some("Test error"))
            .unwrap();

        // Get counts
        let counts = db.get_job_counts().unwrap();
        assert!(!counts.is_empty());

        // Check failed job has error
        let failed_job = db.get_job(&job2.id).unwrap().unwrap();
        assert_eq!(failed_job.error, Some("Test error".to_string()));
    }
}
