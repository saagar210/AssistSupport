#![forbid(unsafe_code)]

use std::path::Path;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use memory_kernel_core::ContextPackage;
use multi_agent_center_domain::{
    now_utc, ContextPackageEnvelope, EventRow, GateDecision, GateDecisionRecord, GateKind,
    ProposedMemoryWrite, RunId, RunRecord, RunStatus, StepContextPackageRecord, StepId, StepRecord,
    StepStatus, TraceEvent, TraceEventType, WorkflowSnapshotRecord,
};
use multi_agent_center_trace_core::TraceStore;
use rusqlite::{params, Connection, OptionalExtension};
use time::OffsetDateTime;
use ulid::Ulid;

const TRACE_SCHEMA_VERSION: i64 = 3;

const SCHEMA_V2: &str = r"
CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS workflow_snapshots (
  workflow_hash TEXT PRIMARY KEY,
  normalization_version INTEGER NOT NULL,
  source_format TEXT NOT NULL,
  source_yaml_hash TEXT NOT NULL,
  normalized_json TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS runs (
  run_id TEXT PRIMARY KEY,
  workflow_name TEXT NOT NULL,
  workflow_version TEXT NOT NULL,
  workflow_hash TEXT NOT NULL,
  as_of TEXT NOT NULL,
  as_of_was_default INTEGER NOT NULL CHECK (as_of_was_default IN (0,1)),
  started_at TEXT NOT NULL,
  ended_at TEXT,
  status TEXT NOT NULL CHECK (status IN ('pending','running','succeeded','failed','rejected')),
  replay_of_run_id TEXT,
  external_correlation_id TEXT,
  engine_version TEXT NOT NULL,
  cli_args_json TEXT NOT NULL,
  manifest_hash TEXT,
  manifest_signature TEXT,
  manifest_signature_status TEXT NOT NULL DEFAULT 'unsigned',
  FOREIGN KEY (workflow_hash) REFERENCES workflow_snapshots(workflow_hash)
);

CREATE TABLE IF NOT EXISTS steps (
  step_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  step_index INTEGER NOT NULL,
  step_key TEXT NOT NULL,
  agent_name TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('pending','running','succeeded','failed','rejected','skipped')),
  started_at TEXT,
  ended_at TEXT,
  task_payload_json TEXT NOT NULL,
  constraints_json TEXT NOT NULL,
  permissions_json TEXT NOT NULL,
  input_hash TEXT NOT NULL,
  output_hash TEXT,
  error_json TEXT,
  UNIQUE(run_id, step_index),
  UNIQUE(run_id, step_key),
  FOREIGN KEY (run_id) REFERENCES runs(run_id)
);

CREATE TABLE IF NOT EXISTS trace_events (
  event_seq INTEGER PRIMARY KEY AUTOINCREMENT,
  event_id TEXT NOT NULL UNIQUE,
  run_id TEXT NOT NULL,
  step_id TEXT,
  event_type TEXT NOT NULL,
  occurred_at TEXT NOT NULL,
  recorded_at TEXT NOT NULL,
  actor_type TEXT NOT NULL,
  actor_id TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  payload_hash TEXT NOT NULL,
  prev_event_hash TEXT,
  event_hash TEXT NOT NULL,
  FOREIGN KEY (run_id) REFERENCES runs(run_id),
  FOREIGN KEY (step_id) REFERENCES steps(step_id)
);

CREATE TABLE IF NOT EXISTS step_context_packages (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  run_id TEXT NOT NULL,
  step_id TEXT NOT NULL,
  package_slot INTEGER NOT NULL,
  context_package_id TEXT NOT NULL,
  generated_at TEXT NOT NULL,
  query_json TEXT NOT NULL,
  determinism_json TEXT NOT NULL,
  answer_json TEXT NOT NULL,
  ordering_trace_json TEXT NOT NULL,
  package_json TEXT NOT NULL,
  package_hash TEXT NOT NULL,
  UNIQUE(step_id, package_slot),
  FOREIGN KEY (run_id) REFERENCES runs(run_id),
  FOREIGN KEY (step_id) REFERENCES steps(step_id)
);

CREATE TABLE IF NOT EXISTS step_context_selected (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  step_context_package_id INTEGER NOT NULL,
  rank INTEGER NOT NULL,
  memory_version_id TEXT NOT NULL,
  memory_id TEXT NOT NULL,
  version INTEGER NOT NULL,
  record_type TEXT NOT NULL,
  truth_status TEXT NOT NULL,
  confidence REAL,
  authority TEXT NOT NULL,
  why_reasons_json TEXT NOT NULL,
  rule_scores_json TEXT,
  injected INTEGER NOT NULL CHECK (injected IN (0,1)),
  permission_decision TEXT NOT NULL,
  permission_reason TEXT,
  FOREIGN KEY (step_context_package_id) REFERENCES step_context_packages(id)
);

CREATE TABLE IF NOT EXISTS step_context_excluded (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  step_context_package_id INTEGER NOT NULL,
  rank INTEGER NOT NULL,
  memory_version_id TEXT NOT NULL,
  memory_id TEXT NOT NULL,
  version INTEGER NOT NULL,
  record_type TEXT NOT NULL,
  truth_status TEXT NOT NULL,
  confidence REAL,
  authority TEXT NOT NULL,
  why_reasons_json TEXT NOT NULL,
  FOREIGN KEY (step_context_package_id) REFERENCES step_context_packages(id)
);

CREATE TABLE IF NOT EXISTS step_gate_decisions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  run_id TEXT NOT NULL,
  step_id TEXT NOT NULL,
  gate_kind TEXT NOT NULL,
  gate_name TEXT NOT NULL,
  subject_type TEXT NOT NULL,
  memory_id TEXT,
  version INTEGER,
  memory_version_id TEXT,
  decision TEXT NOT NULL,
  reason_codes_json TEXT NOT NULL,
  notes TEXT,
  decided_by TEXT NOT NULL,
  decided_at TEXT NOT NULL,
  source_ruleset_version INTEGER,
  evidence_json TEXT,
  CHECK (
    gate_kind <> 'trust'
    OR subject_type <> 'memory_ref'
    OR (
      memory_id IS NOT NULL
      AND version IS NOT NULL
      AND memory_version_id IS NOT NULL
    )
  ),
  FOREIGN KEY (run_id) REFERENCES runs(run_id),
  FOREIGN KEY (step_id) REFERENCES steps(step_id)
);

