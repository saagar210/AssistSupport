use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use memory_kernel_core::{
    Authority, ConstraintEffect, ConstraintPayload, ConstraintScope, ContextPackage, KernelError,
    LinkType, MemoryId, MemoryPayload, MemoryRecord, MemoryVersionId, RecordType, TruthStatus,
};
use rusqlite::{params, Connection, DatabaseName, OptionalExtension};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use ulid::Ulid;

const LATEST_SCHEMA_VERSION: i64 = 2;

const CREATE_SCHEMA_MIGRATIONS_SQL: &str = r"
CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);
";

const MIGRATION_001_SQL: &str = r"
CREATE TABLE IF NOT EXISTS memory_records (
  memory_id TEXT PRIMARY KEY,
  version INTEGER NOT NULL CHECK (version >= 1),
  record_type TEXT NOT NULL CHECK (record_type IN ('constraint','decision','preference','event','outcome')),
  created_at TEXT NOT NULL,
  effective_at TEXT NOT NULL,
  truth_status TEXT NOT NULL CHECK (truth_status IN ('asserted','observed','inferred','speculative','retracted')),
  authority TEXT NOT NULL CHECK (authority IN ('authoritative','derived','note')),
  confidence REAL,
  writer TEXT NOT NULL,
  justification TEXT NOT NULL,
  source_uri TEXT NOT NULL,
  source_hash TEXT,
  evidence_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS memory_links (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  from_memory_id TEXT NOT NULL,
  to_memory_id TEXT NOT NULL,
  link_type TEXT NOT NULL CHECK (link_type IN ('supersedes','contradicts')),
  writer TEXT NOT NULL,
  justification TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (from_memory_id) REFERENCES memory_records(memory_id),
  FOREIGN KEY (to_memory_id) REFERENCES memory_records(memory_id)
);

CREATE TABLE IF NOT EXISTS context_packages (
  context_package_id TEXT PRIMARY KEY,
  generated_at TEXT NOT NULL,
  package_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS constraint_payloads (
  memory_id TEXT PRIMARY KEY,
  actor TEXT NOT NULL,
  action TEXT NOT NULL,
  resource TEXT NOT NULL,
  effect TEXT NOT NULL CHECK (effect IN ('allow','deny')),
  note TEXT,
  FOREIGN KEY (memory_id) REFERENCES memory_records(memory_id)
);

CREATE TABLE IF NOT EXISTS decision_payloads (
  memory_id TEXT PRIMARY KEY,
  summary TEXT NOT NULL,
  FOREIGN KEY (memory_id) REFERENCES memory_records(memory_id)
);

CREATE TABLE IF NOT EXISTS preference_payloads (
  memory_id TEXT PRIMARY KEY,
  summary TEXT NOT NULL,
  FOREIGN KEY (memory_id) REFERENCES memory_records(memory_id)
);

CREATE TABLE IF NOT EXISTS event_payloads (
  memory_id TEXT PRIMARY KEY,
  summary TEXT NOT NULL,
  FOREIGN KEY (memory_id) REFERENCES memory_records(memory_id)
);

CREATE TABLE IF NOT EXISTS outcome_payloads (
  memory_id TEXT PRIMARY KEY,
  summary TEXT NOT NULL,
  FOREIGN KEY (memory_id) REFERENCES memory_records(memory_id)
);

CREATE INDEX IF NOT EXISTS idx_memory_records_type ON memory_records(record_type);
CREATE INDEX IF NOT EXISTS idx_memory_records_effective_at ON memory_records(effective_at);
CREATE INDEX IF NOT EXISTS idx_memory_links_from ON memory_links(from_memory_id);
CREATE INDEX IF NOT EXISTS idx_memory_links_to ON memory_links(to_memory_id);
";

const MIGRATION_002_CREATE_V2_TABLES_SQL: &str = r"
CREATE TABLE IF NOT EXISTS memory_records_v2 (
  memory_version_id TEXT PRIMARY KEY,
  memory_id TEXT NOT NULL,
  version INTEGER NOT NULL CHECK (version >= 1),
  record_type TEXT NOT NULL CHECK (record_type IN ('constraint','decision','preference','event','outcome')),
  created_at TEXT NOT NULL,
  effective_at TEXT NOT NULL,
  truth_status TEXT NOT NULL CHECK (truth_status IN ('asserted','observed','inferred','speculative','retracted')),
  authority TEXT NOT NULL CHECK (authority IN ('authoritative','derived','note')),
  confidence REAL,
  writer TEXT NOT NULL,
  justification TEXT NOT NULL,
  source_uri TEXT NOT NULL,
  source_hash TEXT,
  evidence_json TEXT NOT NULL,
  UNIQUE(memory_id, version)
);

CREATE TABLE IF NOT EXISTS memory_links_v2 (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  from_memory_version_id TEXT NOT NULL,
  to_memory_version_id TEXT NOT NULL,
  link_type TEXT NOT NULL CHECK (link_type IN ('supersedes','contradicts')),
  writer TEXT NOT NULL,
  justification TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (from_memory_version_id) REFERENCES memory_records_v2(memory_version_id),
  FOREIGN KEY (to_memory_version_id) REFERENCES memory_records_v2(memory_version_id)
);

CREATE TABLE IF NOT EXISTS constraint_payloads_v2 (
  memory_version_id TEXT PRIMARY KEY,
  actor TEXT NOT NULL,
  action TEXT NOT NULL,
  resource TEXT NOT NULL,
  effect TEXT NOT NULL CHECK (effect IN ('allow','deny')),
  note TEXT,
  FOREIGN KEY (memory_version_id) REFERENCES memory_records_v2(memory_version_id)
);

CREATE TABLE IF NOT EXISTS decision_payloads_v2 (
  memory_version_id TEXT PRIMARY KEY,
  summary TEXT NOT NULL,
  FOREIGN KEY (memory_version_id) REFERENCES memory_records_v2(memory_version_id)
);

CREATE TABLE IF NOT EXISTS preference_payloads_v2 (
  memory_version_id TEXT PRIMARY KEY,
  summary TEXT NOT NULL,
  FOREIGN KEY (memory_version_id) REFERENCES memory_records_v2(memory_version_id)
);

CREATE TABLE IF NOT EXISTS event_payloads_v2 (
  memory_version_id TEXT PRIMARY KEY,
  summary TEXT NOT NULL,
  FOREIGN KEY (memory_version_id) REFERENCES memory_records_v2(memory_version_id)
);

CREATE TABLE IF NOT EXISTS outcome_payloads_v2 (
  memory_version_id TEXT PRIMARY KEY,
  summary TEXT NOT NULL,
  FOREIGN KEY (memory_version_id) REFERENCES memory_records_v2(memory_version_id)
);
";

const MIGRATION_002_REPLACE_TABLES_SQL: &str = r"
DROP TABLE constraint_payloads;
DROP TABLE decision_payloads;
DROP TABLE preference_payloads;
DROP TABLE event_payloads;
DROP TABLE outcome_payloads;
DROP TABLE memory_links;
DROP TABLE memory_records;

ALTER TABLE memory_records_v2 RENAME TO memory_records;
ALTER TABLE memory_links_v2 RENAME TO memory_links;
ALTER TABLE constraint_payloads_v2 RENAME TO constraint_payloads;
ALTER TABLE decision_payloads_v2 RENAME TO decision_payloads;
ALTER TABLE preference_payloads_v2 RENAME TO preference_payloads;
ALTER TABLE event_payloads_v2 RENAME TO event_payloads;
ALTER TABLE outcome_payloads_v2 RENAME TO outcome_payloads;
";

const MIGRATION_002_FINAL_INDEXES_SQL: &str = r"
CREATE INDEX IF NOT EXISTS idx_memory_records_type ON memory_records(record_type);
CREATE INDEX IF NOT EXISTS idx_memory_records_memory_id ON memory_records(memory_id);
CREATE INDEX IF NOT EXISTS idx_memory_records_effective_at ON memory_records(effective_at);
CREATE INDEX IF NOT EXISTS idx_memory_links_from ON memory_links(from_memory_version_id);
CREATE INDEX IF NOT EXISTS idx_memory_links_to ON memory_links(to_memory_version_id);
";

pub struct SqliteStore {
    conn: Connection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaStatus {
    pub current_version: i64,
    pub target_version: i64,
    pub pending_versions: Vec<i64>,
    pub inferred_from_legacy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportFileDigest {
    pub path: String,
    pub sha256: String,
    pub records: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportManifest {
    pub schema_version: i64,
    pub exported_at: String,
    pub files: Vec<ExportFileDigest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImportSummary {
    pub imported_records: usize,
    pub skipped_existing_records: usize,
    pub imported_context_packages: usize,
    pub skipped_existing_context_packages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ForeignKeyViolation {
    pub table: String,
    pub rowid: i64,
    pub parent: String,
    pub fk_index: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrityReport {
    pub quick_check_ok: bool,
    pub quick_check_message: String,
    pub foreign_key_violations: Vec<ForeignKeyViolation>,
    pub schema_status: SchemaStatus,
}

impl SqliteStore {
    /// Open a SQLite-backed memory store and configure required runtime pragmas.
    ///
    /// # Errors
    /// Returns an error when the database cannot be opened or pragmas cannot be applied.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("failed to open sqlite database at {}", path.display()))?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        )
        .context("failed to configure sqlite pragmas")?;

        Ok(Self { conn })
    }

    /// Report current and target schema versions plus pending migrations.
    ///
    /// # Errors
    /// Returns an error when schema metadata cannot be read or initialized.
    pub fn schema_status(&self) -> Result<SchemaStatus> {
        self.conn
            .execute_batch(CREATE_SCHEMA_MIGRATIONS_SQL)
            .context("failed to apply schema_migrations table")?;
        let (current_version, inferred_from_legacy) = detect_effective_schema_version(&self.conn)?;
        let pending_versions = if current_version < LATEST_SCHEMA_VERSION {
            ((current_version + 1)..=LATEST_SCHEMA_VERSION).collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        Ok(SchemaStatus {
            current_version,
            target_version: LATEST_SCHEMA_VERSION,
            pending_versions,
            inferred_from_legacy,
        })
    }

    /// Apply all forward migrations up to the latest supported schema version.
    ///
    /// # Errors
    /// Returns an error when migration bootstrapping or any migration step fails.
    pub fn migrate(&mut self) -> Result<()> {
        self.conn
            .execute_batch(CREATE_SCHEMA_MIGRATIONS_SQL)
            .context("failed to apply schema_migrations table")?;

        let mut version = current_schema_version(&self.conn)?;

        if version == 0 {
            version = self.bootstrap_schema_version()?;
        }

        if version < 2 {
            self.apply_migration_2()?;
            version = current_schema_version(&self.conn)?;
        }

        if version != LATEST_SCHEMA_VERSION {
            return Err(anyhow!(
                "unsupported schema version {version}; expected {LATEST_SCHEMA_VERSION}"
            ));
        }

        Ok(())
    }

    fn bootstrap_schema_version(&self) -> Result<i64> {
        let has_memory_records = table_exists(&self.conn, "memory_records")?;

        if !has_memory_records {
            apply_migration_1(&self.conn)?;
            return Ok(1);
        }

        if table_has_column(&self.conn, "memory_records", "memory_version_id")? {
            // Database already in v2 shape (possibly created by an older scaffold),
            // but missing migration records.
            record_schema_version(&self.conn, 1)?;
            record_schema_version(&self.conn, 2)?;
            return Ok(2);
        }

        if table_has_column(&self.conn, "memory_records", "memory_id")? {
            // Legacy v1 table exists; mark version 1 and allow standard v2 upgrade.
            record_schema_version(&self.conn, 1)?;
            return Ok(1);
        }

        Err(anyhow!(
            "database schema is invalid: memory_records has neither memory_id nor memory_version_id"
        ))
    }

    fn apply_migration_2(&mut self) -> Result<()> {
        if table_has_column(&self.conn, "memory_records", "memory_version_id")? {
            record_schema_version(&self.conn, 2)?;
            return Ok(());
        }

        if !table_has_column(&self.conn, "memory_records", "memory_id")? {
            return Err(anyhow!(
                "cannot apply migration v2: legacy memory_records.memory_id column is missing"
            ));
        }

        let tx = self.conn.transaction().context("failed to start migration v2 transaction")?;

        tx.execute_batch(MIGRATION_002_CREATE_V2_TABLES_SQL)
            .context("failed to create v2 staging tables")?;

        let mut id_map: BTreeMap<String, String> = BTreeMap::new();

        {
            let mut stmt = tx.prepare(
                "SELECT
                    memory_id, version, record_type, created_at, effective_at,
                    truth_status, authority, confidence, writer, justification,
                    source_uri, source_hash, evidence_json
                 FROM memory_records
                 ORDER BY memory_id ASC",
            )?;

            let rows = stmt.query_map([], |row| {
                Ok(LegacyRecordRow {
                    memory_id: row.get(0)?,
                    version: row.get(1)?,
                    record_type: row.get(2)?,
                    created_at: row.get(3)?,
                    effective_at: row.get(4)?,
                    truth_status: row.get(5)?,
                    authority: row.get(6)?,
                    confidence: row.get(7)?,
                    writer: row.get(8)?,
                    justification: row.get(9)?,
                    source_uri: row.get(10)?,
                    source_hash: row.get(11)?,
                    evidence_json: row.get(12)?,
                })
            })?;

            for row in rows {
                let row = row?;
                let version_id = MemoryVersionId::new().to_string();

                tx.execute(
                    "INSERT INTO memory_records_v2(
                        memory_version_id, memory_id, version, record_type, created_at, effective_at,
                        truth_status, authority, confidence, writer, justification,
                        source_uri, source_hash, evidence_json
                    ) VALUES (
                        ?1, ?2, ?3, ?4, ?5, ?6,
                        ?7, ?8, ?9, ?10, ?11,
                        ?12, ?13, ?14
                    )",
                    params![
                        version_id,
                        row.memory_id,
                        row.version,
                        row.record_type,
                        row.created_at,
                        row.effective_at,
                        row.truth_status,
                        row.authority,
                        row.confidence,
                        row.writer,
                        row.justification,
                        row.source_uri,
                        row.source_hash,
                        row.evidence_json,
                    ],
                )
                .context("failed to copy memory_records row into v2")?;

                id_map.insert(row.memory_id, version_id);
            }
        }

        copy_constraint_payloads_to_v2(&tx, &id_map)?;
        copy_summary_payloads_to_v2(&tx, "decision_payloads", "decision_payloads_v2", &id_map)?;
        copy_summary_payloads_to_v2(&tx, "preference_payloads", "preference_payloads_v2", &id_map)?;
        copy_summary_payloads_to_v2(&tx, "event_payloads", "event_payloads_v2", &id_map)?;
        copy_summary_payloads_to_v2(&tx, "outcome_payloads", "outcome_payloads_v2", &id_map)?;
        copy_links_to_v2(&tx, &id_map)?;

        tx.execute_batch(MIGRATION_002_REPLACE_TABLES_SQL)
            .context("failed to replace legacy tables with v2 tables")?;
        tx.execute_batch(MIGRATION_002_FINAL_INDEXES_SQL).context("failed to create v2 indexes")?;

        let now = now_rfc3339()?;
        tx.execute(
            "INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
            params![2_i64, now],
        )
        .context("failed to record migration version 2")?;

        tx.commit().context("failed to commit migration v2")?;
        Ok(())
    }

    /// Persist one validated append-only memory record and its payload/link rows.
    ///
    /// # Errors
    /// Returns an error when validation fails or any write in the transaction fails.
    pub fn write_record(&mut self, record: &MemoryRecord) -> Result<()> {
        record.validate().map_err(|err| anyhow!("record validation failed: {err}"))?;

        let tx = self.conn.transaction().context("failed to start transaction")?;

        tx.execute(
            "INSERT INTO memory_records(
                memory_version_id, memory_id, version, record_type, created_at, effective_at,
                truth_status, authority, confidence, writer, justification,
                source_uri, source_hash, evidence_json
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                ?7, ?8, ?9, ?10, ?11,
                ?12, ?13, ?14
            )",
            params![
                record.memory_version_id.to_string(),
                record.memory_id.to_string(),
                i64::from(record.version),
                record.payload.record_type().as_str(),
                rfc3339(record.created_at)?,
                rfc3339(record.effective_at)?,
                record.truth_status.as_str(),
                record.authority.as_str(),
                record.confidence,
                record.writer,
                record.justification,
                record.provenance.source_uri,
                record.provenance.source_hash,
                serde_json::to_string(&record.provenance.evidence)
                    .context("failed to serialize evidence")?,
            ],
        )
        .context("failed to insert memory record")?;

        Self::insert_payload(&tx, record)?;
        Self::insert_links(&tx, record, LinkType::Supersedes, &record.supersedes)?;
        Self::insert_links(&tx, record, LinkType::Contradicts, &record.contradicts)?;

        tx.commit().context("failed to commit write transaction")?;
        Ok(())
    }

    /// Load all persisted memory records with payloads and lineage links.
    ///
    /// # Errors
    /// Returns an error when rows cannot be read or decoded from `SQLite`.
    pub fn list_records(&self) -> Result<Vec<MemoryRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                memory_version_id, memory_id, version, record_type, created_at, effective_at,
                truth_status, authority, confidence, writer, justification,
                source_uri, source_hash, evidence_json
             FROM memory_records
             ORDER BY created_at DESC, memory_id ASC, memory_version_id ASC",
        )?;

        let mut rows = stmt.query([])?;
        let mut records = Vec::new();

        while let Some(row) = rows.next()? {
            let memory_version_id_raw: String = row.get(0)?;
            let memory_id_raw: String = row.get(1)?;
            let record_type_raw: String = row.get(3)?;
            let memory_version_id = parse_memory_version_id(&memory_version_id_raw)?;
            let memory_id = parse_memory_id(&memory_id_raw)?;
            let record_type = RecordType::parse(&record_type_raw)
                .ok_or_else(|| anyhow!("unknown record_type: {record_type_raw}"))?;

            let payload = self.load_payload(memory_version_id, record_type)?;
            let supersedes = self.load_links(memory_version_id, LinkType::Supersedes)?;
            let contradicts = self.load_links(memory_version_id, LinkType::Contradicts)?;

            let truth_status_raw: String = row.get(6)?;
            let authority_raw: String = row.get(7)?;
            let evidence_json: String = row.get(13)?;

            records.push(MemoryRecord {
                memory_version_id,
                memory_id,
                version: row.get::<_, u32>(2)?,
                payload,
                created_at: parse_rfc3339(&row.get::<_, String>(4)?)?,
                effective_at: parse_rfc3339(&row.get::<_, String>(5)?)?,
                truth_status: TruthStatus::parse(&truth_status_raw)
                    .ok_or_else(|| anyhow!("unknown truth_status: {truth_status_raw}"))?,
                authority: Authority::parse(&authority_raw)
                    .ok_or_else(|| anyhow!("unknown authority: {authority_raw}"))?,
                confidence: row.get(8)?,
                writer: row.get(9)?,
                justification: row.get(10)?,
                provenance: memory_kernel_core::Provenance {
                    source_uri: row.get(11)?,
                    source_hash: row.get(12)?,
                    evidence: serde_json::from_str(&evidence_json)
                        .context("failed to deserialize evidence")?,
                },
                supersedes,
                contradicts,
            });
        }

        Ok(records)
    }

    /// Persist one explicit lineage link between two memory version IDs.
    ///
    /// # Errors
    /// Returns an error when accountability fields are empty or persistence fails.
    pub fn add_link(
        &mut self,
        from: MemoryVersionId,
        to: MemoryVersionId,
        link_type: LinkType,
        writer: &str,
        justification: &str,
    ) -> Result<()> {
        if writer.trim().is_empty() {
            return Err(anyhow!("writer MUST be provided for every link write"));
        }
        if justification.trim().is_empty() {
            return Err(anyhow!("justification MUST be provided for every link write"));
        }

        let tx = self.conn.transaction().context("failed to start transaction")?;
        tx.execute(
            "INSERT INTO memory_links(
                from_memory_version_id, to_memory_version_id, link_type, writer, justification, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                from.to_string(),
                to.to_string(),
                link_type.as_str(),
                writer,
                justification,
                now_rfc3339()?
            ],
        )
        .context("failed to insert memory link")?;
        tx.commit().context("failed to commit link transaction")?;
        Ok(())
    }

    /// Persist one Context Package artifact.
    ///
    /// # Errors
    /// Returns an error when serialization or transaction writes fail.
    pub fn save_context_package(&mut self, package: &ContextPackage) -> Result<()> {
        let tx = self.conn.transaction().context("failed to start transaction")?;
        tx.execute(
            "INSERT INTO context_packages(context_package_id, generated_at, package_json)
             VALUES (?1, ?2, ?3)",
            params![
                package.context_package_id,
                rfc3339(package.generated_at)?,
                serde_json::to_string(package).context("failed to serialize context package")?,
            ],
        )
        .context("failed to persist context package")?;
        tx.commit().context("failed to commit context package transaction")?;
        Ok(())
    }

    /// Retrieve a Context Package by its stable identifier.
    ///
    /// # Errors
    /// Returns an error when lookup or JSON deserialization fails.
    pub fn get_context_package(&self, context_package_id: &str) -> Result<Option<ContextPackage>> {
        let mut stmt = self
            .conn
            .prepare("SELECT package_json FROM context_packages WHERE context_package_id = ?1")?;
        let value = stmt
            .query_row(params![context_package_id], |row| row.get::<_, String>(0))
            .optional()?;

        match value {
            Some(json) => {
                let package = serde_json::from_str(&json)
                    .context("failed to deserialize stored context package")?;
                Ok(Some(package))
            }
            None => Ok(None),
        }
    }

    /// Export records and context packages as deterministic NDJSON plus manifest.
    ///
    /// # Errors
    /// Returns an error when export files cannot be created, written, or serialized.
    pub fn export_snapshot(&self, out_dir: &Path) -> Result<ExportManifest> {
        fs::create_dir_all(out_dir)
            .with_context(|| format!("failed to create export directory {}", out_dir.display()))?;

        let records = self.list_records()?;
        let context_packages = self.list_context_packages()?;

        let records_path = out_dir.join("memory_records.ndjson");
        let record_digest = write_ndjson_file(&records_path, &records)?;

        let packages_path = out_dir.join("context_packages.ndjson");
        let package_digest = write_ndjson_file(&packages_path, &context_packages)?;

        let manifest = ExportManifest {
            schema_version: LATEST_SCHEMA_VERSION,
            exported_at: now_rfc3339()?,
            files: vec![
                ExportFileDigest {
                    path: "memory_records.ndjson".to_string(),
                    sha256: record_digest.0,
                    records: record_digest.1,
                },
                ExportFileDigest {
                    path: "context_packages.ndjson".to_string(),
                    sha256: package_digest.0,
                    records: package_digest.1,
                },
            ],
        };

        let manifest_path = out_dir.join("manifest.json");
        let manifest_json =
            serde_json::to_vec_pretty(&manifest).context("failed to serialize export manifest")?;
        fs::write(&manifest_path, manifest_json).with_context(|| {
            format!("failed to write export manifest {}", manifest_path.display())
        })?;

        Ok(manifest)
    }

    /// Import an exported snapshot directory into this database.
    ///
    /// # Errors
    /// Returns an error when migration, parsing, duplicate handling, or writes fail.
    pub fn import_snapshot(&mut self, in_dir: &Path, skip_existing: bool) -> Result<ImportSummary> {
        self.migrate()?;
        let manifest_path = in_dir.join("manifest.json");
        let manifest = read_export_manifest(&manifest_path)?;
        validate_import_manifest(in_dir, &manifest)?;

        let records_path = in_dir.join("memory_records.ndjson");
        let package_path = in_dir.join("context_packages.ndjson");

        let mut summary = ImportSummary {
            imported_records: 0,
            skipped_existing_records: 0,
            imported_context_packages: 0,
            skipped_existing_context_packages: 0,
        };

        for record in read_ndjson_file::<MemoryRecord>(&records_path)? {
            if self.record_exists(record.memory_version_id)? {
                if skip_existing {
                    summary.skipped_existing_records += 1;
                    continue;
                }

                return Err(anyhow!(
                    "record already exists for memory_version_id {}",
                    record.memory_version_id
                ));
            }
            self.write_record(&record)?;
            summary.imported_records += 1;
        }

        for package in read_ndjson_file::<ContextPackage>(&package_path)? {
            if self.context_package_exists(&package.context_package_id)? {
                if skip_existing {
                    summary.skipped_existing_context_packages += 1;
                    continue;
                }

                return Err(anyhow!(
                    "context package already exists: {}",
                    package.context_package_id
                ));
            }
            self.save_context_package(&package)?;
            summary.imported_context_packages += 1;
        }

        Ok(summary)
    }

    /// Create a `SQLite` backup file of the current main database.
    ///
    /// # Errors
    /// Returns an error when backup directories cannot be created or backup fails.
    pub fn backup_database(&self, out_file: &Path) -> Result<()> {
        if let Some(parent) = out_file.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create parent directory for backup file {}", out_file.display())
            })?;
        }

        self.conn
            .backup(DatabaseName::Main, out_file, None)
            .with_context(|| format!("failed to create sqlite backup at {}", out_file.display()))
    }

    /// Restore this database from a `SQLite` backup file, then migrate to latest.
    ///
    /// # Errors
    /// Returns an error when the backup file is missing, restore fails, or migrations fail.
    pub fn restore_database(&mut self, in_file: &Path) -> Result<()> {
        if !in_file.exists() {
            return Err(anyhow!("backup file does not exist: {}", in_file.display()));
        }

        self.conn
            .restore(DatabaseName::Main, in_file, None::<fn(rusqlite::backup::Progress)>)
            .with_context(|| {
                format!("failed to restore sqlite backup from {}", in_file.display())
            })?;

        self.migrate()?;
        Ok(())
    }

    /// Run quick-check, foreign-key-check, and schema status health probes.
    ///
    /// # Errors
    /// Returns an error when any integrity probe query fails.
    pub fn integrity_check(&self) -> Result<IntegrityReport> {
        let quick_check_message: String = self
            .conn
            .query_row("PRAGMA quick_check", [], |row| row.get::<_, String>(0))
            .context("failed to run PRAGMA quick_check")?;

        let mut stmt = self
            .conn
            .prepare("PRAGMA foreign_key_check")
            .context("failed to prepare PRAGMA foreign_key_check")?;
        let rows = stmt.query_map([], |row| {
            Ok(ForeignKeyViolation {
                table: row.get(0)?,
                rowid: row.get(1)?,
                parent: row.get(2)?,
                fk_index: row.get(3)?,
            })
        })?;

        let mut foreign_key_violations = Vec::new();
        for row in rows {
            foreign_key_violations.push(row?);
        }

        let schema_status = self.schema_status()?;
        Ok(IntegrityReport {
            quick_check_ok: quick_check_message == "ok",
            quick_check_message,
            foreign_key_violations,
            schema_status,
        })
    }

    fn insert_payload(tx: &rusqlite::Transaction<'_>, record: &MemoryRecord) -> Result<()> {
        match &record.payload {
            MemoryPayload::Constraint(payload) => {
                tx.execute(
                    "INSERT INTO constraint_payloads(memory_version_id, actor, action, resource, effect, note)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        record.memory_version_id.to_string(),
                        payload.scope.actor,
                        payload.scope.action,
                        payload.scope.resource,
                        payload.effect.as_str(),
                        payload.note,
                    ],
                )
                .context("failed to insert constraint payload")?;
            }
            MemoryPayload::Decision(payload) => {
                tx.execute(
                    "INSERT INTO decision_payloads(memory_version_id, summary) VALUES (?1, ?2)",
                    params![record.memory_version_id.to_string(), payload.summary],
                )
                .context("failed to insert decision payload")?;
            }
            MemoryPayload::Preference(payload) => {
                tx.execute(
                    "INSERT INTO preference_payloads(memory_version_id, summary) VALUES (?1, ?2)",
                    params![record.memory_version_id.to_string(), payload.summary],
                )
                .context("failed to insert preference payload")?;
            }
            MemoryPayload::Event(payload) => {
                tx.execute(
                    "INSERT INTO event_payloads(memory_version_id, summary) VALUES (?1, ?2)",
                    params![record.memory_version_id.to_string(), payload.summary],
                )
                .context("failed to insert event payload")?;
            }
            MemoryPayload::Outcome(payload) => {
                tx.execute(
                    "INSERT INTO outcome_payloads(memory_version_id, summary) VALUES (?1, ?2)",
                    params![record.memory_version_id.to_string(), payload.summary],
                )
                .context("failed to insert outcome payload")?;
            }
        }

        Ok(())
    }

    fn insert_links(
        tx: &rusqlite::Transaction<'_>,
        record: &MemoryRecord,
        link_type: LinkType,
        targets: &[MemoryVersionId],
    ) -> Result<()> {
        let now = now_rfc3339()?;
        for target in targets {
            tx.execute(
                "INSERT INTO memory_links(
                    from_memory_version_id, to_memory_version_id, link_type, writer, justification, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    record.memory_version_id.to_string(),
                    target.to_string(),
                    link_type.as_str(),
                    record.writer,
                    record.justification,
                    now
                ],
            )
            .context("failed to insert memory link")?;
        }

        Ok(())
    }

    fn load_payload(
        &self,
        memory_version_id: MemoryVersionId,
        record_type: RecordType,
    ) -> Result<MemoryPayload> {
        match record_type {
            RecordType::Constraint => {
                let mut stmt = self.conn.prepare(
                    "SELECT actor, action, resource, effect, note
                     FROM constraint_payloads
                     WHERE memory_version_id = ?1",
                )?;
                let payload = stmt
                    .query_row(params![memory_version_id.to_string()], |row| {
                        let effect_raw: String = row.get(3)?;
                        let effect = ConstraintEffect::parse(&effect_raw).ok_or_else(|| {
                            rusqlite::Error::FromSqlConversionFailure(
                                3,
                                rusqlite::types::Type::Text,
                                Box::new(KernelError::Validation(format!(
                                    "invalid constraint effect: {effect_raw}"
                                ))),
                            )
                        })?;

                        Ok(ConstraintPayload {
                            scope: ConstraintScope {
                                actor: row.get(0)?,
                                action: row.get(1)?,
                                resource: row.get(2)?,
                            },
                            effect,
                            note: row.get(4)?,
                        })
                    })
                    .optional()?
                    .ok_or_else(|| anyhow!("missing constraint payload for {memory_version_id}"))?;

                Ok(MemoryPayload::Constraint(payload))
            }
            RecordType::Decision => {
                let summary = self.load_summary("decision_payloads", memory_version_id)?;
                Ok(MemoryPayload::Decision(memory_kernel_core::DecisionPayload { summary }))
            }
            RecordType::Preference => {
                let summary = self.load_summary("preference_payloads", memory_version_id)?;
                Ok(MemoryPayload::Preference(memory_kernel_core::PreferencePayload { summary }))
            }
            RecordType::Event => {
                let summary = self.load_summary("event_payloads", memory_version_id)?;
                Ok(MemoryPayload::Event(memory_kernel_core::EventPayload { summary }))
            }
            RecordType::Outcome => {
                let summary = self.load_summary("outcome_payloads", memory_version_id)?;
                Ok(MemoryPayload::Outcome(memory_kernel_core::OutcomePayload { summary }))
            }
        }
    }

    fn load_summary(&self, table_name: &str, memory_version_id: MemoryVersionId) -> Result<String> {
        let query = format!("SELECT summary FROM {table_name} WHERE memory_version_id = ?1");
        let mut stmt = self.conn.prepare(&query)?;
        let value = stmt
            .query_row(params![memory_version_id.to_string()], |row| row.get::<_, String>(0))
            .optional()?
            .ok_or_else(|| anyhow!("missing payload in {table_name} for {memory_version_id}"))?;
        Ok(value)
    }

    fn load_links(
        &self,
        memory_version_id: MemoryVersionId,
        link_type: LinkType,
    ) -> Result<Vec<MemoryVersionId>> {
        let mut stmt = self.conn.prepare(
            "SELECT to_memory_version_id FROM memory_links
             WHERE from_memory_version_id = ?1 AND link_type = ?2
             ORDER BY id ASC",
        )?;

        let rows =
            stmt.query_map(params![memory_version_id.to_string(), link_type.as_str()], |row| {
                let raw: String = row.get(0)?;
                let parsed = Ulid::from_str(&raw).map_err(|_| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("invalid ULID in link row: {raw}"),
                        )),
                    )
                })?;
                Ok(MemoryVersionId(parsed))
            })?;

        let mut ids = Vec::new();
        for row in rows {
            ids.push(row?);
        }

        Ok(ids)
    }

    fn list_context_packages(&self) -> Result<Vec<ContextPackage>> {
        let mut stmt = self.conn.prepare(
            "SELECT package_json FROM context_packages ORDER BY generated_at DESC, context_package_id ASC",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut packages = Vec::new();
        for row in rows {
            let raw = row?;
            let parsed = serde_json::from_str::<ContextPackage>(&raw)
                .context("failed to deserialize context package row")?;
            packages.push(parsed);
        }
        Ok(packages)
    }

    fn record_exists(&self, memory_version_id: MemoryVersionId) -> Result<bool> {
        let exists = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM memory_records WHERE memory_version_id = ?1)",
            params![memory_version_id.to_string()],
            |row| row.get::<_, i64>(0),
        )?;
        Ok(exists == 1)
    }

    fn context_package_exists(&self, context_package_id: &str) -> Result<bool> {
        let exists = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM context_packages WHERE context_package_id = ?1)",
            params![context_package_id],
            |row| row.get::<_, i64>(0),
        )?;
        Ok(exists == 1)
    }
}

