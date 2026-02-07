//! Disk folder ingestion module for AssistSupport
//! Wraps KbIndexer with ingest source/run tracking so disk-indexed
//! articles appear in the source management UI.

use super::{IngestError, IngestResult, IngestedDocument};
use crate::db::{Database, IngestRunCompletion, IngestSource};
use crate::kb::indexer::KbIndexer;
use std::path::Path;

/// Result of a disk folder ingestion
#[derive(Debug, Clone)]
pub struct DiskIngestResult {
    pub total_files: usize,
    pub ingested: usize,
    pub skipped: usize,
    pub errors: usize,
    pub documents: Vec<IngestedDocument>,
}

/// Disk folder ingester with ingest source/run tracking
pub struct DiskIngester {
    indexer: KbIndexer,
}

impl DiskIngester {
    pub fn new() -> Self {
        Self {
            indexer: KbIndexer::new(),
        }
    }

    /// Ingest a folder into the knowledge base with source tracking.
    ///
    /// Creates `IngestSource` (source_type="file") and `IngestRun` entries
    /// for each file, so disk-indexed articles appear in the source management UI.
    /// Uses file hash comparison for incremental re-ingestion.
    pub fn ingest_folder(
        &self,
        db: &Database,
        folder: &Path,
        namespace_id: &str,
    ) -> IngestResult<DiskIngestResult> {
        let files = self
            .indexer
            .scan_folder(folder)
            .map_err(|e| IngestError::Io(std::io::Error::other(e.to_string())))?;
        let total_files = files.len();

        let mut ingested = 0;
        let mut skipped = 0;
        let mut errors = 0;
        let mut documents = Vec::new();

        for file_path in &files {
            match self.ingest_file(db, file_path, namespace_id) {
                Ok(Some(doc)) => {
                    ingested += 1;
                    documents.push(doc);
                }
                Ok(None) => {
                    skipped += 1;
                }
                Err(e) => {
                    tracing::warn!("Failed to ingest {:?}: {}", file_path, e);
                    errors += 1;
                }
            }
        }

        Ok(DiskIngestResult {
            total_files,
            ingested,
            skipped,
            errors,
            documents,
        })
    }

    /// Ingest a single file with source/run tracking.
    /// Returns `Ok(None)` if the file content is unchanged (skipped).
    fn ingest_file(
        &self,
        db: &Database,
        file_path: &Path,
        namespace_id: &str,
    ) -> IngestResult<Option<IngestedDocument>> {
        let file_path_str = file_path.to_string_lossy().to_string();
        let source_uri = format!("file://{}", file_path_str);
        let now = chrono::Utc::now().to_rfc3339();

        // Compute file hash for incremental check
        let file_hash = KbIndexer::file_hash(file_path)
            .map_err(|e| IngestError::Io(std::io::Error::other(e.to_string())))?;

        // Find or create ingest source
        let source = match db.find_ingest_source("file", &source_uri, namespace_id)? {
            Some(mut existing) => {
                // Check if content changed via hash
                if existing.content_hash.as_deref() == Some(&file_hash) {
                    // Content unchanged — record a no-op run and skip
                    let run_id = db.create_ingest_run(&existing.id)?;
                    db.complete_ingest_run(IngestRunCompletion {
                        run_id: &run_id,
                        status: "completed",
                        docs_added: 0,
                        docs_updated: 0,
                        docs_removed: 0,
                        chunks_added: 0,
                        error_message: None,
                    })?;
                    return Ok(None);
                }

                // Content changed — update source metadata
                existing.content_hash = Some(file_hash.clone());
                existing.last_ingested_at = Some(now.clone());
                existing.status = "active".to_string();
                existing.updated_at = now.clone();
                db.save_ingest_source(&existing)?;
                existing
            }
            None => {
                // Create new source
                let title = file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());

                let source = IngestSource {
                    id: uuid::Uuid::new_v4().to_string(),
                    source_type: "file".to_string(),
                    source_uri: source_uri.clone(),
                    namespace_id: namespace_id.to_string(),
                    title,
                    etag: None,
                    last_modified: None,
                    content_hash: Some(file_hash.clone()),
                    last_ingested_at: Some(now.clone()),
                    status: "active".to_string(),
                    error_message: None,
                    metadata_json: None,
                    created_at: now.clone(),
                    updated_at: now.clone(),
                };
                db.save_ingest_source(&source)?;
                source
            }
        };

        // Create ingest run
        let run_id = db.create_ingest_run(&source.id)?;

        // Parse and chunk the document
        let parsed = self
            .indexer
            .parse_document(file_path)
            .map_err(|e| IngestError::Parse(e.to_string()))?;