CREATE TABLE IF NOT EXISTS provider_calls (
  provider_call_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  step_id TEXT NOT NULL,
  provider_name TEXT NOT NULL,
  adapter_version TEXT NOT NULL,
  model_id TEXT NOT NULL,
  request_json TEXT NOT NULL,
  request_hash TEXT NOT NULL,
  response_json TEXT,
  response_hash TEXT,
  latency_ms INTEGER,
  input_tokens INTEGER,
  output_tokens INTEGER,
  started_at TEXT NOT NULL,
  ended_at TEXT,
  status TEXT NOT NULL,
  error_text TEXT,
  FOREIGN KEY (run_id) REFERENCES runs(run_id),
  FOREIGN KEY (step_id) REFERENCES steps(step_id)
);

CREATE TABLE IF NOT EXISTS proposed_memory_writes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  run_id TEXT NOT NULL,
  step_id TEXT NOT NULL,
  proposal_index INTEGER NOT NULL,
  proposal_json TEXT NOT NULL,
  proposal_hash TEXT NOT NULL,
  disposition TEXT NOT NULL,
  disposition_reason TEXT,
  FOREIGN KEY (run_id) REFERENCES runs(run_id),
  FOREIGN KEY (step_id) REFERENCES steps(step_id)
);

CREATE INDEX IF NOT EXISTS idx_trace_events_run_seq ON trace_events(run_id, event_seq);
CREATE INDEX IF NOT EXISTS idx_trace_events_step_seq ON trace_events(step_id, event_seq);
CREATE INDEX IF NOT EXISTS idx_steps_run_index ON steps(run_id, step_index);
CREATE INDEX IF NOT EXISTS idx_selected_memory ON step_context_selected(memory_id, version);
CREATE INDEX IF NOT EXISTS idx_selected_memory_version ON step_context_selected(memory_version_id);
CREATE INDEX IF NOT EXISTS idx_excluded_memory ON step_context_excluded(memory_id, version);
CREATE INDEX IF NOT EXISTS idx_gate_step ON step_gate_decisions(step_id, gate_kind);
CREATE INDEX IF NOT EXISTS idx_gate_memory ON step_gate_decisions(memory_id, version, memory_version_id);
CREATE INDEX IF NOT EXISTS idx_provider_step ON provider_calls(step_id, started_at);

CREATE TRIGGER IF NOT EXISTS trg_trace_events_no_update
BEFORE UPDATE ON trace_events
BEGIN
  SELECT RAISE(FAIL, 'trace_events is append-only');
END;
CREATE TRIGGER IF NOT EXISTS trg_trace_events_no_delete
BEFORE DELETE ON trace_events
BEGIN
  SELECT RAISE(FAIL, 'trace_events is append-only');
END;

CREATE TRIGGER IF NOT EXISTS trg_step_context_selected_no_update
BEFORE UPDATE ON step_context_selected
BEGIN
  SELECT RAISE(FAIL, 'step_context_selected is append-only');
END;
CREATE TRIGGER IF NOT EXISTS trg_step_context_selected_no_delete
BEFORE DELETE ON step_context_selected
BEGIN
  SELECT RAISE(FAIL, 'step_context_selected is append-only');
END;

CREATE TRIGGER IF NOT EXISTS trg_step_context_excluded_no_update
BEFORE UPDATE ON step_context_excluded
BEGIN
  SELECT RAISE(FAIL, 'step_context_excluded is append-only');
END;
CREATE TRIGGER IF NOT EXISTS trg_step_context_excluded_no_delete
BEFORE DELETE ON step_context_excluded
BEGIN
  SELECT RAISE(FAIL, 'step_context_excluded is append-only');
END;

CREATE TRIGGER IF NOT EXISTS trg_step_gate_decisions_no_update
BEFORE UPDATE ON step_gate_decisions
BEGIN
  SELECT RAISE(FAIL, 'step_gate_decisions is append-only');
END;
CREATE TRIGGER IF NOT EXISTS trg_step_gate_decisions_no_delete
BEFORE DELETE ON step_gate_decisions
BEGIN
  SELECT RAISE(FAIL, 'step_gate_decisions is append-only');
END;
CREATE TRIGGER IF NOT EXISTS trg_step_gate_decisions_trust_memory_ref_identity
BEFORE INSERT ON step_gate_decisions
WHEN NEW.gate_kind = 'trust'
  AND NEW.subject_type = 'memory_ref'
  AND (
    NEW.memory_id IS NULL
    OR NEW.version IS NULL
    OR NEW.memory_version_id IS NULL
  )
BEGIN
  SELECT RAISE(FAIL, 'trust memory_ref gate decisions require memory_id, version, and memory_version_id');
END;

CREATE TRIGGER IF NOT EXISTS trg_provider_calls_no_update
BEFORE UPDATE ON provider_calls
BEGIN
  SELECT RAISE(FAIL, 'provider_calls is append-only');
END;
CREATE TRIGGER IF NOT EXISTS trg_provider_calls_no_delete
BEFORE DELETE ON provider_calls
BEGIN
  SELECT RAISE(FAIL, 'provider_calls is append-only');
END;

CREATE TRIGGER IF NOT EXISTS trg_proposed_memory_writes_no_update
BEFORE UPDATE ON proposed_memory_writes
BEGIN
  SELECT RAISE(FAIL, 'proposed_memory_writes is append-only');
END;
CREATE TRIGGER IF NOT EXISTS trg_proposed_memory_writes_no_delete
BEFORE DELETE ON proposed_memory_writes
BEGIN
  SELECT RAISE(FAIL, 'proposed_memory_writes is append-only');
END;
";

pub struct SqliteTraceStore {
    conn: Connection,
}

impl SqliteTraceStore {
    /// Open or create a `SQLite` trace database and configure local pragmas.
    ///
    /// # Errors
    /// Returns an error if opening the database or applying pragmas fails.
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
}

impl TraceStore for SqliteTraceStore {
    fn migrate(&self) -> Result<()> {
        self.conn
            .execute_batch(SCHEMA_V2)
            .context("failed to apply trace schema")?;

        ensure_column(&self.conn, "runs", "manifest_hash", "TEXT")?;
        ensure_column(&self.conn, "runs", "manifest_signature", "TEXT")?;
        ensure_column(
            &self.conn,
            "runs",
            "manifest_signature_status",
            "TEXT NOT NULL DEFAULT 'unsigned'",
        )?;
        ensure_column(&self.conn, "step_gate_decisions", "memory_id", "TEXT")?;
        ensure_column(&self.conn, "step_gate_decisions", "version", "INTEGER")?;
        ensure_column(
            &self.conn,
            "step_gate_decisions",
            "memory_version_id",
            "TEXT",
        )?;
        ensure_column(
            &self.conn,
            "step_gate_decisions",
            "source_ruleset_version",
            "INTEGER",
        )?;
        ensure_column(&self.conn, "step_gate_decisions", "evidence_json", "TEXT")?;

        let now = rfc3339(now_utc())?;
        self.conn
            .execute(
                "INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                params![TRACE_SCHEMA_VERSION, now],
            )
            .context("failed to record trace migration")?;

        Ok(())
    }