#[derive(Debug)]
struct LegacyRecordRow {
    memory_id: String,
    version: i64,
    record_type: String,
    created_at: String,
    effective_at: String,
    truth_status: String,
    authority: String,
    confidence: Option<f64>,
    writer: String,
    justification: String,
    source_uri: String,
    source_hash: Option<String>,
    evidence_json: String,
}

fn apply_migration_1(conn: &Connection) -> Result<()> {
    conn.execute_batch(MIGRATION_001_SQL).context("failed to apply migration v1")?;
    record_schema_version(conn, 1)?;
    Ok(())
}

fn copy_constraint_payloads_to_v2(
    tx: &rusqlite::Transaction<'_>,
    id_map: &BTreeMap<String, String>,
) -> Result<()> {
    let mut stmt = tx.prepare(
        "SELECT memory_id, actor, action, resource, effect, note
         FROM constraint_payloads",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, Option<String>>(5)?,
        ))
    })?;

    for row in rows {
        let (memory_id, actor, action, resource, effect, note) = row?;
        let memory_version_id = mapped_version_id(id_map, &memory_id)?;
        tx.execute(
            "INSERT INTO constraint_payloads_v2(memory_version_id, actor, action, resource, effect, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![memory_version_id, actor, action, resource, effect, note],
        )
        .context("failed to copy constraint payload into v2")?;
    }

    Ok(())
}