        let title = parsed
            .title
            .clone()
            .or_else(|| {
                file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| file_path_str.clone());

        let chunks = self.indexer.chunk_document(&parsed);
        let chunk_count = chunks.len();
        let word_count: usize = chunks.iter().map(|c| c.word_count).sum();

        if chunks.is_empty() {
            db.complete_ingest_run(IngestRunCompletion {
                run_id: &run_id,
                status: "completed",
                docs_added: 0,
                docs_updated: 0,
                docs_removed: 0,
                chunks_added: 0,
                error_message: None,
            })?;
            return Ok(None);
        }

        // Delete existing documents for this source (handles re-ingestion)
        db.delete_documents_for_source(&source.id)?;

        // Insert document with namespace_id, source_type, and source_id
        let doc_id = uuid::Uuid::new_v4().to_string();
        db.conn()
            .execute(
                "INSERT INTO kb_documents (id, file_path, file_hash, title, indexed_at, chunk_count,
                        namespace_id, source_type, source_id)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    doc_id,
                    file_path_str,
                    file_hash,
                    title,
                    now,
                    chunk_count as i32,
                    namespace_id,
                    "file",
                    source.id,
                ],
            )
            .map_err(IngestError::Sqlite)?;

        // Insert chunks with namespace_id
        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_id = uuid::Uuid::new_v4().to_string();
            db.conn()
                .execute(
                    "INSERT INTO kb_chunks (id, document_id, chunk_index, heading_path, content, word_count, namespace_id)
                     VALUES (?, ?, ?, ?, ?, ?, ?)",
                    rusqlite::params![
                        chunk_id,
                        doc_id,
                        i as i32,
                        chunk.heading_path,
                        chunk.content,
                        chunk.word_count as i32,
                        namespace_id,
                    ],
                )
                .map_err(IngestError::Sqlite)?;
        }

        // Determine if this was an add or update
        let (docs_added, docs_updated) = if source.created_at == source.updated_at {
            (1, 0)
        } else {
            (0, 1)
        };

        // Complete ingest run
        db.complete_ingest_run(IngestRunCompletion {
            run_id: &run_id,
            status: "completed",
            docs_added,
            docs_updated,
            docs_removed: 0,
            chunks_added: chunk_count as i32,
            error_message: None,
        })?;

        Ok(Some(IngestedDocument {
            id: doc_id,
            title,
            source_uri,
            chunk_count,
            word_count,
        }))
    }
}

impl Default for DiskIngester {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::MasterKey;
    use tempfile::TempDir;

    fn setup_test_db() -> (TempDir, Database) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let key = MasterKey::generate();
        let db = Database::open(&db_path, &key).unwrap();
        db.initialize().unwrap();
        (dir, db)
    }

    #[test]
    fn test_disk_ingest_basic() {
        let (_db_dir, db) = setup_test_db();
        let kb_dir = TempDir::new().unwrap();

        std::fs::write(
            kb_dir.path().join("test.md"),
            "# Test Document\n\nThis is test content for disk ingestion.",
        )
        .unwrap();

        db.ensure_namespace_exists("default").unwrap();

        let ingester = DiskIngester::new();
        let result = ingester
            .ingest_folder(&db, kb_dir.path(), "default")
            .unwrap();

        assert_eq!(result.total_files, 1);
        assert_eq!(result.ingested, 1);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.errors, 0);
        assert_eq!(result.documents.len(), 1);
    }

    #[test]
    fn test_disk_ingest_creates_source_and_run() {
        let (_db_dir, db) = setup_test_db();
        let kb_dir = TempDir::new().unwrap();

        std::fs::write(
            kb_dir.path().join("doc.md"),
            "# Source Tracking\n\nVerify ingest source and run are created.",
        )
        .unwrap();

        db.ensure_namespace_exists("default").unwrap();

        let ingester = DiskIngester::new();
        ingester
            .ingest_folder(&db, kb_dir.path(), "default")
            .unwrap();

        // Check ingest_sources entry
        let source_count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM ingest_sources WHERE source_type = 'file'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(source_count, 1);

        // Check ingest_runs entry
        let run_count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM ingest_runs WHERE status = 'completed'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(run_count, 1);
    }

    #[test]
    fn test_disk_ingest_incremental_skips_unchanged() {
        let (_db_dir, db) = setup_test_db();
        let kb_dir = TempDir::new().unwrap();

        std::fs::write(
            kb_dir.path().join("stable.md"),
            "# Stable Document\n\nContent that won't change.",
        )
        .unwrap();

        db.ensure_namespace_exists("default").unwrap();

        let ingester = DiskIngester::new();

        // First ingestion
        let result1 = ingester
            .ingest_folder(&db, kb_dir.path(), "default")
            .unwrap();
        assert_eq!(result1.ingested, 1);
        assert_eq!(result1.skipped, 0);

        // Second ingestion — same content
        let result2 = ingester
            .ingest_folder(&db, kb_dir.path(), "default")
            .unwrap();
        assert_eq!(result2.ingested, 0);
        assert_eq!(result2.skipped, 1);
    }

    #[test]
    fn test_disk_ingest_reindexes_changed_file() {
        let (_db_dir, db) = setup_test_db();
        let kb_dir = TempDir::new().unwrap();

        let file_path = kb_dir.path().join("mutable.md");
        std::fs::write(&file_path, "# Version 1\n\nOriginal content.").unwrap();

        db.ensure_namespace_exists("default").unwrap();

        let ingester = DiskIngester::new();

        // First ingestion
        let result1 = ingester
            .ingest_folder(&db, kb_dir.path(), "default")
            .unwrap();
        assert_eq!(result1.ingested, 1);

        // Modify file
        std::fs::write(&file_path, "# Version 2\n\nUpdated content with new info.").unwrap();

        // Second ingestion — should re-ingest the changed file
        let result2 = ingester
            .ingest_folder(&db, kb_dir.path(), "default")
            .unwrap();
        assert_eq!(result2.ingested, 1);
        assert_eq!(result2.skipped, 0);
    }

    #[test]
    fn test_disk_ingest_sets_namespace_and_source_on_docs() {
        let (_db_dir, db) = setup_test_db();
        let kb_dir = TempDir::new().unwrap();

        std::fs::write(
            kb_dir.path().join("ns-test.md"),
            "# Namespace Test\n\nVerify namespace and source fields.",
        )
        .unwrap();

        db.ensure_namespace_exists("test-ns").unwrap();

        let ingester = DiskIngester::new();
        ingester
            .ingest_folder(&db, kb_dir.path(), "test-ns")
            .unwrap();

        // Verify document has correct namespace_id and source_type
        let (ns, st): (String, String) = db
            .conn()
            .query_row(
                "SELECT namespace_id, source_type FROM kb_documents LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(ns, "test-ns");
        assert_eq!(st, "file");
    }
}