    fn upsert_workflow_snapshot(
        &self,
        workflow_hash: &str,
        normalization_version: u32,
        source_format: &str,
        source_yaml_hash: &str,
        normalized_json: &serde_json::Value,
    ) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO workflow_snapshots(
                    workflow_hash, normalization_version, source_format,
                    source_yaml_hash, normalized_json, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(workflow_hash) DO UPDATE SET
                    normalization_version = excluded.normalization_version,
                    source_format = excluded.source_format,
                    source_yaml_hash = excluded.source_yaml_hash,
                    normalized_json = excluded.normalized_json",
                params![
                    workflow_hash,
                    i64::from(normalization_version),
                    source_format,
                    source_yaml_hash,
                    serde_json::to_string(normalized_json)?,
                    rfc3339(now_utc())?,
                ],
            )
            .context("failed to upsert workflow snapshot")?;
        Ok(())
    }

    fn insert_run(&self, run: &RunRecord) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO runs(
                    run_id, workflow_name, workflow_version, workflow_hash,
                    as_of, as_of_was_default, started_at, ended_at, status,
                    replay_of_run_id, external_correlation_id, engine_version, cli_args_json,
                    manifest_hash, manifest_signature, manifest_signature_status
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                params![
                    run.run_id.to_string(),
                    run.workflow_name,
                    run.workflow_version,
                    run.workflow_hash,
                    rfc3339(run.as_of)?,
                    bool_to_sql(run.as_of_was_default),
                    rfc3339(run.started_at)?,
                    run.ended_at.map(rfc3339).transpose()?,
                    run_status_to_str(&run.status),
                    run.replay_of_run_id.map(|id| id.to_string()),
                    run.external_correlation_id,
                    run.engine_version,
                    serde_json::to_string(&run.cli_args_json)?,
                    run.manifest_hash,
                    run.manifest_signature,
                    run.manifest_signature_status,
                ],
            )
            .context("failed to insert run")?;
        Ok(())
    }

    fn update_run_finished(&self, run_id: RunId, status: RunStatus) -> Result<()> {
        self.conn
            .execute(
                "UPDATE runs SET status = ?2, ended_at = ?3 WHERE run_id = ?1",
                params![
                    run_id.to_string(),
                    run_status_to_str(&status),
                    rfc3339(now_utc())?
                ],
            )
            .context("failed to update run status")?;
        Ok(())
    }

    fn update_run_manifest(
        &self,
        run_id: RunId,
        manifest_hash: &str,
        manifest_signature: Option<&str>,
        manifest_signature_status: &str,
    ) -> Result<()> {
        self.conn
            .execute(
                "UPDATE runs SET
                    manifest_hash = ?2,
                    manifest_signature = ?3,
                    manifest_signature_status = ?4
                 WHERE run_id = ?1",
                params![
                    run_id.to_string(),
                    manifest_hash,
                    manifest_signature,
                    manifest_signature_status,
                ],
            )
            .context("failed to update run manifest")?;
        Ok(())
    }

    fn insert_step(&self, step: &StepRecord) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO steps(
                    step_id, run_id, step_index, step_key, agent_name,
                    status, started_at, ended_at, task_payload_json,
                    constraints_json, permissions_json, input_hash, output_hash, error_json
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    step.step_id.to_string(),
                    step.run_id.to_string(),
                    i64::try_from(step.step_index)
                        .map_err(|_| anyhow!("step_index too large for sqlite"))?,
                    step.step_key,
                    step.agent_name,
                    step_status_to_str(&step.status),
                    step.started_at.map(rfc3339).transpose()?,
                    step.ended_at.map(rfc3339).transpose()?,
                    serde_json::to_string(&step.task_payload_json)?,
                    serde_json::to_string(&step.constraints_json)?,
                    serde_json::to_string(&step.permissions_json)?,
                    step.input_hash,
                    step.output_hash,
                    step.error_json
                        .as_ref()
                        .map(serde_json::to_string)
                        .transpose()?,
                ],
            )
            .context("failed to insert step")?;
        Ok(())
    }

    fn update_step_status(
        &self,
        step_id: StepId,
        status: StepStatus,
        output_hash: Option<&str>,
        error_json: Option<&serde_json::Value>,
    ) -> Result<()> {
        self.conn
            .execute(
                "UPDATE steps SET status = ?2, ended_at = ?3, output_hash = ?4, error_json = ?5 WHERE step_id = ?1",
                params![
                    step_id.to_string(),
                    step_status_to_str(&status),
                    rfc3339(now_utc())?,
                    output_hash,
                    error_json.map(serde_json::to_string).transpose()?,
                ],
            )
            .context("failed to update step status")?;
        Ok(())
    }

    fn append_event(&self, event: &TraceEvent) -> Result<i64> {
        self.conn
            .execute(
                "INSERT INTO trace_events(
                    event_id, run_id, step_id, event_type,
                    occurred_at, recorded_at, actor_type, actor_id,
                    payload_json, payload_hash, prev_event_hash, event_hash
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    event.event_id.to_string(),
                    event.run_id.to_string(),
                    event.step_id.map(|id| id.to_string()),
                    event_type_to_str(&event.event_type),
                    rfc3339(event.occurred_at)?,
                    rfc3339(event.recorded_at)?,
                    event.actor_type,
                    event.actor_id,
                    serde_json::to_string(&event.payload_json)?,
                    event.payload_hash,
                    event.prev_event_hash,
                    event.event_hash,
                ],
            )
            .context("failed to append trace event")?;

        Ok(self.conn.last_insert_rowid())
    }

    fn append_context_package(
        &self,
        run_id: RunId,
        step_id: StepId,
        envelope: &ContextPackageEnvelope,
    ) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO step_context_packages(
                    run_id, step_id, package_slot, context_package_id, generated_at,
                    query_json, determinism_json, answer_json, ordering_trace_json,
                    package_json, package_hash
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    run_id.to_string(),
                    step_id.to_string(),
                    i64::try_from(envelope.package_slot)
                        .map_err(|_| anyhow!("package_slot too large"))?,
                    envelope.context_package.context_package_id,
                    rfc3339(envelope.context_package.generated_at)?,
                    serde_json::to_string(&envelope.context_package.query)?,
                    serde_json::to_string(&envelope.context_package.determinism)?,
                    serde_json::to_string(&envelope.context_package.answer)?,
                    serde_json::to_string(&envelope.context_package.ordering_trace)?,
                    serde_json::to_string(&envelope.context_package)?,
                    envelope.package_hash,
                ],
            )
            .context("failed to insert step_context_packages row")?;

        let package_row_id = self.conn.last_insert_rowid();

        for item in &envelope.context_package.selected_items {
            self.conn
                .execute(
                    "INSERT INTO step_context_selected(
                        step_context_package_id, rank, memory_version_id, memory_id,
                        version, record_type, truth_status, confidence, authority,
                        why_reasons_json, rule_scores_json, injected,
                        permission_decision, permission_reason
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1, 'allowed', NULL)",
                    params![
                        package_row_id,
                        i64::try_from(item.rank).map_err(|_| anyhow!("rank overflow"))?,
                        item.memory_version_id.to_string(),
                        item.memory_id.to_string(),
                        i64::from(item.version),
                        item.record_type.as_str(),
                        item.truth_status.as_str(),
                        item.confidence,
                        item.authority.as_str(),
                        serde_json::to_string(&item.why.reasons)?,
                        item.why
                            .rule_scores
                            .as_ref()
                            .map(serde_json::to_string)
                            .transpose()?,
                    ],
                )
                .context("failed to insert step_context_selected row")?;
        }

        for item in &envelope.context_package.excluded_items {
            self.conn
                .execute(
                    "INSERT INTO step_context_excluded(
                        step_context_package_id, rank, memory_version_id, memory_id,
                        version, record_type, truth_status, confidence, authority, why_reasons_json
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        package_row_id,
                        i64::try_from(item.rank).map_err(|_| anyhow!("rank overflow"))?,
                        item.memory_version_id.to_string(),
                        item.memory_id.to_string(),
                        i64::from(item.version),
                        item.record_type.as_str(),
                        item.truth_status.as_str(),
                        item.confidence,
                        item.authority.as_str(),
                        serde_json::to_string(&item.why.reasons)?,
                    ],
                )
                .context("failed to insert step_context_excluded row")?;
        }

        Ok(())
    }

    fn append_gate_decision(
        &self,
        run_id: RunId,
        step_id: StepId,
        decision: &GateDecisionRecord,
    ) -> Result<()> {
        if matches!(decision.gate_kind, GateKind::Trust)
            && decision.subject_type == "memory_ref"
            && (decision.memory_id.is_none()
                || decision.version.is_none()
                || decision.memory_version_id.is_none())
        {
            return Err(anyhow!(
                "trust gate decision for memory_ref requires memory_id, version, and memory_version_id"
            ));
        }

        self.conn
            .execute(
                "INSERT INTO step_gate_decisions(
                    run_id, step_id, gate_kind, gate_name, subject_type,
                    memory_id, version, memory_version_id,
                    decision, reason_codes_json, notes, decided_by, decided_at,
                    source_ruleset_version, evidence_json
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    run_id.to_string(),
                    step_id.to_string(),
                    gate_kind_to_str(&decision.gate_kind),
                    decision.gate_name,
                    decision.subject_type,
                    decision.memory_id.map(|value| value.to_string()),
                    decision.version.map(i64::from),
                    decision.memory_version_id.map(|value| value.to_string()),
                    gate_decision_to_str(&decision.decision),
                    serde_json::to_string(&decision.reason_codes)?,
                    decision.notes,
                    decision.decided_by,
                    rfc3339(decision.decided_at)?,
                    decision.source_ruleset_version.map(i64::from),
                    decision
                        .evidence_json
                        .as_ref()
                        .map(serde_json::to_string)
                        .transpose()?,
                ],
            )
            .context("failed to insert step_gate_decisions row")?;
        Ok(())
    }

    fn append_provider_call(
        &self,
        run_id: RunId,
        step_id: StepId,
        call: &multi_agent_center_domain::ProviderCallRecord,
    ) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO provider_calls(
                    provider_call_id, run_id, step_id, provider_name,
                    adapter_version, model_id, request_json, request_hash,
                    response_json, response_hash, latency_ms,
                    input_tokens, output_tokens, started_at, ended_at,
                    status, error_text
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                params![
                    call.provider_call_id.to_string(),
                    run_id.to_string(),
                    step_id.to_string(),
                    call.provider_name,
                    call.adapter_version,
                    call.model_id,
                    serde_json::to_string(&call.request_json)?,
                    call.request_hash,
                    serde_json::to_string(&call.response_json)?,
                    call.response_hash,
                    call.latency_ms
                        .map(i64::try_from)
                        .transpose()
                        .map_err(|_| anyhow!("latency overflow"))?,
                    call.input_tokens.map(i64::from),
                    call.output_tokens.map(i64::from),
                    rfc3339(call.started_at)?,
                    rfc3339(call.ended_at)?,
                    call.status,
                    call.error_text,
                ],
            )
            .context("failed to insert provider_call row")?;
        Ok(())
    }

    fn append_proposed_memory_write(
        &self,
        run_id: RunId,
        step_id: StepId,
        write: &ProposedMemoryWrite,
        disposition: &str,
        disposition_reason: Option<&str>,
    ) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO proposed_memory_writes(
                    run_id, step_id, proposal_index, proposal_json,
                    proposal_hash, disposition, disposition_reason
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    run_id.to_string(),
                    step_id.to_string(),
                    i64::try_from(write.proposal_index)
                        .map_err(|_| anyhow!("proposal_index overflow"))?,
                    serde_json::to_string(&write.payload)?,
                    multi_agent_center_domain::hash_json(&write.payload)?,
                    disposition,
                    disposition_reason,
                ],
            )
            .context("failed to insert proposed_memory_writes row")?;
        Ok(())
    }

    fn list_runs(&self) -> Result<Vec<RunRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                run_id, workflow_name, workflow_version, workflow_hash,
                as_of, as_of_was_default, started_at, ended_at,
                status, replay_of_run_id, external_correlation_id,
                engine_version, cli_args_json,
                manifest_hash, manifest_signature, manifest_signature_status
             FROM runs
             ORDER BY started_at DESC, run_id ASC",
        )?;

        let mut rows = stmt.query([])?;
        let mut out = Vec::new();

        while let Some(row) = rows.next()? {
            let run_id_str: String = row.get(0)?;
            let replay_of_run_id_str: Option<String> = row.get(9)?;
            let cli_args_json: String = row.get(12)?;
            out.push(RunRecord {
                run_id: parse_run_id(&run_id_str)?,
                workflow_name: row.get(1)?,
                workflow_version: row.get(2)?,
                workflow_hash: row.get(3)?,
                as_of: parse_rfc3339(&row.get::<_, String>(4)?)?,
                as_of_was_default: sql_to_bool(row.get::<_, i64>(5)?),
                started_at: parse_rfc3339(&row.get::<_, String>(6)?)?,
                ended_at: row
                    .get::<_, Option<String>>(7)?
                    .map(|v| parse_rfc3339(&v))
                    .transpose()?,
                status: parse_run_status(&row.get::<_, String>(8)?)?,
                replay_of_run_id: replay_of_run_id_str
                    .map(|value| parse_run_id(&value))
                    .transpose()?,
                external_correlation_id: row.get(10)?,
                engine_version: row.get(11)?,
                cli_args_json: serde_json::from_str(&cli_args_json)
                    .context("invalid cli_args_json")?,
                manifest_hash: row.get(13)?,
                manifest_signature: row.get(14)?,
                manifest_signature_status: row.get(15)?,
            });
        }

        Ok(out)
    }

    fn list_events_for_run(&self, run_id: RunId) -> Result<Vec<EventRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                event_seq, event_id, run_id, step_id, event_type,
                occurred_at, recorded_at, actor_type, actor_id,
                payload_json, payload_hash, prev_event_hash, event_hash
             FROM trace_events
             WHERE run_id = ?1
             ORDER BY event_seq ASC",
        )?;

        let mut rows = stmt.query(params![run_id.to_string()])?;
        let mut out = Vec::new();

        while let Some(row) = rows.next()? {
            let event_id_raw: String = row.get(1)?;
            let run_id_raw: String = row.get(2)?;
            let step_id_raw: Option<String> = row.get(3)?;
            let payload_raw: String = row.get(9)?;
            out.push(EventRow {
                event_seq: row.get(0)?,
                event: TraceEvent {
                    event_id: Ulid::from_str(&event_id_raw)
                        .map_err(|err| anyhow!("invalid event_id ULID: {err}"))?,
                    run_id: parse_run_id(&run_id_raw)?,
                    step_id: step_id_raw.map(|value| parse_step_id(&value)).transpose()?,
                    event_type: parse_event_type(&row.get::<_, String>(4)?)?,
                    occurred_at: parse_rfc3339(&row.get::<_, String>(5)?)?,
                    recorded_at: parse_rfc3339(&row.get::<_, String>(6)?)?,
                    actor_type: row.get(7)?,
                    actor_id: row.get(8)?,
                    payload_json: serde_json::from_str(&payload_raw)
                        .context("invalid payload_json")?,
                    payload_hash: row.get(10)?,
                    prev_event_hash: row.get(11)?,
                    event_hash: row.get(12)?,
                },
            });
        }

        Ok(out)
    }

    fn get_run(&self, run_id: RunId) -> Result<Option<RunRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                run_id, workflow_name, workflow_version, workflow_hash,
                as_of, as_of_was_default, started_at, ended_at,
                status, replay_of_run_id, external_correlation_id,
                engine_version, cli_args_json,
                manifest_hash, manifest_signature, manifest_signature_status
             FROM runs WHERE run_id = ?1",
        )?;

        stmt.query_row(params![run_id.to_string()], |row| {
            let run_id_str: String = row.get(0)?;
            let replay_of_run_id_str: Option<String> = row.get(9)?;
            let cli_args_json: String = row.get(12)?;
            Ok((
                run_id_str,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
                replay_of_run_id_str,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, String>(11)?,
                cli_args_json,
                row.get::<_, Option<String>>(13)?,
                row.get::<_, Option<String>>(14)?,
                row.get::<_, String>(15)?,
            ))
        })
        .optional()?
        .map(
            |(
                run_id_raw,
                workflow_name,
                workflow_version,
                workflow_hash,
                as_of,
                as_of_was_default,
                started_at,
                ended_at,
                status,
                replay_of_run_id,
                external_correlation_id,
                engine_version,
                cli_args_json,
                manifest_hash,
                manifest_signature,
                manifest_signature_status,
            )| {
                Ok(RunRecord {
                    run_id: parse_run_id(&run_id_raw)?,
                    workflow_name,
                    workflow_version,
                    workflow_hash,
                    as_of: parse_rfc3339(&as_of)?,
                    as_of_was_default: sql_to_bool(as_of_was_default),
                    started_at: parse_rfc3339(&started_at)?,
                    ended_at: ended_at.map(|value| parse_rfc3339(&value)).transpose()?,
                    status: parse_run_status(&status)?,
                    replay_of_run_id: replay_of_run_id
                        .map(|value| parse_run_id(&value))
                        .transpose()?,
                    external_correlation_id,
                    engine_version,
                    cli_args_json: serde_json::from_str(&cli_args_json)
                        .context("invalid cli_args_json")?,
                    manifest_hash,
                    manifest_signature,
                    manifest_signature_status,
                })
            },
        )
        .transpose()
    }

    fn get_step_records(&self, run_id: RunId) -> Result<Vec<StepRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                step_id, run_id, step_index, step_key, agent_name,
                status, started_at, ended_at, task_payload_json,
                constraints_json, permissions_json, input_hash,
                output_hash, error_json
             FROM steps
             WHERE run_id = ?1
             ORDER BY step_index ASC",
        )?;

        let mut rows = stmt.query(params![run_id.to_string()])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let step_id_str: String = row.get(0)?;
            let run_id_str: String = row.get(1)?;
            let task_payload_json: String = row.get(8)?;
            let constraints_json: String = row.get(9)?;
            let permissions_json: String = row.get(10)?;
            let error_json: Option<String> = row.get(13)?;
            let step_index: i64 = row.get(2)?;
            out.push(StepRecord {
                step_id: parse_step_id(&step_id_str)?,
                run_id: parse_run_id(&run_id_str)?,
                step_index: usize::try_from(step_index)
                    .map_err(|_| anyhow!("invalid step_index: {step_index}"))?,
                step_key: row.get(3)?,
                agent_name: row.get(4)?,
                status: parse_step_status(&row.get::<_, String>(5)?)?,
                started_at: row
                    .get::<_, Option<String>>(6)?
                    .map(|v| parse_rfc3339(&v))
                    .transpose()?,
                ended_at: row
                    .get::<_, Option<String>>(7)?
                    .map(|v| parse_rfc3339(&v))
                    .transpose()?,
                task_payload_json: serde_json::from_str(&task_payload_json)
                    .context("invalid task_payload_json")?,
                constraints_json: serde_json::from_str(&constraints_json)
                    .context("invalid constraints_json")?,
                permissions_json: serde_json::from_str(&permissions_json)
                    .context("invalid permissions_json")?,
                input_hash: row.get(11)?,
                output_hash: row.get(12)?,
                error_json: error_json
                    .map(|value| serde_json::from_str(&value).context("invalid error_json"))
                    .transpose()?,
            });
        }

        Ok(out)
    }

    fn get_workflow_snapshot(&self, workflow_hash: &str) -> Result<Option<WorkflowSnapshotRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                workflow_hash, normalization_version, source_format,
                source_yaml_hash, normalized_json
             FROM workflow_snapshots
             WHERE workflow_hash = ?1",
        )?;

        stmt.query_row(params![workflow_hash], |row| {
            let normalized_json: String = row.get(4)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                normalized_json,
            ))
        })
        .optional()?
        .map(
            |(
                workflow_hash_value,
                normalization_version_raw,
                source_format,
                source_yaml_hash,
                normalized_json,
            )| {
                let normalization_version =
                    u32::try_from(normalization_version_raw).map_err(|_| {
                        anyhow!("invalid normalization_version: {normalization_version_raw}")
                    })?;
                Ok(WorkflowSnapshotRecord {
                    workflow_hash: workflow_hash_value,
                    normalization_version,
                    source_format,
                    source_yaml_hash,
                    normalized_json: serde_json::from_str(&normalized_json)
                        .context("invalid workflow_snapshots.normalized_json")?,
                })
            },
        )
        .transpose()
    }

    fn get_step_context_packages(&self, run_id: RunId) -> Result<Vec<StepContextPackageRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                s.step_key,
                scp.package_slot,
                scp.package_json,
                scp.package_hash
             FROM step_context_packages scp
             INNER JOIN steps s ON s.step_id = scp.step_id
             WHERE scp.run_id = ?1
             ORDER BY s.step_index ASC, scp.package_slot ASC",
        )?;

        let mut rows = stmt.query(params![run_id.to_string()])?;
        let mut out = Vec::new();

        while let Some(row) = rows.next()? {
            let step_key: String = row.get(0)?;
            let package_slot_raw: i64 = row.get(1)?;
            let package_json: String = row.get(2)?;
            let package_hash: String = row.get(3)?;
            let context_package: ContextPackage =
                serde_json::from_str(&package_json).context("invalid step_context package_json")?;
            let package_slot =
                usize::try_from(package_slot_raw).map_err(|_| anyhow!("invalid package_slot"))?;

            out.push(StepContextPackageRecord {
                step_key,
                envelope: ContextPackageEnvelope {
                    package_slot,
                    source: "trace.snapshot".to_string(),
                    context_package,
                    package_hash,
                },
            });
        }

        Ok(out)
    }
}