fn copy_summary_payloads_to_v2(
    tx: &rusqlite::Transaction<'_>,
    source_table: &str,
    target_table: &str,
    id_map: &BTreeMap<String, String>,
) -> Result<()> {
    let query = format!("SELECT memory_id, summary FROM {source_table}");
    let mut stmt = tx.prepare(&query)?;
    let rows =
        stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?;

    for row in rows {
        let (memory_id, summary) = row?;
        let memory_version_id = mapped_version_id(id_map, &memory_id)?;
        let insert =
            format!("INSERT INTO {target_table}(memory_version_id, summary) VALUES (?1, ?2)");
        tx.execute(&insert, params![memory_version_id, summary])
            .with_context(|| format!("failed to copy payload row into {target_table}"))?;
    }

    Ok(())
}

fn copy_links_to_v2(
    tx: &rusqlite::Transaction<'_>,
    id_map: &BTreeMap<String, String>,
) -> Result<()> {
    let mut stmt = tx.prepare(
        "SELECT from_memory_id, to_memory_id, link_type, writer, justification, created_at
         FROM memory_links
         ORDER BY id ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
        ))
    })?;

    for row in rows {
        let (from_memory_id, to_memory_id, link_type, writer, justification, created_at) = row?;
        let from_memory_version_id = mapped_version_id(id_map, &from_memory_id)?;
        let to_memory_version_id = mapped_version_id(id_map, &to_memory_id)?;

        tx.execute(
            "INSERT INTO memory_links_v2(
                from_memory_version_id, to_memory_version_id, link_type, writer, justification, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                from_memory_version_id,
                to_memory_version_id,
                link_type,
                writer,
                justification,
                created_at,
            ],
        )
        .context("failed to copy memory link into v2")?;
    }

    Ok(())
}

