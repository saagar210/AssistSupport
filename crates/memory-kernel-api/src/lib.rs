use std::path::PathBuf;

use anyhow::{anyhow, Result};
use memory_kernel_core::{
    build_context_package, build_recall_context_package, default_recall_record_types, Authority,
    ConstraintEffect, ConstraintPayload, ConstraintScope, ContextPackage, DecisionPayload,
    EventPayload, LinkType, MemoryId, MemoryPayload, MemoryRecord, MemoryVersionId,
    PreferencePayload, QueryRequest, RecordType, TruthStatus,
};
use memory_kernel_store_sqlite::{SchemaStatus, SqliteStore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

pub const API_CONTRACT_VERSION: &str = "api.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrateResult {
    pub dry_run: bool,
    pub current_version: i64,
    pub target_version: i64,
    pub would_apply_versions: Vec<i64>,
    pub inferred_from_legacy: bool,
    pub after_version: Option<i64>,
    pub up_to_date: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddConstraintRequest {
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub effect: ConstraintEffect,
    pub note: Option<String>,
    pub memory_id: Option<MemoryId>,
    pub version: u32,
    pub writer: String,
    pub justification: String,
    pub source_uri: String,
    pub source_hash: Option<String>,
    pub evidence: Vec<String>,
    pub confidence: Option<f32>,
    pub truth_status: TruthStatus,
    pub authority: Authority,
    #[serde(with = "time::serde::rfc3339::option")]
    pub created_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub effective_at: Option<OffsetDateTime>,
    pub supersedes: Vec<MemoryVersionId>,
    pub contradicts: Vec<MemoryVersionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddSummaryRequest {
    pub record_type: RecordType,
    pub summary: String,
    pub memory_id: Option<MemoryId>,
    pub version: u32,
    pub writer: String,
    pub justification: String,
    pub source_uri: String,
    pub source_hash: Option<String>,
    pub evidence: Vec<String>,
    pub confidence: Option<f32>,
    pub truth_status: TruthStatus,
    pub authority: Authority,
    #[serde(with = "time::serde::rfc3339::option")]
    pub created_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub effective_at: Option<OffsetDateTime>,
    pub supersedes: Vec<MemoryVersionId>,
    pub contradicts: Vec<MemoryVersionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddLinkRequest {
    pub from: MemoryVersionId,
    pub to: MemoryVersionId,
    pub relation: LinkType,
    pub writer: String,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddLinkResult {
    pub from_memory_version_id: MemoryVersionId,
    pub to_memory_version_id: MemoryVersionId,
    pub relation: LinkType,
    pub writer: String,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AskRequest {
    pub text: String,
    pub actor: String,
    pub action: String,
    pub resource: String,
    #[serde(with = "time::serde::rfc3339::option")]
    pub as_of: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecallRequest {
    pub text: String,
    pub record_types: Vec<RecordType>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub as_of: Option<OffsetDateTime>,
}

#[derive(Debug, Clone)]
pub struct MemoryKernelApi {
    db_path: PathBuf,
}

impl MemoryKernelApi {
    #[must_use]
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    fn open_store(&self) -> Result<SqliteStore> {
        SqliteStore::open(&self.db_path)
    }

    /// Inspect schema status without mutating data.
    ///
    /// # Errors
    /// Returns an error when the `SQLite` database cannot be opened or queried.
    pub fn schema_status(&self) -> Result<SchemaStatus> {
        let store = self.open_store()?;
        store.schema_status()
    }

    /// Apply pending migrations, or return planned versions for dry-run mode.
    ///
    /// # Errors
    /// Returns an error when migration planning or execution fails.
    pub fn migrate(&self, dry_run: bool) -> Result<MigrateResult> {
        let mut store = self.open_store()?;
        let before = store.schema_status()?;
        if dry_run {
            return Ok(MigrateResult {
                dry_run: true,
                current_version: before.current_version,
                target_version: before.target_version,
                would_apply_versions: before.pending_versions,
                inferred_from_legacy: before.inferred_from_legacy,
                after_version: None,
                up_to_date: None,
            });
        }

        let planned_versions = before.pending_versions;
        store.migrate()?;
        let after = store.schema_status()?;
        Ok(MigrateResult {
            dry_run: false,
            current_version: before.current_version,
            target_version: before.target_version,
            would_apply_versions: planned_versions,
            inferred_from_legacy: before.inferred_from_legacy,
            after_version: Some(after.current_version),
            up_to_date: Some(after.pending_versions.is_empty()),
        })
    }

    /// Add one `constraint` memory record.
    ///
    /// # Errors
    /// Returns an error when record validation or persistence fails.
    pub fn add_constraint(&self, input: AddConstraintRequest) -> Result<MemoryRecord> {
        let mut store = self.open_store()?;
        store.migrate()?;
        let record = build_constraint_record(input);
        store.write_record(&record)?;
        Ok(record)
    }

    /// Add one summary-backed memory record (`decision`, `preference`, `event`, or `outcome`).
    ///
    /// # Errors
    /// Returns an error when an unsupported record type is provided, or persistence fails.
    pub fn add_summary(&self, input: AddSummaryRequest) -> Result<MemoryRecord> {
        let mut store = self.open_store()?;
        store.migrate()?;
        let record = build_summary_record(input)?;
        store.write_record(&record)?;
        Ok(record)
    }

    /// Add one lineage link between memory versions.
    ///
    /// # Errors
    /// Returns an error when link persistence fails.
    pub fn add_link(&self, input: AddLinkRequest) -> Result<AddLinkResult> {
        let mut store = self.open_store()?;
        store.migrate()?;
        store.add_link(
            input.from,
            input.to,
            input.relation,
            &input.writer,
            &input.justification,
        )?;

        Ok(AddLinkResult {
            from_memory_version_id: input.from,
            to_memory_version_id: input.to,
            relation: input.relation,
            writer: input.writer,
            justification: input.justification,
        })
    }

    /// Execute a policy query and persist the generated context package.
    ///
    /// # Errors
    /// Returns an error when retrieval or persistence fails.
    pub fn query_ask(&self, input: AskRequest) -> Result<ContextPackage> {
        let mut store = self.open_store()?;
        store.migrate()?;

        let as_of = input.as_of.unwrap_or_else(OffsetDateTime::now_utc);
        let records = store.list_records()?;
        let snapshot_id = compute_snapshot_id(
            &records,
            as_of,
            &input.text,
            &[
                "query_mode=policy".to_string(),
                format!("actor={}", input.actor),
                format!("action={}", input.action),
                format!("resource={}", input.resource),
            ],
        );

        let package = build_context_package(
            &records,
            QueryRequest {
                text: input.text,
                actor: input.actor,
                action: input.action,
                resource: input.resource,
                as_of,
            },
            &snapshot_id,
        )?;
        store.save_context_package(&package)?;
        Ok(package)
    }

    /// Execute deterministic recall retrieval across selected record types.
    ///
    /// # Errors
    /// Returns an error when retrieval or persistence fails.
    pub fn query_recall(&self, input: RecallRequest) -> Result<ContextPackage> {
        let mut store = self.open_store()?;
        store.migrate()?;

        let as_of = input.as_of.unwrap_or_else(OffsetDateTime::now_utc);
        let selected_record_types = if input.record_types.is_empty() {
            default_recall_record_types()
        } else {
            input.record_types
        };
        let records = store.list_records()?;

        let mut record_type_names = selected_record_types
            .iter()
            .map(|record_type| record_type.as_str())
            .collect::<Vec<_>>();
        record_type_names.sort_unstable();

        let snapshot_id = compute_snapshot_id(
            &records,
            as_of,
            &input.text,
            &[
                "query_mode=recall".to_string(),
                format!("record_types={}", record_type_names.join(",")),
            ],
        );

        let package = build_recall_context_package(
            &records,
            QueryRequest {
                text: input.text,
                actor: "*".to_string(),
                action: "*".to_string(),
                resource: "*".to_string(),
                as_of,
            },
            &snapshot_id,
            &selected_record_types,
        )?;
        store.save_context_package(&package)?;
        Ok(package)
    }

    /// Fetch a previously persisted context package.
    ///
    /// # Errors
    /// Returns an error when lookup fails or package does not exist.
    pub fn context_show(&self, context_package_id: &str) -> Result<ContextPackage> {
        let mut store = self.open_store()?;
        store.migrate()?;
        let package = store
            .get_context_package(context_package_id)?
            .ok_or_else(|| anyhow!("context package not found: {context_package_id}"))?;
        Ok(package)
    }
}

fn build_constraint_record(input: AddConstraintRequest) -> MemoryRecord {
    let created_at = input.created_at.unwrap_or_else(OffsetDateTime::now_utc);
    let effective_at = input.effective_at.unwrap_or(created_at);

    MemoryRecord {
        memory_version_id: MemoryVersionId::new(),
        memory_id: input.memory_id.unwrap_or_default(),
        version: input.version,
        created_at,
        effective_at,
        truth_status: input.truth_status,
        authority: input.authority,
        confidence: input.confidence,
        writer: input.writer,
        justification: input.justification,
        provenance: memory_kernel_core::Provenance {
            source_uri: input.source_uri,
            source_hash: input.source_hash,
            evidence: input.evidence,
        },
        supersedes: input.supersedes,
        contradicts: input.contradicts,
        payload: MemoryPayload::Constraint(ConstraintPayload {
            scope: ConstraintScope {
                actor: input.actor,
                action: input.action,
                resource: input.resource,
            },
            effect: input.effect,
            note: input.note,
        }),
    }
}

fn build_summary_record(input: AddSummaryRequest) -> Result<MemoryRecord> {
    let created_at = input.created_at.unwrap_or_else(OffsetDateTime::now_utc);
    let effective_at = input.effective_at.unwrap_or(created_at);

    let payload = match input.record_type {
        RecordType::Decision => MemoryPayload::Decision(DecisionPayload { summary: input.summary }),
        RecordType::Preference => {
            MemoryPayload::Preference(PreferencePayload { summary: input.summary })
        }
        RecordType::Event => MemoryPayload::Event(EventPayload { summary: input.summary }),
        RecordType::Outcome => {
            MemoryPayload::Outcome(memory_kernel_core::OutcomePayload { summary: input.summary })
        }
        RecordType::Constraint => {
            return Err(anyhow!("add_summary does not support record_type=constraint"));
        }
    };

    Ok(MemoryRecord {
        memory_version_id: MemoryVersionId::new(),
        memory_id: input.memory_id.unwrap_or_default(),
        version: input.version,
        created_at,
        effective_at,
        truth_status: input.truth_status,
        authority: input.authority,
        confidence: input.confidence,
        writer: input.writer,
        justification: input.justification,
        provenance: memory_kernel_core::Provenance {
            source_uri: input.source_uri,
            source_hash: input.source_hash,
            evidence: input.evidence,
        },
        supersedes: input.supersedes,
        contradicts: input.contradicts,
        payload,
    })
}

fn compute_snapshot_id(
    records: &[MemoryRecord],
    as_of: OffsetDateTime,
    text: &str,
    scope_parts: &[String],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher.update(as_of.unix_timestamp().to_string().as_bytes());
    for part in scope_parts {
        hasher.update(part.as_bytes());
    }

    let mut sorted_ids = records
        .iter()
        .map(|record| format!("{}:{}", record.memory_id, record.memory_version_id))
        .collect::<Vec<_>>();
    sorted_ids.sort_unstable();

    for value in sorted_ids {
        hasher.update(value.as_bytes());
    }

    let digest = hasher.finalize();
    let digest_hex = format!("{digest:x}");
    format!("txn_{}", &digest_hex[..16])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_temp_db_path() -> PathBuf {
        std::env::temp_dir().join(format!("memorykernel-api-{}.sqlite3", ulid::Ulid::new()))
    }

    // Test IDs: TAPI-001
    #[test]
    fn api_add_query_and_show_round_trip() -> Result<()> {
        let db_path = unique_temp_db_path();
        let api = MemoryKernelApi::new(db_path.clone());

        let _record = api.add_constraint(AddConstraintRequest {
            actor: "user".to_string(),
            action: "use".to_string(),
            resource: "usb_drive".to_string(),
            effect: ConstraintEffect::Deny,
            note: None,
            memory_id: None,
            version: 1,
            writer: "tester".to_string(),
            justification: "api fixture".to_string(),
            source_uri: "file:///policy.md".to_string(),
            source_hash: Some("sha256:abc123".to_string()),
            evidence: Vec::new(),
            confidence: Some(0.9),
            truth_status: TruthStatus::Asserted,
            authority: Authority::Authoritative,
            created_at: None,
            effective_at: None,
            supersedes: Vec::new(),
            contradicts: Vec::new(),
        })?;

        let package = api.query_ask(AskRequest {
            text: "Am I allowed to use a USB drive?".to_string(),
            actor: "user".to_string(),
            action: "use".to_string(),
            resource: "usb_drive".to_string(),
            as_of: None,
        })?;

        let loaded = api.context_show(&package.context_package_id)?;
        assert_eq!(loaded.context_package_id, package.context_package_id);

        let _ = std::fs::remove_file(&db_path);
        Ok(())
    }

    // Test IDs: TAPI-002
    #[test]
    fn api_recall_query_supports_mixed_summary_records() -> Result<()> {
        let db_path = unique_temp_db_path();
        let api = MemoryKernelApi::new(db_path.clone());

        let _decision = api.add_summary(AddSummaryRequest {
            record_type: RecordType::Decision,
            summary: "Decision: USB media access must be approved".to_string(),
            memory_id: None,
            version: 1,
            writer: "tester".to_string(),
            justification: "api recall fixture".to_string(),
            source_uri: "file:///decision.md".to_string(),
            source_hash: Some("sha256:abc123".to_string()),
            evidence: Vec::new(),
            confidence: Some(0.8),
            truth_status: TruthStatus::Observed,
            authority: Authority::Authoritative,
            created_at: None,
            effective_at: None,
            supersedes: Vec::new(),
            contradicts: Vec::new(),
        })?;

        let _outcome = api.add_summary(AddSummaryRequest {
            record_type: RecordType::Outcome,
            summary: "Outcome: USB compliance improved after controls".to_string(),
            memory_id: None,
            version: 1,
            writer: "tester".to_string(),
            justification: "api recall fixture".to_string(),
            source_uri: "file:///outcome.md".to_string(),
            source_hash: Some("sha256:def456".to_string()),
            evidence: Vec::new(),
            confidence: Some(0.9),
            truth_status: TruthStatus::Observed,
            authority: Authority::Authoritative,
            created_at: None,
            effective_at: None,
            supersedes: Vec::new(),
            contradicts: Vec::new(),
        })?;

        let package = api.query_recall(RecallRequest {
            text: "usb compliance".to_string(),
            record_types: vec![RecordType::Decision, RecordType::Outcome],
            as_of: None,
        })?;

        assert_eq!(package.determinism.ruleset_version, "recall-ordering.v1");
        assert!(!package.selected_items.is_empty());

        let _ = std::fs::remove_file(&db_path);
        Ok(())
    }

    // Test IDs: TAPI-003
    #[test]
    fn api_recall_query_defaults_to_non_constraint_record_types() -> Result<()> {
        let db_path = unique_temp_db_path();
        let api = MemoryKernelApi::new(db_path.clone());

        let _constraint = api.add_constraint(AddConstraintRequest {
            actor: "user".to_string(),
            action: "use".to_string(),
            resource: "usb_drive".to_string(),
            effect: ConstraintEffect::Deny,
            note: Some("constraint should not be in default recall scope".to_string()),
            memory_id: None,
            version: 1,
            writer: "tester".to_string(),
            justification: "api recall default-scope fixture".to_string(),
            source_uri: "file:///constraint.md".to_string(),
            source_hash: Some("sha256:abc123".to_string()),
            evidence: Vec::new(),
            confidence: Some(0.9),
            truth_status: TruthStatus::Observed,
            authority: Authority::Authoritative,
            created_at: None,
            effective_at: None,
            supersedes: Vec::new(),
            contradicts: Vec::new(),
        })?;

        let _decision = api.add_summary(AddSummaryRequest {
            record_type: RecordType::Decision,
            summary: "Decision: USB usage requires manager approval".to_string(),
            memory_id: None,
            version: 1,
            writer: "tester".to_string(),
            justification: "api recall default-scope fixture".to_string(),
            source_uri: "file:///decision.md".to_string(),
            source_hash: Some("sha256:def456".to_string()),
            evidence: Vec::new(),
            confidence: Some(0.7),
            truth_status: TruthStatus::Observed,
            authority: Authority::Derived,
            created_at: None,
            effective_at: None,
            supersedes: Vec::new(),
            contradicts: Vec::new(),
        })?;

        let package = api.query_recall(RecallRequest {
            text: "usb usage".to_string(),
            record_types: Vec::new(),
            as_of: None,
        })?;

        assert_eq!(package.determinism.ruleset_version, "recall-ordering.v1");
        assert!(!package.selected_items.is_empty());
        assert!(package
            .selected_items
            .iter()
            .all(|item| item.record_type != RecordType::Constraint));

        let _ = std::fs::remove_file(&db_path);
        Ok(())
    }
}