fn ensure_column(conn: &Connection, table: &str, column: &str, sql_type: &str) -> Result<()> {
    if table_has_column(conn, table, column)? {
        return Ok(());
    }

    conn.execute(
        &format!("ALTER TABLE {table} ADD COLUMN {column} {sql_type}"),
        [],
    )
    .with_context(|| format!("failed to add missing column {table}.{column}"))?;
    Ok(())
}

fn table_has_column(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .with_context(|| format!("failed to inspect table info for {table}"))?;

    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn parse_run_status(value: &str) -> Result<RunStatus> {
    match value {
        "pending" => Ok(RunStatus::Pending),
        "running" => Ok(RunStatus::Running),
        "succeeded" => Ok(RunStatus::Succeeded),
        "failed" => Ok(RunStatus::Failed),
        "rejected" => Ok(RunStatus::Rejected),
        _ => Err(anyhow!("unknown run status: {value}")),
    }
}

fn run_status_to_str(status: &RunStatus) -> &'static str {
    match status {
        RunStatus::Pending => "pending",
        RunStatus::Running => "running",
        RunStatus::Succeeded => "succeeded",
        RunStatus::Failed => "failed",
        RunStatus::Rejected => "rejected",
    }
}

fn parse_step_status(value: &str) -> Result<StepStatus> {
    match value {
        "pending" => Ok(StepStatus::Pending),
        "running" => Ok(StepStatus::Running),
        "succeeded" => Ok(StepStatus::Succeeded),
        "failed" => Ok(StepStatus::Failed),
        "rejected" => Ok(StepStatus::Rejected),
        "skipped" => Ok(StepStatus::Skipped),
        _ => Err(anyhow!("unknown step status: {value}")),
    }
}