fn mapped_version_id(id_map: &BTreeMap<String, String>, memory_id: &str) -> Result<String> {
    id_map.get(memory_id).cloned().ok_or_else(|| {
        anyhow!("migration mapping missing memory_version_id for legacy memory_id {memory_id}")
    })
}

fn table_exists(conn: &Connection, table_name: &str) -> Result<bool> {
    let exists = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
            params![table_name],
            |row| row.get::<_, i64>(0),
        )
        .with_context(|| format!("failed to check if table exists: {table_name}"))?;
    Ok(exists == 1)
}

fn table_has_column(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    if !table_exists(conn, table)? {
        return Ok(false);
    }

    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .with_context(|| format!("failed to inspect table_info for {table}"))?;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(true);
        }
    }

    Ok(false)
}

fn current_schema_version(conn: &Connection) -> Result<i64> {
    let version = conn
        .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_migrations", [], |row| {
            row.get::<_, i64>(0)
        })
        .context("failed to read current schema version")?;
    Ok(version)
}

fn detect_effective_schema_version(conn: &Connection) -> Result<(i64, bool)> {
    let recorded = current_schema_version(conn)?;
    if recorded > 0 {
        return Ok((recorded, false));
    }

    if !table_exists(conn, "memory_records")? {
        return Ok((0, false));
    }

    if table_has_column(conn, "memory_records", "memory_version_id")? {
        return Ok((2, true));
    }

    if table_has_column(conn, "memory_records", "memory_id")? {
        return Ok((1, true));
    }

    Err(anyhow!(
        "database schema is invalid: memory_records has neither memory_id nor memory_version_id"
    ))
}