fn step_status_to_str(status: &StepStatus) -> &'static str {
    match status {
        StepStatus::Pending => "pending",
        StepStatus::Running => "running",
        StepStatus::Succeeded => "succeeded",
        StepStatus::Failed => "failed",
        StepStatus::Rejected => "rejected",
        StepStatus::Skipped => "skipped",
    }
}

fn parse_event_type(value: &str) -> Result<TraceEventType> {
    match value {
        "run_started" => Ok(TraceEventType::RunStarted),
        "run_finished" => Ok(TraceEventType::RunFinished),
        "workflow_normalized" => Ok(TraceEventType::WorkflowNormalized),
        "step_ready" => Ok(TraceEventType::StepReady),
        "step_started" => Ok(TraceEventType::StepStarted),
        "step_input_prepared" => Ok(TraceEventType::StepInputPrepared),
        "step_permission_pruned" => Ok(TraceEventType::StepPermissionPruned),
        "gate_evaluated" => Ok(TraceEventType::GateEvaluated),
        "provider_called" => Ok(TraceEventType::ProviderCalled),
        "step_finished" => Ok(TraceEventType::StepFinished),
        "proposed_memory_write" => Ok(TraceEventType::ProposedMemoryWrite),
        "replay_started" => Ok(TraceEventType::ReplayStarted),
        "replay_finished" => Ok(TraceEventType::ReplayFinished),
        "warning" => Ok(TraceEventType::Warning),
        "error" => Ok(TraceEventType::Error),
        _ => Err(anyhow!("unknown event_type: {value}")),
    }
}

fn event_type_to_str(value: &TraceEventType) -> &'static str {
    match value {
        TraceEventType::RunStarted => "run_started",
        TraceEventType::RunFinished => "run_finished",
        TraceEventType::WorkflowNormalized => "workflow_normalized",
        TraceEventType::StepReady => "step_ready",
        TraceEventType::StepStarted => "step_started",
        TraceEventType::StepInputPrepared => "step_input_prepared",
        TraceEventType::StepPermissionPruned => "step_permission_pruned",
        TraceEventType::GateEvaluated => "gate_evaluated",
        TraceEventType::ProviderCalled => "provider_called",
        TraceEventType::StepFinished => "step_finished",
        TraceEventType::ProposedMemoryWrite => "proposed_memory_write",
        TraceEventType::ReplayStarted => "replay_started",
        TraceEventType::ReplayFinished => "replay_finished",
        TraceEventType::Warning => "warning",
        TraceEventType::Error => "error",
    }
}

fn gate_kind_to_str(value: &GateKind) -> &'static str {
    match value {
        GateKind::Human => "human",
        GateKind::Trust => "trust",
        GateKind::Policy => "policy",
    }
}

fn gate_decision_to_str(value: &GateDecision) -> &'static str {
    match value {
        GateDecision::Approved => "approved",
        GateDecision::Rejected => "rejected",
        GateDecision::Pruned => "pruned",
    }
}