fn record_schema_version(conn: &Connection, version: i64) -> Result<()> {
    let now = now_rfc3339()?;
    conn.execute(
        "INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
        params![version, now],
    )
    .with_context(|| format!("failed to record migration version {version}"))?;
    Ok(())
}

fn now_rfc3339() -> Result<String> {
    rfc3339(OffsetDateTime::now_utc())
}

fn rfc3339(value: OffsetDateTime) -> Result<String> {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .context("failed to format RFC3339 timestamp")
}

fn parse_rfc3339(value: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
        .with_context(|| format!("invalid RFC3339 timestamp: {value}"))
}

fn parse_memory_id(raw: &str) -> Result<MemoryId> {
    let parsed = Ulid::from_string(raw).with_context(|| format!("invalid ULID: {raw}"))?;
    Ok(MemoryId(parsed))
}

fn parse_memory_version_id(raw: &str) -> Result<MemoryVersionId> {
    let parsed = Ulid::from_string(raw).with_context(|| format!("invalid ULID: {raw}"))?;
    Ok(MemoryVersionId(parsed))
}

fn write_ndjson_file<T: Serialize>(path: &Path, values: &[T]) -> Result<(String, usize)> {
    let file = File::create(path)
        .with_context(|| format!("failed to create export file {}", path.display()))?;
    let mut writer = BufWriter::new(file);
    let mut hasher = Sha256::new();

    for value in values {
        let line = serde_json::to_string(value).context("failed to serialize NDJSON row")?;
        writer
            .write_all(line.as_bytes())
            .with_context(|| format!("failed to write export file {}", path.display()))?;
        writer
            .write_all(b"\n")
            .with_context(|| format!("failed to write export file {}", path.display()))?;
        hasher.update(line.as_bytes());
        hasher.update(b"\n");
    }

    writer.flush().with_context(|| format!("failed to flush export file {}", path.display()))?;

    Ok((format!("{:x}", hasher.finalize()), values.len()))
}