fn parse_run_id(value: &str) -> Result<RunId> {
    let ulid = Ulid::from_str(value).map_err(|err| anyhow!("invalid run_id ULID: {err}"))?;
    Ok(RunId(ulid))
}

fn parse_step_id(value: &str) -> Result<StepId> {
    let ulid = Ulid::from_str(value).map_err(|err| anyhow!("invalid step_id ULID: {err}"))?;
    Ok(StepId(ulid))
}

fn bool_to_sql(value: bool) -> i64 {
    i64::from(value)
}

fn sql_to_bool(value: i64) -> bool {
    value != 0
}

fn rfc3339(value: OffsetDateTime) -> Result<String> {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|err| anyhow!("invalid datetime format: {err}"))
}

fn parse_rfc3339(value: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
        .map_err(|err| anyhow!("invalid RFC3339 datetime: {err}"))
}

#[cfg(test)]
mod tests {
    use super::SqliteTraceStore;
    use memory_kernel_core::{
        Answer, AnswerResult, Authority, ContextItem, ContextPackage, DeterminismMetadata,
        MemoryId, MemoryVersionId, QueryRequest, RecordType, TruthStatus, Why,
    };
    use multi_agent_center_domain::{
        ContextPackageEnvelope, GateDecision, GateDecisionRecord, GateKind, RunId, RunRecord,
        RunStatus, StepId, StepRecord, StepStatus, TraceEvent, TraceEventType,
    };
    use multi_agent_center_trace_core::TraceStore;
    use rusqlite::{params, Connection};
    use serde_json::json;
    use ulid::Ulid;

    fn temp_db_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "mac-trace-sqlite-test-{}-{}.sqlite",
            name,
            Ulid::new()
        ))
    }

    fn fixture_run(run_id: RunId) -> RunRecord {
        let now = time::OffsetDateTime::now_utc();
        RunRecord {
            run_id,
            workflow_name: "wf".to_string(),
            workflow_version: "v1".to_string(),
            workflow_hash: "hash".to_string(),
            as_of: now,
            as_of_was_default: true,
            started_at: now,
            ended_at: None,
            status: RunStatus::Running,
            replay_of_run_id: None,
            external_correlation_id: None,
            engine_version: "test".to_string(),
            cli_args_json: json!({}),
            manifest_hash: None,
            manifest_signature: None,
            manifest_signature_status: "unsigned".to_string(),
        }
    }

    fn fixture_step(run_id: RunId, step_id: StepId) -> StepRecord {
        StepRecord {
            step_id,
            run_id,
            step_index: 0,
            step_key: "step".to_string(),
            agent_name: "agent".to_string(),
            status: StepStatus::Running,
            started_at: Some(time::OffsetDateTime::now_utc()),
            ended_at: None,
            task_payload_json: json!({}),
            constraints_json: json!({}),
            permissions_json: json!({}),
            input_hash: "input-hash".to_string(),
            output_hash: None,
            error_json: None,
        }
    }

    fn fixture_package() -> ContextPackageEnvelope {
        let now = time::OffsetDateTime::now_utc();
        let selected = ContextItem {
            rank: 1,
            memory_version_id: MemoryVersionId::new(),
            memory_id: MemoryId::new(),
            record_type: RecordType::Constraint,
            version: 1,
            truth_status: TruthStatus::Asserted,
            confidence: Some(0.9),
            authority: Authority::Authoritative,
            why: Why {
                included: true,
                reasons: vec!["fixture".to_string()],
                rule_scores: None,
            },
        };
        let package = ContextPackage {
            context_package_id: "pkg".to_string(),
            generated_at: now,
            query: QueryRequest {
                text: "t".to_string(),
                actor: "a".to_string(),
                action: "act".to_string(),
                resource: "res".to_string(),
                as_of: now,
            },
            determinism: DeterminismMetadata {
                ruleset_version: "mk.v1".to_string(),
                snapshot_id: "snap".to_string(),
                tie_breakers: vec!["x".to_string()],
            },
            answer: Answer {
                result: AnswerResult::Allow,
                why: "fixture".to_string(),
            },
            selected_items: vec![selected],
            excluded_items: Vec::new(),
            ordering_trace: vec!["fixture".to_string()],
        };
        ContextPackageEnvelope {
            package_slot: 0,
            source: "test".to_string(),
            package_hash: "hash".to_string(),
            context_package: package,
        }
    }

    #[test]
    fn migrate_is_idempotent_and_manifest_columns_exist() {
        let path = temp_db_path("migrate");
        let store = SqliteTraceStore::open(&path);
        assert!(store.is_ok());
        let store = store.unwrap_or_else(|_| unreachable!());

        assert!(store.migrate().is_ok());
        assert!(store.migrate().is_ok());

        let mut stmt = store
            .conn
            .prepare("PRAGMA table_info(runs)")
            .unwrap_or_else(|_| unreachable!());
        let mut rows = stmt.query([]).unwrap_or_else(|_| unreachable!());
        let mut found_manifest_hash = false;
        let mut found_manifest_signature_status = false;
        while let Some(row) = rows.next().unwrap_or_else(|_| unreachable!()) {
            let col_name: String = row.get(1).unwrap_or_else(|_| unreachable!());
            if col_name == "manifest_hash" {
                found_manifest_hash = true;
            }
            if col_name == "manifest_signature_status" {
                found_manifest_signature_status = true;
            }
        }
        assert!(found_manifest_hash);
        assert!(found_manifest_signature_status);
    }

    #[test]
    fn trace_events_are_append_only() {
        let path = temp_db_path("append-only");
        let store = SqliteTraceStore::open(&path);
        assert!(store.is_ok());
        let store = store.unwrap_or_else(|_| unreachable!());
        assert!(store.migrate().is_ok());
        assert!(store
            .upsert_workflow_snapshot("hash", 1, "yaml", "yaml-hash", &json!({"x":1}))
            .is_ok());

        let run_id = RunId::new();
        let step_id = StepId::new();
        assert!(store.insert_run(&fixture_run(run_id)).is_ok());
        assert!(store.insert_step(&fixture_step(run_id, step_id)).is_ok());

        let now = time::OffsetDateTime::now_utc();
        let event = TraceEvent {
            event_id: Ulid::new(),
            run_id,
            step_id: Some(step_id),
            event_type: TraceEventType::StepStarted,
            occurred_at: now,
            recorded_at: now,
            actor_type: "system".to_string(),
            actor_id: "test".to_string(),
            payload_json: json!({"k":"v"}),
            payload_hash: "payload".to_string(),
            prev_event_hash: None,
            event_hash: "event".to_string(),
        };
        assert!(store.append_event(&event).is_ok());

        let mutated = store.conn.execute(
            "UPDATE trace_events SET actor_id = 'mutated' WHERE event_seq = 1",
            [],
        );
        assert!(mutated.is_err());
    }

    #[test]
    fn context_package_round_trip_and_trust_gate_persist() {
        let path = temp_db_path("round-trip");
        let store = SqliteTraceStore::open(&path);
        assert!(store.is_ok());
        let store = store.unwrap_or_else(|_| unreachable!());
        assert!(store.migrate().is_ok());
        assert!(store
            .upsert_workflow_snapshot("hash", 1, "yaml", "yaml-hash", &json!({"x":1}))
            .is_ok());

        let run_id = RunId::new();
        let step_id = StepId::new();
        assert!(store.insert_run(&fixture_run(run_id)).is_ok());
        assert!(store.insert_step(&fixture_step(run_id, step_id)).is_ok());

        let package = fixture_package();
        assert!(store
            .append_context_package(run_id, step_id, &package)
            .is_ok());
        assert!(store
            .append_gate_decision(
                run_id,
                step_id,
                &GateDecisionRecord {
                    gate_kind: GateKind::Trust,
                    gate_name: "trust".to_string(),
                    subject_type: "memory_ref".to_string(),
                    memory_id: Some(package.context_package.selected_items[0].memory_id),
                    version: Some(1),
                    memory_version_id: Some(
                        package.context_package.selected_items[0].memory_version_id
                    ),
                    decision: GateDecision::Approved,
                    reason_codes: vec!["included".to_string()],
                    notes: None,
                    decided_by: "test".to_string(),
                    decided_at: time::OffsetDateTime::now_utc(),
                    source_ruleset_version: Some(1),
                    evidence_json: Some(json!({"k":"v"})),
                }
            )
            .is_ok());

        let records = store.get_step_context_packages(run_id);
        assert!(records.is_ok());
        let records = records.unwrap_or_else(|_| unreachable!());
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].step_key, "step");
        assert_eq!(records[0].envelope.package_slot, 0);
    }

    #[test]
    fn trust_gate_memory_ref_requires_memory_version_id() {
        let path = temp_db_path("trust-memory-version-required");
        let store = SqliteTraceStore::open(&path);
        assert!(store.is_ok());
        let store = store.unwrap_or_else(|_| unreachable!());
        assert!(store.migrate().is_ok());
        assert!(store
            .upsert_workflow_snapshot("hash", 1, "yaml", "yaml-hash", &json!({"x":1}))
            .is_ok());

        let run_id = RunId::new();
        let step_id = StepId::new();
        assert!(store.insert_run(&fixture_run(run_id)).is_ok());
        assert!(store.insert_step(&fixture_step(run_id, step_id)).is_ok());

        let package = fixture_package();
        let gate_result = store.append_gate_decision(
            run_id,
            step_id,
            &GateDecisionRecord {
                gate_kind: GateKind::Trust,
                gate_name: "trust".to_string(),
                subject_type: "memory_ref".to_string(),
                memory_id: Some(package.context_package.selected_items[0].memory_id),
                version: Some(1),
                memory_version_id: None,
                decision: GateDecision::Approved,
                reason_codes: vec!["included".to_string()],
                notes: None,
                decided_by: "test".to_string(),
                decided_at: time::OffsetDateTime::now_utc(),
                source_ruleset_version: Some(1),
                evidence_json: Some(json!({"k":"v"})),
            },
        );
        assert!(gate_result.is_err());
    }

    #[test]
    fn migrate_keeps_legacy_rows_and_enforces_new_trust_memory_ref_identity() {
        let path = temp_db_path("legacy-trust-memory-ref");
        let legacy_conn = Connection::open(&path);
        assert!(legacy_conn.is_ok());
        let legacy_conn = legacy_conn.unwrap_or_else(|_| unreachable!());
        assert!(legacy_conn
            .execute_batch(
                "CREATE TABLE step_gate_decisions (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    run_id TEXT NOT NULL,
                    step_id TEXT NOT NULL,
                    gate_kind TEXT NOT NULL,
                    gate_name TEXT NOT NULL,
                    subject_type TEXT NOT NULL,
                    memory_id TEXT,
                    version INTEGER,
                    memory_version_id TEXT,
                    decision TEXT NOT NULL,
                    reason_codes_json TEXT NOT NULL,
                    notes TEXT,
                    decided_by TEXT NOT NULL,
                    decided_at TEXT NOT NULL
                );
                INSERT INTO step_gate_decisions(
                    run_id, step_id, gate_kind, gate_name, subject_type,
                    memory_id, version, memory_version_id,
                    decision, reason_codes_json, notes, decided_by, decided_at
                ) VALUES (
                    'legacy-run', 'legacy-step', 'trust', 'trust_gate', 'memory_ref',
                    NULL, NULL, NULL,
                    'rejected', '[\"legacy\"]', NULL, 'legacy', '2026-02-07T00:00:00Z'
                );"
            )
            .is_ok());
        drop(legacy_conn);

        let store = SqliteTraceStore::open(&path);
        assert!(store.is_ok());
        let store = store.unwrap_or_else(|_| unreachable!());
        assert!(store.migrate().is_ok());

        let legacy_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM step_gate_decisions
                 WHERE gate_kind = 'trust' AND subject_type = 'memory_ref' AND memory_id IS NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(legacy_count, 1);

        let insert_invalid = store.conn.execute(
            "INSERT INTO step_gate_decisions(
                run_id, step_id, gate_kind, gate_name, subject_type,
                memory_id, version, memory_version_id,
                decision, reason_codes_json, notes, decided_by, decided_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                "new-run",
                "new-step",
                "trust",
                "trust_gate",
                "memory_ref",
                Option::<String>::None,
                Option::<i64>::None,
                Option::<String>::None,
                "rejected",
                "[\"new\"]",
                Option::<String>::None,
                "new",
                "2026-02-07T00:00:00Z"
            ],
        );
        assert!(insert_invalid.is_err());
    }
}