fn read_ndjson_file<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    let file = File::open(path)
        .with_context(|| format!("failed to open NDJSON file {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut values = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!("failed to read line {} from {}", index + 1, path.display())
        })?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value = serde_json::from_str(trimmed).with_context(|| {
            format!("failed to parse NDJSON row {} from {}", index + 1, path.display())
        })?;
        values.push(value);
    }

    Ok(values)
}

fn read_export_manifest(path: &Path) -> Result<ExportManifest> {
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read manifest file {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse manifest JSON {}", path.display()))
}

fn ndjson_digest_and_records(path: &Path) -> Result<(String, usize)> {
    let file = File::open(path)
        .with_context(|| format!("failed to open NDJSON file {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut records = 0_usize;

    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!("failed to read line {} from {}", index + 1, path.display())
        })?;
        hasher.update(line.as_bytes());
        hasher.update(b"\n");
        if !line.trim().is_empty() {
            records += 1;
        }
    }

    Ok((format!("{:x}", hasher.finalize()), records))
}

fn validate_import_manifest(in_dir: &Path, manifest: &ExportManifest) -> Result<()> {
    if manifest.schema_version <= 0 || manifest.schema_version > LATEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported export schema version {}; supported range is 1..={}",
            manifest.schema_version,
            LATEST_SCHEMA_VERSION
        ));
    }

    let mut by_path: BTreeMap<&str, &ExportFileDigest> = BTreeMap::new();
    for file in &manifest.files {
        if by_path.insert(file.path.as_str(), file).is_some() {
            return Err(anyhow!("manifest contains duplicate file entry: {}", file.path));
        }
    }

    for required in ["memory_records.ndjson", "context_packages.ndjson"] {
        let Some(expected) = by_path.get(required) else {
            return Err(anyhow!("manifest is missing required file entry: {required}"));
        };
        let file_path = in_dir.join(required);
        if !file_path.exists() {
            return Err(anyhow!("manifest references missing file {}", file_path.display()));
        }

        let (actual_sha256, actual_records) = ndjson_digest_and_records(&file_path)?;
        if actual_sha256 != expected.sha256 {
            return Err(anyhow!(
                "manifest digest mismatch for {required}: expected {}, got {}",
                expected.sha256,
                actual_sha256
            ));
        }
        if actual_records != expected.records {
            return Err(anyhow!(
                "manifest record count mismatch for {required}: expected {}, got {}",
                expected.records,
                actual_records
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::thread;

    use super::*;
    use memory_kernel_core::{
        build_context_package, ConstraintEffect, ConstraintPayload, ConstraintScope, MemoryPayload,
        Provenance, QueryRequest,
    };

    fn insert_legacy_constraint_record(
        conn: &Connection,
        memory_id: MemoryId,
        created_at: &str,
        confidence: f64,
        justification: &str,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO memory_records(
                memory_id, version, record_type, created_at, effective_at,
                truth_status, authority, confidence, writer, justification,
                source_uri, source_hash, evidence_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                memory_id.to_string(),
                1_i64,
                "constraint",
                created_at,
                created_at,
                "asserted",
                "authoritative",
                confidence,
                "tester",
                justification,
                "file:///policy.md",
                "sha256:abc123",
                "[]",
            ],
        )?;

        Ok(())
    }

    fn insert_legacy_constraint_payload(conn: &Connection, memory_id: MemoryId) -> Result<()> {
        conn.execute(
            "INSERT INTO constraint_payloads(memory_id, actor, action, resource, effect, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                memory_id.to_string(),
                "user",
                "use",
                "usb_drive",
                "deny",
                Option::<String>::None,
            ],
        )?;
        Ok(())
    }

    fn insert_legacy_supersedes_link(
        conn: &Connection,
        from_memory_id: MemoryId,
        to_memory_id: MemoryId,
        created_at: &str,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO memory_links(from_memory_id, to_memory_id, link_type, writer, justification, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                from_memory_id.to_string(),
                to_memory_id.to_string(),
                "supersedes",
                "tester",
                "legacy link",
                created_at,
            ],
        )?;
        Ok(())
    }

    fn mk_store_constraint_record(
        memory_id: MemoryId,
        version: u32,
        truth_status: TruthStatus,
        confidence: Option<f32>,
        effect: ConstraintEffect,
    ) -> MemoryRecord {
        MemoryRecord {
            memory_version_id: MemoryVersionId::new(),
            memory_id,
            version,
            created_at: OffsetDateTime::now_utc(),
            effective_at: OffsetDateTime::now_utc(),
            truth_status,
            authority: Authority::Authoritative,
            confidence,
            writer: "tester".to_string(),
            justification: "fixture".to_string(),
            provenance: Provenance {
                source_uri: "file:///policy.md".to_string(),
                source_hash: Some("sha256:abc123".to_string()),
                evidence: vec![],
            },
            supersedes: vec![],
            contradicts: vec![],
            payload: MemoryPayload::Constraint(ConstraintPayload {
                scope: ConstraintScope {
                    actor: "user".to_string(),
                    action: "use".to_string(),
                    resource: "usb_drive".to_string(),
                },
                effect,
                note: None,
            }),
        }
    }

    // Test IDs: TDB-002
    #[test]
    fn sqlite_constraints_enforce_checks_and_foreign_keys() -> Result<()> {
        let mut store = SqliteStore::open(Path::new(":memory:"))?;
        store.migrate()?;

        let check_result = store.conn.execute(
            "INSERT INTO memory_records(
                memory_version_id, memory_id, version, record_type, created_at, effective_at,
                truth_status, authority, confidence, writer, justification,
                source_uri, source_hash, evidence_json
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                ?7, ?8, ?9, ?10, ?11,
                ?12, ?13, ?14
            )",
            params![
                MemoryVersionId::new().to_string(),
                MemoryId::new().to_string(),
                1_i64,
                "not_a_valid_record_type",
                "2026-01-01T00:00:00Z",
                "2026-01-01T00:00:00Z",
                "asserted",
                "authoritative",
                0.5_f64,
                "tester",
                "check invalid enum",
                "file:///policy.md",
                "sha256:abc123",
                "[]",
            ],
        );
        assert!(check_result.is_err());

        let fk_result = store.conn.execute(
            "INSERT INTO memory_links(
                from_memory_version_id, to_memory_version_id, link_type, writer, justification, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                MemoryVersionId::new().to_string(),
                MemoryVersionId::new().to_string(),
                "supersedes",
                "tester",
                "foreign key invalid",
                "2026-01-01T00:00:00Z",
            ],
        );
        assert!(fk_result.is_err());

        Ok(())
    }

    // Test IDs: TID-001
    #[test]
    fn inserted_records_receive_distinct_memory_version_ids() -> Result<()> {
        let mut store = SqliteStore::open(Path::new(":memory:"))?;
        store.migrate()?;

        let a = mk_store_constraint_record(
            MemoryId::new(),
            1,
            TruthStatus::Asserted,
            Some(0.8),
            ConstraintEffect::Deny,
        );
        let b = mk_store_constraint_record(
            MemoryId::new(),
            1,
            TruthStatus::Asserted,
            Some(0.9),
            ConstraintEffect::Allow,
        );

        store.write_record(&a)?;
        store.write_record(&b)?;

        let records = store.list_records()?;
        let ids = records.iter().map(|record| record.memory_version_id).collect::<BTreeSet<_>>();

        assert_eq!(records.len(), 2);
        assert_eq!(ids.len(), 2);
        Ok(())
    }

    // Test IDs: TID-002
    #[test]
    fn duplicate_memory_id_version_is_rejected() -> Result<()> {
        let mut store = SqliteStore::open(Path::new(":memory:"))?;
        store.migrate()?;

        let memory_id = MemoryId::new();
        let first = mk_store_constraint_record(
            memory_id,
            1,
            TruthStatus::Asserted,
            Some(0.9),
            ConstraintEffect::Deny,
        );
        let second = mk_store_constraint_record(
            memory_id,
            1,
            TruthStatus::Observed,
            Some(0.95),
            ConstraintEffect::Allow,
        );

        store.write_record(&first)?;
        let second_err = store.write_record(&second);
        assert!(second_err.is_err());

        Ok(())
    }

    // Test IDs: TDB-003
    #[test]
    fn write_and_read_constraint_round_trip() -> Result<()> {
        let mut store = SqliteStore::open(Path::new(":memory:"))?;
        store.migrate()?;

        let record = MemoryRecord {
            memory_version_id: MemoryVersionId::new(),
            memory_id: MemoryId::new(),
            version: 1,
            created_at: OffsetDateTime::now_utc(),
            effective_at: OffsetDateTime::now_utc(),
            truth_status: TruthStatus::Asserted,
            authority: Authority::Authoritative,
            confidence: Some(0.95),
            writer: "tester".to_string(),
            justification: "seed policy".to_string(),
            provenance: Provenance {
                source_uri: "file:///policy.md".to_string(),
                source_hash: Some("sha256:abc123".to_string()),
                evidence: vec![],
            },
            supersedes: vec![],
            contradicts: vec![],
            payload: MemoryPayload::Constraint(ConstraintPayload {
                scope: ConstraintScope {
                    actor: "user".to_string(),
                    action: "use".to_string(),
                    resource: "usb_drive".to_string(),
                },
                effect: ConstraintEffect::Deny,
                note: None,
            }),
        };

        store.write_record(&record)?;
        let records = store.list_records()?;

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].memory_id, record.memory_id);

        let package = build_context_package(
            &records,
            QueryRequest {
                text: "Am I allowed to use a USB drive?".to_string(),
                actor: "user".to_string(),
                action: "use".to_string(),
                resource: "usb_drive".to_string(),
                as_of: OffsetDateTime::now_utc(),
            },
            "txn_fixture",
        )?;

        assert_eq!(package.answer.result, memory_kernel_core::AnswerResult::Deny);
        Ok(())
    }

    // Test IDs: TID-003
    #[test]
    fn supersedes_links_round_trip_by_memory_version_id() -> Result<()> {
        let mut store = SqliteStore::open(Path::new(":memory:"))?;
        store.migrate()?;

        let entity_id = MemoryId::new();
        let old = MemoryRecord {
            memory_version_id: MemoryVersionId::new(),
            memory_id: entity_id,
            version: 1,
            created_at: OffsetDateTime::now_utc(),
            effective_at: OffsetDateTime::now_utc(),
            truth_status: TruthStatus::Asserted,
            authority: Authority::Authoritative,
            confidence: Some(0.8),
            writer: "tester".to_string(),
            justification: "seed policy v1".to_string(),
            provenance: Provenance {
                source_uri: "file:///policy.md".to_string(),
                source_hash: Some("sha256:abc123".to_string()),
                evidence: vec![],
            },
            supersedes: vec![],
            contradicts: vec![],
            payload: MemoryPayload::Constraint(ConstraintPayload {
                scope: ConstraintScope {
                    actor: "user".to_string(),
                    action: "use".to_string(),
                    resource: "usb_drive".to_string(),
                },
                effect: ConstraintEffect::Deny,
                note: None,
            }),
        };

        let new = MemoryRecord {
            memory_version_id: MemoryVersionId::new(),
            memory_id: entity_id,
            version: 2,
            created_at: OffsetDateTime::now_utc(),
            effective_at: OffsetDateTime::now_utc(),
            truth_status: TruthStatus::Asserted,
            authority: Authority::Authoritative,
            confidence: Some(0.95),
            writer: "tester".to_string(),
            justification: "seed policy v2".to_string(),
            provenance: Provenance {
                source_uri: "file:///policy.md".to_string(),
                source_hash: Some("sha256:def456".to_string()),
                evidence: vec![],
            },
            supersedes: vec![old.memory_version_id],
            contradicts: vec![],
            payload: MemoryPayload::Constraint(ConstraintPayload {
                scope: ConstraintScope {
                    actor: "user".to_string(),
                    action: "use".to_string(),
                    resource: "usb_drive".to_string(),
                },
                effect: ConstraintEffect::Deny,
                note: Some("new baseline".to_string()),
            }),
        };

        store.write_record(&old)?;
        store.write_record(&new)?;

        let records = store.list_records()?;
        let Some(loaded_new) =
            records.iter().find(|record| record.memory_version_id == new.memory_version_id)
        else {
            return Err(anyhow!(
                "new record not found by memory_version_id {}",
                new.memory_version_id
            ));
        };

        assert_eq!(loaded_new.supersedes, vec![old.memory_version_id]);
        Ok(())
    }

    // Test IDs: TDB-001
    #[test]
    fn migrate_legacy_v1_database_to_v2() -> Result<()> {
        let mut store = SqliteStore::open(Path::new(":memory:"))?;
        store.conn.execute_batch(CREATE_SCHEMA_MIGRATIONS_SQL)?;
        store.conn.execute_batch(MIGRATION_001_SQL)?;

        let old_memory_id = MemoryId::new();
        let new_memory_id = MemoryId::new();
        let created_at = "2026-01-01T00:00:00Z";

        insert_legacy_constraint_record(
            &store.conn,
            old_memory_id,
            created_at,
            0.8_f64,
            "legacy v1 old",
        )?;
        insert_legacy_constraint_record(
            &store.conn,
            new_memory_id,
            created_at,
            0.95_f64,
            "legacy v1 new",
        )?;
        insert_legacy_constraint_payload(&store.conn, old_memory_id)?;
        insert_legacy_constraint_payload(&store.conn, new_memory_id)?;
        insert_legacy_supersedes_link(&store.conn, new_memory_id, old_memory_id, created_at)?;

        store.migrate()?;

        let version = current_schema_version(&store.conn)?;
        assert_eq!(version, 2);

        let records = store.list_records()?;
        assert_eq!(records.len(), 2);

        let Some(old) = records.iter().find(|record| record.memory_id == old_memory_id) else {
            return Err(anyhow!("old migrated record not found"));
        };
        let Some(new) = records.iter().find(|record| record.memory_id == new_memory_id) else {
            return Err(anyhow!("new migrated record not found"));
        };

        assert_eq!(new.supersedes, vec![old.memory_version_id]);
        Ok(())
    }

    // Test IDs: TDB-004
    #[test]
    fn migrate_rejects_invalid_legacy_schema_without_identity_columns() -> Result<()> {
        let mut store = SqliteStore::open(Path::new(":memory:"))?;
        store.conn.execute_batch(CREATE_SCHEMA_MIGRATIONS_SQL)?;
        store.conn.execute_batch(
            "CREATE TABLE memory_records(
                id INTEGER PRIMARY KEY,
                version INTEGER NOT NULL
            );",
        )?;

        let err = match store.migrate() {
            Ok(()) => return Err(anyhow!("expected migration to fail on invalid legacy schema")),
            Err(err) => err,
        };

        assert!(err
            .to_string()
            .contains("memory_records has neither memory_id nor memory_version_id"));

        Ok(())
    }

    // Test IDs: TDB-005
    #[test]
    fn schema_status_reports_pending_migration_for_legacy_v1() -> Result<()> {
        let store = SqliteStore::open(Path::new(":memory:"))?;
        store.conn.execute_batch(CREATE_SCHEMA_MIGRATIONS_SQL)?;
        store.conn.execute_batch(MIGRATION_001_SQL)?;

        let status = store.schema_status()?;
        assert_eq!(status.current_version, 1);
        assert_eq!(status.target_version, 2);
        assert_eq!(status.pending_versions, vec![2]);
        assert!(status.inferred_from_legacy);

        Ok(())
    }

    // Test IDs: TDB-006
    #[test]
    fn export_and_import_snapshot_round_trip() -> Result<()> {
        let mut source = SqliteStore::open(Path::new(":memory:"))?;
        source.migrate()?;

        let record = MemoryRecord {
            memory_version_id: MemoryVersionId::new(),
            memory_id: MemoryId::new(),
            version: 1,
            created_at: OffsetDateTime::now_utc(),
            effective_at: OffsetDateTime::now_utc(),
            truth_status: TruthStatus::Asserted,
            authority: Authority::Authoritative,
            confidence: Some(0.95),
            writer: "tester".to_string(),
            justification: "seed export".to_string(),
            provenance: Provenance {
                source_uri: "file:///policy.md".to_string(),
                source_hash: Some("sha256:abc123".to_string()),
                evidence: vec![],
            },
            supersedes: vec![],
            contradicts: vec![],
            payload: MemoryPayload::Constraint(ConstraintPayload {
                scope: ConstraintScope {
                    actor: "user".to_string(),
                    action: "use".to_string(),
                    resource: "usb_drive".to_string(),
                },
                effect: ConstraintEffect::Deny,
                note: None,
            }),
        };

        source.write_record(&record)?;
        let package = build_context_package(
            &source.list_records()?,
            QueryRequest {
                text: "Am I allowed to use a USB drive?".to_string(),
                actor: "user".to_string(),
                action: "use".to_string(),
                resource: "usb_drive".to_string(),
                as_of: OffsetDateTime::now_utc(),
            },
            "txn_export_import",
        )?;
        source.save_context_package(&package)?;

        let export_dir = std::env::temp_dir().join(format!("memorykernel-export-{}", Ulid::new()));
        let manifest = source.export_snapshot(&export_dir)?;
        assert_eq!(manifest.files.len(), 2);
        assert!(export_dir.join("memory_records.ndjson").exists());
        assert!(export_dir.join("context_packages.ndjson").exists());
        assert!(export_dir.join("manifest.json").exists());

        let mut target = SqliteStore::open(Path::new(":memory:"))?;
        let summary = target.import_snapshot(&export_dir, true)?;
        assert_eq!(summary.imported_records, 1);
        assert_eq!(summary.imported_context_packages, 1);

        let imported_records = target.list_records()?;
        assert_eq!(imported_records.len(), 1);
        assert_eq!(imported_records[0].memory_version_id, record.memory_version_id);

        let imported_package = target.get_context_package(&package.context_package_id)?;
        assert!(imported_package.is_some());

        fs::remove_dir_all(&export_dir).with_context(|| {
            format!("failed to cleanup temp export dir {}", export_dir.display())
        })?;

        Ok(())
    }

    // Test IDs: TDB-009
    #[test]
    fn import_rejects_manifest_digest_mismatch() -> Result<()> {
        use std::io::Write as _;

        let mut source = SqliteStore::open(Path::new(":memory:"))?;
        source.migrate()?;

        let record = mk_store_constraint_record(
            MemoryId::new(),
            1,
            TruthStatus::Asserted,
            Some(0.95),
            ConstraintEffect::Deny,
        );
        source.write_record(&record)?;

        let export_dir = std::env::temp_dir().join(format!("memorykernel-export-{}", Ulid::new()));
        source.export_snapshot(&export_dir)?;

        let records_path = export_dir.join("memory_records.ndjson");
        let mut tampered = std::fs::OpenOptions::new().append(true).open(&records_path)?;
        writeln!(tampered, "{{\"tampered\":true}}")?;

        let mut target = SqliteStore::open(Path::new(":memory:"))?;
        let Err(err) = target.import_snapshot(&export_dir, true) else {
            return Err(anyhow!("expected import failure for mismatched manifest digest"));
        };
        assert!(err.to_string().contains("manifest digest mismatch for memory_records.ndjson"));

        fs::remove_dir_all(&export_dir).with_context(|| {
            format!("failed to cleanup temp export dir {}", export_dir.display())
        })?;

        Ok(())
    }

    // Test IDs: TDB-007
    #[test]
    fn backup_and_restore_database_round_trip() -> Result<()> {
        let mut source = SqliteStore::open(Path::new(":memory:"))?;
        source.migrate()?;

        let record = MemoryRecord {
            memory_version_id: MemoryVersionId::new(),
            memory_id: MemoryId::new(),
            version: 1,
            created_at: OffsetDateTime::now_utc(),
            effective_at: OffsetDateTime::now_utc(),
            truth_status: TruthStatus::Asserted,
            authority: Authority::Authoritative,
            confidence: Some(0.91),
            writer: "tester".to_string(),
            justification: "seed backup".to_string(),
            provenance: Provenance {
                source_uri: "file:///policy.md".to_string(),
                source_hash: Some("sha256:backup123".to_string()),
                evidence: vec![],
            },
            supersedes: vec![],
            contradicts: vec![],
            payload: MemoryPayload::Constraint(ConstraintPayload {
                scope: ConstraintScope {
                    actor: "user".to_string(),
                    action: "use".to_string(),
                    resource: "usb_drive".to_string(),
                },
                effect: ConstraintEffect::Deny,
                note: Some("backup flow".to_string()),
            }),
        };
        source.write_record(&record)?;

        let backup_file =
            std::env::temp_dir().join(format!("memorykernel-backup-{}.sqlite3", Ulid::new()));
        source.backup_database(&backup_file)?;

        let mut target = SqliteStore::open(Path::new(":memory:"))?;
        target.restore_database(&backup_file)?;
        let restored = target.list_records()?;
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].memory_version_id, record.memory_version_id);

        fs::remove_file(&backup_file).with_context(|| {
            format!("failed to cleanup temp backup file {}", backup_file.display())
        })?;

        Ok(())
    }

    // Test IDs: TDB-008
    #[test]
    fn integrity_check_reports_clean_database() -> Result<()> {
        let mut store = SqliteStore::open(Path::new(":memory:"))?;
        store.migrate()?;

        let report = store.integrity_check()?;
        assert!(report.quick_check_ok);
        assert!(report.foreign_key_violations.is_empty());
        assert_eq!(report.schema_status.current_version, 2);

        Ok(())
    }

    // Test IDs: TCONC-001
    #[test]
    fn concurrent_writes_and_reads_preserve_integrity() -> Result<()> {
        let db_path =
            std::env::temp_dir().join(format!("memorykernel-concurrency-{}.sqlite3", Ulid::new()));
        {
            let mut init = SqliteStore::open(&db_path)?;
            init.migrate()?;
        }

        let writer_threads = 4;
        let writes_per_thread = 20;
        let reader_threads = 2;
        let read_iterations = 30;

        let mut handles = Vec::new();

        for _ in 0..writer_threads {
            let writer_path = db_path.clone();
            handles.push(thread::spawn(move || -> Result<()> {
                let mut store = SqliteStore::open(&writer_path)?;
                store.migrate()?;
                for _ in 0..writes_per_thread {
                    let record = MemoryRecord {
                        memory_version_id: MemoryVersionId::new(),
                        memory_id: MemoryId::new(),
                        version: 1,
                        created_at: OffsetDateTime::now_utc(),
                        effective_at: OffsetDateTime::now_utc(),
                        truth_status: TruthStatus::Asserted,
                        authority: Authority::Authoritative,
                        confidence: Some(0.8),
                        writer: "thread-writer".to_string(),
                        justification: "concurrency fixture".to_string(),
                        provenance: Provenance {
                            source_uri: "file:///concurrency.md".to_string(),
                            source_hash: Some("sha256:abc123".to_string()),
                            evidence: vec![],
                        },
                        supersedes: vec![],
                        contradicts: vec![],
                        payload: MemoryPayload::Constraint(ConstraintPayload {
                            scope: ConstraintScope {
                                actor: "user".to_string(),
                                action: "use".to_string(),
                                resource: "usb_drive".to_string(),
                            },
                            effect: ConstraintEffect::Deny,
                            note: Some("concurrency write".to_string()),
                        }),
                    };
                    store.write_record(&record)?;
                }
                Ok(())
            }));
        }

        for _ in 0..reader_threads {
            let reader_path = db_path.clone();
            handles.push(thread::spawn(move || -> Result<()> {
                let store = SqliteStore::open(&reader_path)?;
                for _ in 0..read_iterations {
                    let _ = store.list_records()?;
                }
                Ok(())
            }));
        }

        for handle in handles {
            let Ok(thread_result) = handle.join() else {
                return Err(anyhow!("concurrency thread panicked"));
            };
            thread_result?;
        }

        let store = SqliteStore::open(&db_path)?;
        let records = store.list_records()?;
        assert_eq!(records.len(), writer_threads * writes_per_thread);

        let report = store.integrity_check()?;
        assert!(report.quick_check_ok);
        assert!(report.foreign_key_violations.is_empty());

        for suffix in ["", "-wal", "-shm"] {
            let path = if suffix.is_empty() {
                db_path.clone()
            } else {
                std::path::PathBuf::from(format!("{}{}", db_path.display(), suffix))
            };
            if path.exists() {
                fs::remove_file(&path)
                    .with_context(|| format!("failed to cleanup sqlite file {}", path.display()))?;
            }
        }

        Ok(())
    }
}
