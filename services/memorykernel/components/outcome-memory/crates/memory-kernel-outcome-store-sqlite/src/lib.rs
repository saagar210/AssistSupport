#![allow(clippy::missing_errors_doc)]
#![allow(clippy::uninlined_format_args)]

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use memory_kernel_core::MemoryId;
use memory_kernel_outcome_core::{
    apply_as_of_decay, format_rfc3339, gate_memory, now_utc, parse_rfc3339_utc,
    project_memory_trust, GateDecision, MemoryKey, MemoryTrust, OutcomeEvent, OutcomeEventInput,
    OutcomeEventType, OutcomeRuleset, RetrievalMode, Severity, TrustStatus,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;
use ulid::Ulid;

const OUTCOME_MIGRATION_VERSION: i64 = 2;
const PROJECTOR_NAME: &str = "trust_v0";

const SCHEMA_OUTCOME_V1: &str = r"
CREATE TABLE IF NOT EXISTS outcome_rulesets (
  ruleset_version INTEGER PRIMARY KEY,
  ruleset_json TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS outcome_events (
  event_seq INTEGER PRIMARY KEY AUTOINCREMENT,
  event_id TEXT NOT NULL UNIQUE,
  ruleset_version INTEGER NOT NULL,
  memory_id TEXT NOT NULL,
  version INTEGER NOT NULL CHECK (version >= 1),
  event_type TEXT NOT NULL CHECK (
    event_type IN (
      'inherited',
      'success',
      'failure',
      'ignored',
      'unknown',
      'manual_set_confidence',
      'manual_promote',
      'manual_retire',
      'authoritative_contradiction'
    )
  ),
  occurred_at TEXT NOT NULL,
  recorded_at TEXT NOT NULL,
  writer TEXT NOT NULL,
  justification TEXT NOT NULL,
  context_id TEXT,
  edited INTEGER NOT NULL DEFAULT 0 CHECK (edited IN (0, 1)),
  escalated INTEGER NOT NULL DEFAULT 0 CHECK (escalated IN (0, 1)),
  severity TEXT CHECK (severity IN ('low','med','high') OR severity IS NULL),
  manual_confidence REAL CHECK (manual_confidence BETWEEN 0.0 AND 1.0 OR manual_confidence IS NULL),
  override_cap INTEGER NOT NULL DEFAULT 0 CHECK (override_cap IN (0, 1)),
  payload_json TEXT NOT NULL DEFAULT '{}',
  FOREIGN KEY (ruleset_version) REFERENCES outcome_rulesets(ruleset_version),
  FOREIGN KEY (memory_id, version) REFERENCES memory_records(memory_id, version)
);

CREATE TRIGGER IF NOT EXISTS trg_outcome_events_no_update
BEFORE UPDATE ON outcome_events
BEGIN
  SELECT RAISE(FAIL, 'outcome_events is append-only');
END;

CREATE TRIGGER IF NOT EXISTS trg_outcome_events_no_delete
BEFORE DELETE ON outcome_events
BEGIN
  SELECT RAISE(FAIL, 'outcome_events is append-only');
END;

CREATE INDEX IF NOT EXISTS idx_outcome_events_memory_version_seq
  ON outcome_events(memory_id, version, event_seq);
CREATE INDEX IF NOT EXISTS idx_outcome_events_type_seq
  ON outcome_events(event_type, event_seq);
CREATE INDEX IF NOT EXISTS idx_outcome_events_context_seq
  ON outcome_events(context_id, event_seq);

CREATE TABLE IF NOT EXISTS memory_trust (
  memory_id TEXT NOT NULL,
  version INTEGER NOT NULL CHECK (version >= 1),
  confidence_raw REAL NOT NULL CHECK (confidence_raw BETWEEN 0.0 AND 1.0),
  confidence_effective REAL NOT NULL CHECK (confidence_effective BETWEEN 0.0 AND 1.0),
  baseline_confidence REAL NOT NULL CHECK (baseline_confidence BETWEEN 0.0 AND 1.0),
  trust_status TEXT NOT NULL CHECK (trust_status IN ('active', 'validated', 'retired')),
  contradiction_cap_active INTEGER NOT NULL CHECK (contradiction_cap_active IN (0, 1)),
  cap_value REAL NOT NULL CHECK (cap_value BETWEEN 0.0 AND 1.0),
  manual_override_active INTEGER NOT NULL CHECK (manual_override_active IN (0, 1)),
  wins_last5 INTEGER NOT NULL CHECK (wins_last5 BETWEEN 0 AND 5),
  failures_last5 INTEGER NOT NULL CHECK (failures_last5 BETWEEN 0 AND 5),
  last_event_seq INTEGER NOT NULL,
  last_ruleset_version INTEGER NOT NULL,
  last_scored_at TEXT,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (memory_id, version)
);

CREATE INDEX IF NOT EXISTS idx_memory_trust_gate
  ON memory_trust(trust_status, confidence_effective DESC, contradiction_cap_active);

CREATE TABLE IF NOT EXISTS outcome_projection_state (
  projector_name TEXT PRIMARY KEY,
  ruleset_version INTEGER NOT NULL,
  last_event_seq INTEGER NOT NULL,
  updated_at TEXT NOT NULL
);
";

pub struct SqliteOutcomeStore {
    conn: Connection,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ReplayReport {
    pub projected_keys: usize,
    pub processed_events: usize,
    pub last_event_seq: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ProjectorStatus {
    pub contract_version: String,
    pub projector_name: String,
    pub ruleset_version: u32,
    pub projected_event_seq: i64,
    pub latest_event_seq: i64,
    pub lag_events: i64,
    pub lag_delta_events: i64,
    pub tracked_keys: usize,
    pub trust_rows: usize,
    pub stale_trust_rows: usize,
    pub keys_with_events_no_trust_row: usize,
    pub trust_rows_without_events: usize,
    pub max_stale_seq_gap: i64,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectorIssueSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ProjectorIssue {
    pub code: String,
    pub severity: ProjectorIssueSeverity,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ProjectorCheck {
    pub contract_version: String,
    pub healthy: bool,
    pub status: ProjectorStatus,
    pub issues: Vec<ProjectorIssue>,
    pub stale_key_sample: Vec<ProjectorStaleKey>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ProjectorStaleKey {
    pub memory_id: MemoryId,
    pub version: u32,
    pub max_event_seq: i64,
    pub projected_event_seq: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BenchmarkThresholds {
    pub append_p95_ms_max: f64,
    pub replay_p95_ms_max: f64,
    pub gate_p95_ms_max: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct BenchmarkConfig {
    pub volumes: Vec<usize>,
    pub repetitions: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BenchmarkVolumeResult {
    pub event_count: usize,
    pub append_p50_ms: f64,
    pub append_p95_ms: f64,
    pub replay_p50_ms: f64,
    pub replay_p95_ms: f64,
    pub gate_p50_ms: f64,
    pub gate_p95_ms: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BenchmarkReport {
    pub contract_version: String,
    pub generated_at: String,
    pub repetitions: usize,
    pub volumes: Vec<BenchmarkVolumeResult>,
    pub thresholds: Option<BenchmarkThresholds>,
    pub within_thresholds: bool,
    pub violations: Vec<String>,
}

impl SqliteOutcomeStore {
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

    pub fn migrate(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS schema_migrations (
                    version INTEGER PRIMARY KEY,
                    applied_at TEXT NOT NULL
                );",
            )
            .context("failed to ensure schema_migrations exists")?;

        ensure_memory_kernel_compatibility(&self.conn)?;

        self.conn
            .execute_batch(SCHEMA_OUTCOME_V1)
            .context("failed to apply outcome schema")?;

        let now = format_rfc3339(now_utc()).map_err(|err| anyhow!(err.to_string()))?;
        self.conn
            .execute(
                "INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                params![OUTCOME_MIGRATION_VERSION, now],
            )
            .context("failed to register outcome schema migration")?;

        self.upsert_ruleset(&OutcomeRuleset::v1())?;

        let now = format_rfc3339(now_utc()).map_err(|err| anyhow!(err.to_string()))?;
        self.conn
            .execute(
                "INSERT OR IGNORE INTO outcome_projection_state(projector_name, ruleset_version, last_event_seq, updated_at)
                 VALUES (?1, ?2, 0, ?3)",
                params![PROJECTOR_NAME, 1_i64, now],
            )
            .context("failed to initialize projection state")?;

        Ok(())
    }

    pub fn upsert_ruleset(&self, ruleset: &OutcomeRuleset) -> Result<()> {
        ruleset
            .validate()
            .map_err(|err| anyhow!("invalid ruleset configuration: {err}"))?;

        let payload = serde_json::to_string(ruleset).context("failed to serialize ruleset")?;
        let now = format_rfc3339(now_utc()).map_err(|err| anyhow!(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO outcome_rulesets(ruleset_version, ruleset_json, created_at)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(ruleset_version) DO UPDATE SET
                   ruleset_json = excluded.ruleset_json,
                   created_at = excluded.created_at",
                params![i64::from(ruleset.ruleset_version), payload, now],
            )
            .context("failed to upsert ruleset")?;

        Ok(())
    }

    pub fn get_rulesets(&self) -> Result<BTreeMap<u32, OutcomeRuleset>> {
        let mut stmt = self
            .conn
            .prepare("SELECT ruleset_version, ruleset_json FROM outcome_rulesets ORDER BY ruleset_version ASC")?;

        let mut rows = stmt.query([])?;
        let mut map = BTreeMap::new();

        while let Some(row) = rows.next()? {
            let version_i64: i64 = row.get(0)?;
            let version = u32::try_from(version_i64)
                .with_context(|| format!("invalid ruleset_version: {version_i64}"))?;
            let json: String = row.get(1)?;
            let value: Value =
                serde_json::from_str(&json).context("invalid stored ruleset JSON")?;
            let ruleset = OutcomeRuleset::from_json(&value)
                .map_err(|err| anyhow!("failed to parse ruleset {version}: {err}"))?;
            map.insert(version, ruleset);
        }

        Ok(map)
    }

    pub fn append_event(&mut self, input: &OutcomeEventInput) -> Result<OutcomeEvent> {
        input
            .validate()
            .map_err(|err| anyhow!("event validation failed: {err}"))?;

        let rulesets = self.get_rulesets()?;
        if !rulesets.contains_key(&input.ruleset_version) {
            return Err(anyhow!(
                "missing ruleset_version {} in outcome_rulesets",
                input.ruleset_version
            ));
        }

        let event_id = match input.event_id {
            Some(value) => value,
            None => Ulid::new(),
        };
        let recorded_at = now_utc();

        let tx = self
            .conn
            .transaction()
            .context("failed to start event transaction")?;

        tx.execute(
            "INSERT INTO outcome_events(
                event_id, ruleset_version, memory_id, version, event_type,
                occurred_at, recorded_at, writer, justification,
                context_id, edited, escalated, severity,
                manual_confidence, override_cap, payload_json
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7, ?8, ?9,
                ?10, ?11, ?12, ?13,
                ?14, ?15, ?16
             )",
            params![
                event_id.to_string(),
                i64::from(input.ruleset_version),
                input.memory_id.to_string(),
                i64::from(input.version),
                input.event_type.as_str(),
                format_rfc3339(input.occurred_at).map_err(|err| anyhow!(err.to_string()))?,
                format_rfc3339(recorded_at).map_err(|err| anyhow!(err.to_string()))?,
                input.writer,
                input.justification,
                input.context_id,
                bool_to_sql(input.edited),
                bool_to_sql(input.escalated),
                input.severity.map(Severity::as_str),
                input.manual_confidence,
                bool_to_sql(input.override_cap),
                serde_json::to_string(&input.payload_json)
                    .context("failed to serialize payload_json")?,
            ],
        )
        .context("failed to append outcome event")?;

        let event_seq = tx.last_insert_rowid();
        tx.commit().context("failed to commit event transaction")?;

        Ok(OutcomeEvent {
            event_seq,
            event_id,
            ruleset_version: input.ruleset_version,
            memory_id: input.memory_id,
            version: input.version,
            event_type: input.event_type,
            occurred_at: input.occurred_at,
            recorded_at,
            writer: input.writer.clone(),
            justification: input.justification.clone(),
            context_id: input.context_id.clone(),
            edited: input.edited,
            escalated: input.escalated,
            severity: input.severity,
            manual_confidence: input.manual_confidence,
            override_cap: input.override_cap,
            payload_json: input.payload_json.clone(),
        })
    }

    pub fn list_events_for_key(
        &self,
        memory_id: MemoryId,
        version: u32,
        limit: Option<usize>,
    ) -> Result<Vec<OutcomeEvent>> {
        let mut query = "SELECT
                event_seq, event_id, ruleset_version, memory_id, version, event_type,
                occurred_at, recorded_at, writer, justification, context_id,
                edited, escalated, severity, manual_confidence, override_cap, payload_json
             FROM outcome_events
             WHERE memory_id = ?1 AND version = ?2
             ORDER BY event_seq ASC"
            .to_string();

        if let Some(raw_limit) = limit {
            query.push_str(" LIMIT ");
            query.push_str(&raw_limit.to_string());
        }

        let mut stmt = self.conn.prepare(&query)?;
        let rows = stmt.query_map(params![memory_id.to_string(), i64::from(version)], |row| {
            parse_event_row(row)
        })?;

        collect_rows(rows)
    }

    pub fn list_events_from_seq(&self, from_event_seq: i64) -> Result<Vec<OutcomeEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                event_seq, event_id, ruleset_version, memory_id, version, event_type,
                occurred_at, recorded_at, writer, justification, context_id,
                edited, escalated, severity, manual_confidence, override_cap, payload_json
             FROM outcome_events
             WHERE event_seq >= ?1
             ORDER BY event_seq ASC",
        )?;

        let rows = stmt.query_map(params![from_event_seq], parse_event_row)?;
        collect_rows(rows)
    }

    pub fn replay(&mut self, from_event_seq: Option<i64>) -> Result<ReplayReport> {
        let keys = if let Some(from) = from_event_seq {
            self.keys_with_events_from(from)?
        } else {
            self.keys_with_any_events()?
        };

        let rulesets = self.get_rulesets()?;
        let mut projected_keys = 0_usize;
        let mut processed_events = 0_usize;

        for key in keys {
            let events = self.list_events_for_key(key.memory_id, key.version, None)?;
            processed_events += events.len();

            if let Some(trust) = project_memory_trust(&events, &rulesets)
                .map_err(|err| anyhow!("failed projecting {key}: {err}"))?
            {
                self.upsert_memory_trust(
                    &trust,
                    events.last().map_or(1, |item| item.ruleset_version),
                )?;
                projected_keys += 1;
            }
        }

        let last_event_seq = self.latest_event_seq()?.unwrap_or(0);
        let now = format_rfc3339(now_utc()).map_err(|err| anyhow!(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO outcome_projection_state(projector_name, ruleset_version, last_event_seq, updated_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(projector_name) DO UPDATE SET
                   ruleset_version = excluded.ruleset_version,
                   last_event_seq = excluded.last_event_seq,
                   updated_at = excluded.updated_at",
                params![PROJECTOR_NAME, 1_i64, last_event_seq, now],
            )
            .context("failed to update projection state")?;

        Ok(ReplayReport {
            projected_keys,
            processed_events,
            last_event_seq,
        })
    }

    pub fn projector_status(&self) -> Result<ProjectorStatus> {
        let projection_state = self.projection_state(PROJECTOR_NAME)?;
        let (ruleset_version, projected_event_seq, updated_at) = match projection_state {
            Some((ruleset, seq, updated_at)) => (ruleset, seq, Some(updated_at)),
            None => (1_u32, 0_i64, None),
        };

        let latest_event_seq = self.latest_event_seq()?.unwrap_or(0);
        let lag_events = (latest_event_seq - projected_event_seq).max(0);
        let lag_delta_events = lag_events;
        let tracked_keys = self.count_distinct_event_keys()?;
        let trust_rows = self.count_memory_trust_rows()?;
        let stale_keys = self.projector_stale_keys(None)?;
        let stale_trust_rows = stale_keys.len();
        let keys_with_events_no_trust_row = stale_keys
            .iter()
            .filter(|item| item.projected_event_seq.is_none())
            .count();
        let max_stale_seq_gap = stale_keys
            .iter()
            .map(|item| item.max_event_seq - item.projected_event_seq.unwrap_or(0))
            .max()
            .unwrap_or(0);
        let trust_rows_without_events = self.count_trust_rows_without_events()?;

        Ok(ProjectorStatus {
            contract_version: "projector_status.v1".to_string(),
            projector_name: PROJECTOR_NAME.to_string(),
            ruleset_version,
            projected_event_seq,
            latest_event_seq,
            lag_events,
            lag_delta_events,
            tracked_keys,
            trust_rows,
            stale_trust_rows,
            keys_with_events_no_trust_row,
            trust_rows_without_events,
            max_stale_seq_gap,
            updated_at,
        })
    }

    pub fn projector_check(&self) -> Result<ProjectorCheck> {
        let status = self.projector_status()?;
        let mut issues = Vec::new();

        if status.lag_events > 0 {
            issues.push(ProjectorIssue {
                code: "projection_lag".to_string(),
                severity: ProjectorIssueSeverity::Error,
                message: format!(
                    "projection lag detected: {} events behind",
                    status.lag_events
                ),
            });
        }

        if status.stale_trust_rows > 0 {
            issues.push(ProjectorIssue {
                code: "stale_trust_rows".to_string(),
                severity: ProjectorIssueSeverity::Error,
                message: format!(
                    "stale trust rows detected: {} keys out of date",
                    status.stale_trust_rows
                ),
            });
        }

        if status.tracked_keys != status.trust_rows {
            issues.push(ProjectorIssue {
                code: "key_snapshot_mismatch".to_string(),
                severity: ProjectorIssueSeverity::Error,
                message: format!(
                    "key/snapshot mismatch: tracked_keys={} trust_rows={}",
                    status.tracked_keys, status.trust_rows
                ),
            });
        }

        if status.trust_rows_without_events > 0 {
            issues.push(ProjectorIssue {
                code: "orphan_trust_rows".to_string(),
                severity: ProjectorIssueSeverity::Warning,
                message: format!(
                    "trust rows without events detected: {} rows",
                    status.trust_rows_without_events
                ),
            });
        }

        let stale_key_sample = self.projector_stale_keys(Some(25))?;
        let healthy = !issues
            .iter()
            .any(|item| item.severity == ProjectorIssueSeverity::Error);

        Ok(ProjectorCheck {
            contract_version: "projector_check.v1".to_string(),
            healthy,
            status,
            issues,
            stale_key_sample,
        })
    }

    pub fn projector_stale_keys(&self, limit: Option<usize>) -> Result<Vec<ProjectorStaleKey>> {
        let mut query = "SELECT
                events.memory_id,
                events.version,
                events.max_event_seq,
                trust.last_event_seq
             FROM (
                SELECT memory_id, version, MAX(event_seq) AS max_event_seq
                FROM outcome_events
                GROUP BY memory_id, version
             ) events
             LEFT JOIN memory_trust trust
               ON trust.memory_id = events.memory_id
              AND trust.version = events.version
             WHERE trust.last_event_seq IS NULL OR trust.last_event_seq < events.max_event_seq
             ORDER BY events.memory_id ASC, events.version ASC"
            .to_string();

        if let Some(raw_limit) = limit {
            query.push_str(" LIMIT ");
            query.push_str(&raw_limit.to_string());
        }

        let mut stmt = self.conn.prepare(&query)?;
        let rows = stmt.query_map([], |row| {
            let memory_id_raw: String = row.get(0)?;
            let version_i64: i64 = row.get(1)?;
            let max_event_seq: i64 = row.get(2)?;
            let projected_event_seq: Option<i64> = row.get(3)?;

            let version = u32::try_from(version_i64).map_err(|_| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Integer,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("invalid version value: {version_i64}"),
                    )),
                )
            })?;

            Ok(ProjectorStaleKey {
                memory_id: parse_memory_id(&memory_id_raw)?,
                version,
                max_event_seq,
                projected_event_seq,
            })
        })?;

        collect_rows(rows)
    }

    #[allow(clippy::too_many_lines)]
    pub fn run_benchmark(
        &self,
        config: &BenchmarkConfig,
        thresholds: Option<BenchmarkThresholds>,
    ) -> Result<BenchmarkReport> {
        if config.volumes.is_empty() {
            return Err(anyhow!(
                "benchmark config must include at least one volume value"
            ));
        }
        if config.repetitions == 0 {
            return Err(anyhow!("benchmark repetitions must be >= 1"));
        }

        let mut volume_results = Vec::new();

        for &event_count in &config.volumes {
            let mut append_samples_ms = Vec::new();
            let mut replay_samples_ms = Vec::new();
            let mut gate_samples_ms = Vec::new();

            for repetition in 0..config.repetitions {
                let db_path = std::env::temp_dir().join(format!(
                    "outcome-bench-{}-{}-{}.sqlite3",
                    event_count,
                    repetition,
                    Ulid::new()
                ));
                let memory_id = MemoryId(Ulid::new());

                let setup_conn = Connection::open(&db_path).with_context(|| {
                    format!("failed to open benchmark db {}", db_path.display())
                })?;
                seed_minimal_memory_record(&setup_conn, memory_id, 1)?;
                drop(setup_conn);

                let mut store = SqliteOutcomeStore::open(&db_path)?;
                store.migrate()?;

                for index in 0..event_count {
                    let event_type = if index % 17 == 0 {
                        OutcomeEventType::Failure
                    } else if index % 11 == 0 {
                        OutcomeEventType::Ignored
                    } else if index % 13 == 0 {
                        OutcomeEventType::Unknown
                    } else {
                        OutcomeEventType::Success
                    };

                    let start = Instant::now();
                    let input = benchmark_event_input(memory_id, 1, 1, event_type);
                    let _ = store.append_event(&input)?;
                    append_samples_ms.push(start.elapsed().as_secs_f64() * 1_000.0);
                }

                let replay_start = Instant::now();
                let _ = store.replay(None)?;
                replay_samples_ms.push(replay_start.elapsed().as_secs_f64() * 1_000.0);

                let gate_start = Instant::now();
                let _ = store.gate_preview(
                    RetrievalMode::Safe,
                    now_utc(),
                    Some("bench"),
                    &[MemoryKey {
                        memory_id,
                        version: 1,
                    }],
                )?;
                gate_samples_ms.push(gate_start.elapsed().as_secs_f64() * 1_000.0);

                drop(store);
                let _ = std::fs::remove_file(&db_path);
            }

            let result = BenchmarkVolumeResult {
                event_count,
                append_p50_ms: percentile(&append_samples_ms, 0.50),
                append_p95_ms: percentile(&append_samples_ms, 0.95),
                replay_p50_ms: percentile(&replay_samples_ms, 0.50),
                replay_p95_ms: percentile(&replay_samples_ms, 0.95),
                gate_p50_ms: percentile(&gate_samples_ms, 0.50),
                gate_p95_ms: percentile(&gate_samples_ms, 0.95),
            };
            volume_results.push(result);
        }

        let mut violations = Vec::new();
        if let Some(limit) = &thresholds {
            for volume in &volume_results {
                if volume.append_p95_ms > limit.append_p95_ms_max {
                    violations.push(format!(
                        "volume={} append_p95_ms={} exceeds max={}",
                        volume.event_count, volume.append_p95_ms, limit.append_p95_ms_max
                    ));
                }
                if volume.replay_p95_ms > limit.replay_p95_ms_max {
                    violations.push(format!(
                        "volume={} replay_p95_ms={} exceeds max={}",
                        volume.event_count, volume.replay_p95_ms, limit.replay_p95_ms_max
                    ));
                }
                if volume.gate_p95_ms > limit.gate_p95_ms_max {
                    violations.push(format!(
                        "volume={} gate_p95_ms={} exceeds max={}",
                        volume.event_count, volume.gate_p95_ms, limit.gate_p95_ms_max
                    ));
                }
            }
        }

        Ok(BenchmarkReport {
            contract_version: "benchmark_report.v1".to_string(),
            generated_at: format_rfc3339(now_utc()).map_err(|err| anyhow!(err.to_string()))?,
            repetitions: config.repetitions,
            volumes: volume_results,
            thresholds,
            within_thresholds: violations.is_empty(),
            violations,
        })
    }

    pub fn get_memory_trust(
        &self,
        memory_id: MemoryId,
        version: u32,
        as_of: Option<time::OffsetDateTime>,
    ) -> Result<Option<MemoryTrust>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                memory_id, version, confidence_raw, confidence_effective, baseline_confidence,
                trust_status, contradiction_cap_active, cap_value, manual_override_active,
                wins_last5, failures_last5, last_event_seq, last_ruleset_version,
                last_scored_at, updated_at
             FROM memory_trust
             WHERE memory_id = ?1 AND version = ?2",
        )?;

        let row = stmt
            .query_row(params![memory_id.to_string(), i64::from(version)], |row| {
                parse_memory_trust_row(row)
            })
            .optional()?;

        let Some((trust, last_ruleset_version)) = row else {
            return Ok(None);
        };

        let Some(as_of_value) = as_of else {
            return Ok(Some(trust));
        };

        let rulesets = self.get_rulesets()?;
        let Some(ruleset) = rulesets.get(&last_ruleset_version) else {
            return Err(anyhow!(
                "missing ruleset {last_ruleset_version} for trust decay",
            ));
        };

        Ok(Some(apply_as_of_decay(&trust, ruleset, as_of_value)))
    }

    pub fn gate_preview(
        &self,
        mode: RetrievalMode,
        as_of: time::OffsetDateTime,
        context_id: Option<&str>,
        candidates: &[MemoryKey],
    ) -> Result<Vec<GateDecision>> {
        let rulesets = self.get_rulesets()?;
        let mut decisions = Vec::new();

        for candidate in candidates {
            let Some((trust, last_ruleset_version)) =
                self.get_memory_trust_and_ruleset(candidate.memory_id, candidate.version)?
            else {
                decisions.push(GateDecision {
                    memory_id: candidate.memory_id,
                    version: candidate.version,
                    include: false,
                    confidence_effective: 0.0,
                    trust_status: TrustStatus::Active,
                    capped: false,
                    reason_codes: vec!["excluded.no_trust_snapshot".to_string()],
                });
                continue;
            };

            let ruleset = rulesets
                .get(&last_ruleset_version)
                .ok_or_else(|| anyhow!("missing ruleset {last_ruleset_version}"))?;
            let trust_with_decay = apply_as_of_decay(&trust, ruleset, as_of);
            decisions.push(gate_memory(&trust_with_decay, mode, context_id, ruleset));
        }

        Ok(decisions)
    }

    fn keys_with_events_from(&self, from_event_seq: i64) -> Result<Vec<MemoryKey>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT memory_id, version
             FROM outcome_events
             WHERE event_seq >= ?1
             ORDER BY memory_id ASC, version ASC",
        )?;

        let rows = stmt.query_map(params![from_event_seq], |row| {
            let memory_id_raw: String = row.get(0)?;
            let version_i64: i64 = row.get(1)?;
            let version = u32::try_from(version_i64).map_err(|_| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Integer,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("invalid version value: {version_i64}"),
                    )),
                )
            })?;
            Ok(MemoryKey {
                memory_id: parse_memory_id(&memory_id_raw)?,
                version,
            })
        })?;

        collect_rows(rows)
    }

    fn keys_with_any_events(&self) -> Result<Vec<MemoryKey>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT memory_id, version
             FROM outcome_events
             ORDER BY memory_id ASC, version ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            let memory_id_raw: String = row.get(0)?;
            let version_i64: i64 = row.get(1)?;
            let version = u32::try_from(version_i64).map_err(|_| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Integer,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("invalid version value: {version_i64}"),
                    )),
                )
            })?;
            Ok(MemoryKey {
                memory_id: parse_memory_id(&memory_id_raw)?,
                version,
            })
        })?;

        collect_rows(rows)
    }

    fn latest_event_seq(&self) -> Result<Option<i64>> {
        let value = self
            .conn
            .query_row("SELECT MAX(event_seq) FROM outcome_events", [], |row| {
                row.get::<_, Option<i64>>(0)
            })
            .context("failed to query latest event_seq")?;
        Ok(value)
    }

    fn projection_state(&self, projector_name: &str) -> Result<Option<(u32, i64, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT ruleset_version, last_event_seq, updated_at
             FROM outcome_projection_state
             WHERE projector_name = ?1",
        )?;

        let row = stmt
            .query_row(params![projector_name], |row| {
                let ruleset_i64: i64 = row.get(0)?;
                let ruleset_version = u32::try_from(ruleset_i64).map_err(|_| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Integer,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("invalid ruleset version: {ruleset_i64}"),
                        )),
                    )
                })?;
                let last_event_seq: i64 = row.get(1)?;
                let updated_at: String = row.get(2)?;
                Ok((ruleset_version, last_event_seq, updated_at))
            })
            .optional()?;

        Ok(row)
    }

    fn count_distinct_event_keys(&self) -> Result<usize> {
        let count = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM (
                    SELECT DISTINCT memory_id, version
                    FROM outcome_events
                 )",
                [],
                |row| row.get::<_, i64>(0),
            )
            .context("failed to count distinct outcome event keys")?;

        usize::try_from(count).with_context(|| format!("invalid distinct key count: {count}"))
    }

    fn count_memory_trust_rows(&self) -> Result<usize> {
        let count = self
            .conn
            .query_row("SELECT COUNT(*) FROM memory_trust", [], |row| {
                row.get::<_, i64>(0)
            })
            .context("failed to count memory_trust rows")?;
        usize::try_from(count).with_context(|| format!("invalid memory_trust row count: {count}"))
    }

    fn count_trust_rows_without_events(&self) -> Result<usize> {
        let count = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM (
                    SELECT trust.memory_id, trust.version
                    FROM memory_trust trust
                    LEFT JOIN (
                        SELECT DISTINCT memory_id, version
                        FROM outcome_events
                    ) events
                      ON events.memory_id = trust.memory_id
                     AND events.version = trust.version
                    WHERE events.memory_id IS NULL
                )",
                [],
                |row| row.get::<_, i64>(0),
            )
            .context("failed to count trust rows without events")?;
        usize::try_from(count)
            .with_context(|| format!("invalid trust rows without events count: {count}"))
    }

    fn upsert_memory_trust(&mut self, trust: &MemoryTrust, ruleset_version: u32) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO memory_trust(
                    memory_id, version, confidence_raw, confidence_effective, baseline_confidence,
                    trust_status, contradiction_cap_active, cap_value, manual_override_active,
                    wins_last5, failures_last5, last_event_seq, last_ruleset_version, last_scored_at,
                    updated_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5,
                    ?6, ?7, ?8, ?9,
                    ?10, ?11, ?12, ?13, ?14,
                    ?15
                 )
                 ON CONFLICT(memory_id, version) DO UPDATE SET
                    confidence_raw = excluded.confidence_raw,
                    confidence_effective = excluded.confidence_effective,
                    baseline_confidence = excluded.baseline_confidence,
                    trust_status = excluded.trust_status,
                    contradiction_cap_active = excluded.contradiction_cap_active,
                    cap_value = excluded.cap_value,
                    manual_override_active = excluded.manual_override_active,
                    wins_last5 = excluded.wins_last5,
                    failures_last5 = excluded.failures_last5,
                    last_event_seq = excluded.last_event_seq,
                    last_ruleset_version = excluded.last_ruleset_version,
                    last_scored_at = excluded.last_scored_at,
                    updated_at = excluded.updated_at",
                params![
                    trust.memory_id.to_string(),
                    i64::from(trust.version),
                    trust.confidence_raw,
                    trust.confidence_effective,
                    trust.baseline_confidence,
                    trust.trust_status.as_str(),
                    bool_to_sql(trust.contradiction_cap_active),
                    trust.cap_value,
                    bool_to_sql(trust.manual_override_active),
                    i64::from(trust.wins_last5),
                    i64::from(trust.failures_last5),
                    trust.last_event_seq,
                    i64::from(ruleset_version),
                    trust
                        .last_scored_at
                        .map(format_rfc3339)
                        .transpose()
                        .map_err(|err| anyhow!(err.to_string()))?,
                    format_rfc3339(trust.updated_at).map_err(|err| anyhow!(err.to_string()))?,
                ],
            )
            .context("failed to upsert memory_trust snapshot")?;

        Ok(())
    }

    fn get_memory_trust_and_ruleset(
        &self,
        memory_id: MemoryId,
        version: u32,
    ) -> Result<Option<(MemoryTrust, u32)>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                memory_id, version, confidence_raw, confidence_effective, baseline_confidence,
                trust_status, contradiction_cap_active, cap_value, manual_override_active,
                wins_last5, failures_last5, last_event_seq, last_ruleset_version,
                last_scored_at, updated_at
             FROM memory_trust
             WHERE memory_id = ?1 AND version = ?2",
        )?;

        let row = stmt
            .query_row(params![memory_id.to_string(), i64::from(version)], |row| {
                parse_memory_trust_row(row)
            })
            .optional()?;

        Ok(row)
    }

    #[cfg(test)]
    fn connection(&self) -> &Connection {
        &self.conn
    }
}

fn parse_event_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<OutcomeEvent> {
    let event_id_raw: String = row.get(1)?;
    let ruleset_version_i64: i64 = row.get(2)?;
    let memory_id_raw: String = row.get(3)?;
    let version_i64: i64 = row.get(4)?;
    let event_type_raw: String = row.get(5)?;
    let severity_raw: Option<String> = row.get(13)?;
    let payload_json: String = row.get(16)?;

    let ruleset_version = u32::try_from(ruleset_version_i64).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Integer,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid ruleset_version: {ruleset_version_i64}"),
            )),
        )
    })?;

    let version = u32::try_from(version_i64).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            4,
            rusqlite::types::Type::Integer,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid version: {version_i64}"),
            )),
        )
    })?;

    let event_type = OutcomeEventType::parse(&event_type_raw).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            5,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid event_type: {event_type_raw}"),
            )),
        )
    })?;

    let severity = severity_raw
        .as_deref()
        .map(|raw| {
            Severity::parse(raw).ok_or_else(|| {
                rusqlite::Error::FromSqlConversionFailure(
                    13,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("invalid severity: {raw}"),
                    )),
                )
            })
        })
        .transpose()?;

    let event_id = Ulid::from_string(&event_id_raw).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            1,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid event_id ULID: {event_id_raw}"),
            )),
        )
    })?;

    let memory_id = parse_memory_id(&memory_id_raw)?;
    let occurred_at = parse_rfc3339_utc(&row.get::<_, String>(6)?).map_err(to_sql_error)?;
    let recorded_at = parse_rfc3339_utc(&row.get::<_, String>(7)?).map_err(to_sql_error)?;
    let payload_value: Value = serde_json::from_str(&payload_json).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            16,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid payload_json: {err}"),
            )),
        )
    })?;

    Ok(OutcomeEvent {
        event_seq: row.get(0)?,
        event_id,
        ruleset_version,
        memory_id,
        version,
        event_type,
        occurred_at,
        recorded_at,
        writer: row.get(8)?,
        justification: row.get(9)?,
        context_id: row.get(10)?,
        edited: row.get::<_, i64>(11)? == 1,
        escalated: row.get::<_, i64>(12)? == 1,
        severity,
        manual_confidence: row.get(14)?,
        override_cap: row.get::<_, i64>(15)? == 1,
        payload_json: payload_value,
    })
}

fn parse_memory_trust_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<(MemoryTrust, u32)> {
    let memory_id_raw: String = row.get(0)?;
    let version_i64: i64 = row.get(1)?;
    let trust_status_raw: String = row.get(5)?;
    let last_ruleset_i64: i64 = row.get(12)?;

    let version = u32::try_from(version_i64).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            1,
            rusqlite::types::Type::Integer,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid version: {version_i64}"),
            )),
        )
    })?;

    let last_ruleset_version = u32::try_from(last_ruleset_i64).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            12,
            rusqlite::types::Type::Integer,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid ruleset version: {last_ruleset_i64}"),
            )),
        )
    })?;

    let trust_status = TrustStatus::parse(&trust_status_raw).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            5,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid trust_status: {trust_status_raw}"),
            )),
        )
    })?;

    let last_scored_at = row
        .get::<_, Option<String>>(13)?
        .as_deref()
        .map(|value| parse_rfc3339_utc(value).map_err(to_sql_error))
        .transpose()?;

    let memory_id = parse_memory_id(&memory_id_raw)?;

    Ok((
        MemoryTrust {
            memory_id,
            version,
            confidence_raw: row.get(2)?,
            confidence_effective: row.get(3)?,
            baseline_confidence: row.get(4)?,
            trust_status,
            contradiction_cap_active: row.get::<_, i64>(6)? == 1,
            cap_value: row.get(7)?,
            manual_override_active: row.get::<_, i64>(8)? == 1,
            wins_last5: row.get(9)?,
            failures_last5: row.get(10)?,
            last_event_seq: row.get(11)?,
            last_scored_at,
            updated_at: parse_rfc3339_utc(&row.get::<_, String>(14)?).map_err(to_sql_error)?,
        },
        last_ruleset_version,
    ))
}

fn parse_memory_id(raw: &str) -> rusqlite::Result<MemoryId> {
    let parsed = Ulid::from_string(raw).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid ULID: {raw}"),
            )),
        )
    })?;

    Ok(MemoryId(parsed))
}

fn bool_to_sql(value: bool) -> i64 {
    i64::from(value)
}

fn ensure_memory_kernel_compatibility(conn: &Connection) -> Result<()> {
    let has_memory_records = table_exists(conn, "memory_records")?;
    if !has_memory_records {
        return Err(anyhow!(
            "MemoryKernel compatibility check failed: expected table memory_records"
        ));
    }

    ensure_table_has_columns(conn, "memory_records", &["memory_id", "version"])?;
    ensure_unique_index_on_columns(conn, "memory_records", &["memory_id", "version"])?;
    Ok(())
}

fn table_exists(conn: &Connection, table_name: &str) -> Result<bool> {
    let exists = conn
        .query_row(
            "SELECT 1
             FROM sqlite_master
             WHERE type = 'table' AND name = ?1
             LIMIT 1",
            params![table_name],
            |_| Ok(()),
        )
        .optional()
        .context("failed to query sqlite_master")?
        .is_some();

    Ok(exists)
}

fn ensure_table_has_columns(conn: &Connection, table_name: &str, columns: &[&str]) -> Result<()> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table_name})"))
        .with_context(|| format!("failed to inspect table_info for {table_name}"))?;
    let mut rows = stmt.query([])?;

    let mut available = Vec::new();
    while let Some(row) = rows.next()? {
        available.push(row.get::<_, String>(1)?);
    }

    for required in columns {
        if !available.iter().any(|candidate| candidate == required) {
            return Err(anyhow!(
                "MemoryKernel compatibility check failed: missing column {table_name}.{required}"
            ));
        }
    }

    Ok(())
}

fn ensure_unique_index_on_columns(
    conn: &Connection,
    table_name: &str,
    columns: &[&str],
) -> Result<()> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA index_list({table_name})"))
        .with_context(|| format!("failed to inspect index_list for {table_name}"))?;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let index_name: String = row.get(1)?;
        let is_unique: i64 = row.get(2)?;
        if is_unique != 1 {
            continue;
        }

        let indexed_columns = index_columns(conn, &index_name)?;
        if indexed_columns == columns {
            return Ok(());
        }
    }

    Err(anyhow!(
        "MemoryKernel compatibility check failed: expected UNIQUE(memory_id, version) on memory_records"
    ))
}

fn index_columns(conn: &Connection, index_name: &str) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA index_info({index_name})"))
        .with_context(|| format!("failed to inspect index_info for {index_name}"))?;
    let mut rows = stmt.query([])?;

    let mut columns = Vec::new();
    while let Some(row) = rows.next()? {
        columns.push(row.get::<_, String>(2)?);
    }

    Ok(columns)
}

fn benchmark_event_input(
    memory_id: MemoryId,
    version: u32,
    ruleset_version: u32,
    event_type: OutcomeEventType,
) -> OutcomeEventInput {
    OutcomeEventInput {
        event_id: None,
        ruleset_version,
        memory_id,
        version,
        event_type,
        occurred_at: now_utc(),
        writer: "benchmark".to_string(),
        justification: "benchmark run".to_string(),
        context_id: Some("bench".to_string()),
        edited: false,
        escalated: false,
        severity: None,
        manual_confidence: None,
        override_cap: false,
        payload_json: Value::Object(serde_json::Map::default()),
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn percentile(values: &[f64], percentile_rank: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap_or(std::cmp::Ordering::Equal));

    let position = (percentile_rank * sorted.len() as f64).ceil() as usize;
    let index = position.saturating_sub(1).min(sorted.len() - 1);
    sorted[index]
}

#[allow(clippy::needless_pass_by_value)]
fn to_sql_error(err: memory_kernel_outcome_core::OutcomeError) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            err.to_string(),
        )),
    )
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>,
) -> Result<Vec<T>> {
    let mut values = Vec::new();
    for row in rows {
        values.push(row?);
    }
    Ok(values)
}

pub fn parse_memory_key(raw: &str) -> Result<MemoryKey> {
    let mut parts = raw.split(':');
    let memory_id_raw = parts
        .next()
        .ok_or_else(|| anyhow!("candidate must be in <memory_id>:<version> format"))?;
    let version_raw = parts
        .next()
        .ok_or_else(|| anyhow!("candidate must be in <memory_id>:<version> format"))?;

    if parts.next().is_some() {
        return Err(anyhow!("candidate must be in <memory_id>:<version> format"));
    }

    let parsed_id = Ulid::from_string(memory_id_raw)
        .with_context(|| format!("invalid ULID memory_id: {memory_id_raw}"))?;
    let version: u32 = version_raw
        .parse()
        .with_context(|| format!("invalid version integer: {version_raw}"))?;

    if version == 0 {
        return Err(anyhow!("version MUST be >= 1"));
    }

    Ok(MemoryKey {
        memory_id: MemoryId(parsed_id),
        version,
    })
}

pub fn seed_minimal_memory_record(
    conn: &Connection,
    memory_id: MemoryId,
    version: u32,
) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS memory_records (
            memory_version_id TEXT PRIMARY KEY,
            memory_id TEXT NOT NULL,
            version INTEGER NOT NULL,
            UNIQUE(memory_id, version)
         );",
    )
    .context("failed to create minimal memory_records table for tests")?;

    conn.execute(
        "INSERT OR IGNORE INTO memory_records(memory_version_id, memory_id, version) VALUES (?1, ?2, ?3)",
        params![Ulid::new().to_string(), memory_id.to_string(), i64::from(version)],
    )
    .context("failed to seed memory_records row")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::manual_let_else,
        clippy::float_cmp,
        clippy::default_trait_access,
        clippy::too_many_lines
    )]

    use super::*;
    use proptest::prelude::*;
    use std::collections::BTreeSet;
    use std::time::Instant;

    fn must<T>(result: Result<T>) -> T {
        match result {
            Ok(value) => value,
            Err(err) => panic!("test failure: {err}"),
        }
    }

    fn fixture_store() -> SqliteOutcomeStore {
        let store = must(SqliteOutcomeStore::open(Path::new(":memory:")));
        let create_result = store.connection().execute_batch(
            "CREATE TABLE IF NOT EXISTS memory_records (
                memory_version_id TEXT PRIMARY KEY,
                memory_id TEXT NOT NULL,
                version INTEGER NOT NULL,
                UNIQUE(memory_id, version)
             );",
        );
        if let Err(err) = create_result {
            panic!("test failure: {err}");
        }
        must(store.migrate());
        store
    }

    fn assert_trust_equivalent(lhs: &MemoryTrust, rhs: &MemoryTrust) {
        assert_eq!(lhs.memory_id, rhs.memory_id);
        assert_eq!(lhs.version, rhs.version);
        assert_eq!(lhs.confidence_raw, rhs.confidence_raw);
        assert_eq!(lhs.confidence_effective, rhs.confidence_effective);
        assert_eq!(lhs.baseline_confidence, rhs.baseline_confidence);
        assert_eq!(lhs.trust_status, rhs.trust_status);
        assert_eq!(lhs.contradiction_cap_active, rhs.contradiction_cap_active);
        assert_eq!(lhs.cap_value, rhs.cap_value);
        assert_eq!(lhs.manual_override_active, rhs.manual_override_active);
        assert_eq!(lhs.wins_last5, rhs.wins_last5);
        assert_eq!(lhs.failures_last5, rhs.failures_last5);
        assert_eq!(lhs.last_event_seq, rhs.last_event_seq);
        assert_eq!(lhs.last_scored_at, rhs.last_scored_at);
    }

    fn fixture_memory_id() -> MemoryId {
        let parsed = match Ulid::from_string("01J0SQQP7M70P6Y3R4T8D8G8M2") {
            Ok(value) => value,
            Err(err) => panic!("invalid fixture ULID: {err}"),
        };
        MemoryId(parsed)
    }

    fn fixture_event_input(event_type: OutcomeEventType) -> OutcomeEventInput {
        fixture_event_input_for(fixture_memory_id(), 1, 1, event_type)
    }

    fn fixture_event_input_for(
        memory_id: MemoryId,
        version: u32,
        ruleset_version: u32,
        event_type: OutcomeEventType,
    ) -> OutcomeEventInput {
        OutcomeEventInput {
            event_id: None,
            ruleset_version,
            memory_id,
            version,
            event_type,
            occurred_at: match parse_rfc3339_utc("2026-02-07T12:00:00Z") {
                Ok(value) => value,
                Err(err) => panic!("invalid fixture timestamp: {err}"),
            },
            writer: "tester".to_string(),
            justification: "fixture".to_string(),
            context_id: Some("ctx-1".to_string()),
            edited: false,
            escalated: false,
            severity: None,
            manual_confidence: None,
            override_cap: false,
            payload_json: Value::Object(Default::default()),
        }
    }

    fn seed_memory_row(store: &SqliteOutcomeStore) {
        must(seed_minimal_memory_record(
            store.connection(),
            fixture_memory_id(),
            1,
        ));
    }

    #[test]
    fn append_only_trigger_blocks_updates() {
        let mut store = fixture_store();
        seed_memory_row(&store);

        let input = fixture_event_input(OutcomeEventType::Success);
        let event = must(store.append_event(&input));

        let update_result = store.connection().execute(
            "UPDATE outcome_events SET writer = 'mutated' WHERE event_seq = ?1",
            params![event.event_seq],
        );

        assert!(update_result.is_err());
    }

    #[test]
    fn replay_projection_is_deterministic_incremental_vs_full() {
        let mut store = fixture_store();
        seed_memory_row(&store);

        let event1 = must(store.append_event(&fixture_event_input(OutcomeEventType::Success)));
        let event2 = must(store.append_event(&fixture_event_input(OutcomeEventType::Success)));
        let event3 = must(store.append_event(&fixture_event_input(OutcomeEventType::Failure)));

        let full = must(store.replay(None));
        assert_eq!(full.processed_events, 3);

        let trust_after_full = match must(store.get_memory_trust(fixture_memory_id(), 1, None)) {
            Some(value) => value,
            None => panic!("missing trust snapshot after full replay"),
        };

        let event4 = must(store.append_event(&fixture_event_input(OutcomeEventType::Success)));
        let incremental = must(store.replay(Some(event4.event_seq)));
        assert_eq!(incremental.processed_events, 4);

        let trust_after_incremental =
            match must(store.get_memory_trust(fixture_memory_id(), 1, None)) {
                Some(value) => value,
                None => panic!("missing trust snapshot after incremental replay"),
            };

        let mut fresh = fixture_store();
        seed_memory_row(&fresh);
        let mut ids = BTreeSet::new();
        for item in [event1, event2, event3, event4] {
            let mut input = fixture_event_input(item.event_type);
            input.event_id = Some(item.event_id);
            input.occurred_at = item.occurred_at;
            let inserted = must(fresh.append_event(&input));
            ids.insert(inserted.event_id);
        }
        assert_eq!(ids.len(), 4);

        must(fresh.replay(None));
        let trust_fresh = match must(fresh.get_memory_trust(fixture_memory_id(), 1, None)) {
            Some(value) => value,
            None => panic!("missing trust snapshot in fresh replay"),
        };

        assert_eq!(trust_after_incremental.memory_id, trust_fresh.memory_id);
        assert_eq!(trust_after_incremental.version, trust_fresh.version);
        assert_eq!(
            trust_after_incremental.confidence_raw,
            trust_fresh.confidence_raw
        );
        assert_eq!(
            trust_after_incremental.confidence_effective,
            trust_fresh.confidence_effective
        );
        assert_eq!(
            trust_after_incremental.baseline_confidence,
            trust_fresh.baseline_confidence
        );
        assert_eq!(
            trust_after_incremental.trust_status,
            trust_fresh.trust_status
        );
        assert_eq!(
            trust_after_incremental.contradiction_cap_active,
            trust_fresh.contradiction_cap_active
        );
        assert_eq!(trust_after_incremental.cap_value, trust_fresh.cap_value);
        assert_eq!(
            trust_after_incremental.manual_override_active,
            trust_fresh.manual_override_active
        );
        assert_eq!(trust_after_incremental.wins_last5, trust_fresh.wins_last5);
        assert_eq!(
            trust_after_incremental.failures_last5,
            trust_fresh.failures_last5
        );
        assert_eq!(
            trust_after_incremental.last_event_seq,
            trust_fresh.last_event_seq
        );
        assert_eq!(
            trust_after_incremental.last_scored_at,
            trust_fresh.last_scored_at
        );
        assert_ne!(
            trust_after_full.last_event_seq,
            trust_after_incremental.last_event_seq
        );
    }

    fn event_type_from_code(code: u8) -> OutcomeEventType {
        match code % 4 {
            0 => OutcomeEventType::Success,
            1 => OutcomeEventType::Failure,
            2 => OutcomeEventType::Ignored,
            _ => OutcomeEventType::Unknown,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        #[test]
        fn prop_event_stream_replay_equivalence(event_codes in prop::collection::vec(0u8..4, 1..80)) {
            let mut store = fixture_store();
            seed_memory_row(&store);

            let mut split_seq = None;
            for (index, code) in event_codes.iter().copied().enumerate() {
                let event = must(store.append_event(&fixture_event_input(event_type_from_code(code))));
                if index == event_codes.len() / 2 {
                    split_seq = Some(event.event_seq);
                }
            }

            let _ = must(store.replay(None));
            let trust_full = match must(store.get_memory_trust(fixture_memory_id(), 1, None)) {
                Some(value) => value,
                None => panic!("missing trust after full replay"),
            };

            let _ = must(store.replay(split_seq));
            let trust_incremental = match must(store.get_memory_trust(fixture_memory_id(), 1, None)) {
                Some(value) => value,
                None => panic!("missing trust after incremental replay"),
            };

            assert_trust_equivalent(&trust_full, &trust_incremental);
        }

        #[test]
        fn prop_mixed_ruleset_replay_equivalence(stream in prop::collection::vec((0u8..4, any::<bool>()), 1..80)) {
            let mut store = fixture_store();
            seed_memory_row(&store);

            let mut ruleset_v2 = OutcomeRuleset::v1();
            ruleset_v2.ruleset_version = 2;
            ruleset_v2.alpha = 0.11;
            ruleset_v2.failure_weight = -1.45;
            must(store.upsert_ruleset(&ruleset_v2));

            let mut inserted_events = Vec::new();
            for (code, use_v2) in stream {
                let mut input = fixture_event_input(event_type_from_code(code));
                input.ruleset_version = if use_v2 { 2 } else { 1 };
                let event = must(store.append_event(&input));
                inserted_events.push(event);
            }

            let _ = must(store.replay(None));
            let trust_original = match must(store.get_memory_trust(fixture_memory_id(), 1, None)) {
                Some(value) => value,
                None => panic!("missing original trust"),
            };

            let mut fresh = fixture_store();
            seed_memory_row(&fresh);
            must(fresh.upsert_ruleset(&ruleset_v2));
            for event in inserted_events {
                let mut input = fixture_event_input(event.event_type);
                input.event_id = Some(event.event_id);
                input.ruleset_version = event.ruleset_version;
                input.occurred_at = event.occurred_at;
                let _ = must(fresh.append_event(&input));
            }
            let _ = must(fresh.replay(None));
            let trust_fresh = match must(fresh.get_memory_trust(fixture_memory_id(), 1, None)) {
                Some(value) => value,
                None => panic!("missing fresh trust"),
            };

            assert_trust_equivalent(&trust_original, &trust_fresh);
        }
    }

    #[test]
    fn golden_fixture_replay_snapshot_matches_expected() {
        let mut store = fixture_store();
        seed_memory_row(&store);

        let fixture_events = [
            OutcomeEventType::Success,
            OutcomeEventType::Success,
            OutcomeEventType::Ignored,
            OutcomeEventType::Success,
            OutcomeEventType::Unknown,
        ];

        for event_type in fixture_events {
            let input = fixture_event_input(event_type);
            let _ = must(store.append_event(&input));
        }

        let _ = must(store.replay(None));
        let trust = match must(store.get_memory_trust(fixture_memory_id(), 1, None)) {
            Some(value) => value,
            None => panic!("missing trust snapshot"),
        };

        assert_eq!(trust.trust_status, TrustStatus::Validated);
        assert_eq!(trust.wins_last5, 3);
        assert_eq!(trust.failures_last5, 0);
        assert!(trust.confidence_effective > 0.55);
    }

    #[test]
    fn gate_preview_supports_safe_and_exploration_modes() {
        let mut store = fixture_store();
        seed_memory_row(&store);

        let events = [
            fixture_event_input(OutcomeEventType::Success),
            fixture_event_input(OutcomeEventType::Success),
            fixture_event_input(OutcomeEventType::Success),
        ];

        for input in events {
            let _ = must(store.append_event(&input));
        }

        let _ = must(store.replay(None));

        let as_of = match parse_rfc3339_utc("2026-02-07T12:00:00Z") {
            Ok(value) => value,
            Err(err) => panic!("invalid as_of: {err}"),
        };

        let decisions_safe = must(store.gate_preview(
            RetrievalMode::Safe,
            as_of,
            Some("ctx-1"),
            &[MemoryKey {
                memory_id: fixture_memory_id(),
                version: 1,
            }],
        ));
        assert_eq!(decisions_safe.len(), 1);
        assert!(decisions_safe[0].include);

        let decisions_explore = must(store.gate_preview(
            RetrievalMode::Exploration,
            as_of,
            Some("ctx-1"),
            &[MemoryKey {
                memory_id: fixture_memory_id(),
                version: 1,
            }],
        ));
        assert_eq!(decisions_explore.len(), 1);
        assert!(decisions_explore[0].include);
    }

    #[test]
    fn migrate_fails_when_memory_records_table_missing() {
        let store = must(SqliteOutcomeStore::open(Path::new(":memory:")));
        let err = match store.migrate() {
            Ok(()) => panic!("migration should fail without memory_records table"),
            Err(err) => err,
        };

        assert!(err.to_string().contains("expected table memory_records"));
    }

    #[test]
    fn migrate_fails_without_unique_memory_id_version_key() {
        let store = must(SqliteOutcomeStore::open(Path::new(":memory:")));
        let create_result = store.connection().execute_batch(
            "CREATE TABLE memory_records (
                memory_version_id TEXT PRIMARY KEY,
                memory_id TEXT NOT NULL,
                version INTEGER NOT NULL
            );",
        );
        if let Err(err) = create_result {
            panic!("test setup failed: {err}");
        }

        let err = match store.migrate() {
            Ok(()) => panic!("migration should fail without unique memory_id/version"),
            Err(err) => err,
        };

        assert!(err
            .to_string()
            .contains("expected UNIQUE(memory_id, version) on memory_records"));
    }

    #[test]
    fn migrate_fails_when_memory_records_version_column_missing() {
        let store = must(SqliteOutcomeStore::open(Path::new(":memory:")));
        let create_result = store.connection().execute_batch(
            "CREATE TABLE memory_records (
                memory_version_id TEXT PRIMARY KEY,
                memory_id TEXT NOT NULL
            );",
        );
        if let Err(err) = create_result {
            panic!("test setup failed: {err}");
        }

        let err = match store.migrate() {
            Ok(()) => panic!("migration should fail without version column"),
            Err(err) => err,
        };

        assert!(err
            .to_string()
            .contains("missing column memory_records.version"));
    }

    #[test]
    fn migrate_succeeds_with_explicit_unique_index_on_identity_columns() {
        let store = must(SqliteOutcomeStore::open(Path::new(":memory:")));
        let create_result = store.connection().execute_batch(
            "CREATE TABLE memory_records (
                memory_version_id TEXT PRIMARY KEY,
                memory_id TEXT NOT NULL,
                version INTEGER NOT NULL
             );
             CREATE UNIQUE INDEX idx_memory_identity ON memory_records(memory_id, version);",
        );
        if let Err(err) = create_result {
            panic!("test setup failed: {err}");
        }

        must(store.migrate());
    }

    #[test]
    fn projector_status_and_check_report_lag_and_recovery() {
        let mut store = fixture_store();
        seed_memory_row(&store);

        let _ = must(store.append_event(&fixture_event_input(OutcomeEventType::Success)));
        let status_before_replay = must(store.projector_status());
        assert_eq!(status_before_replay.contract_version, "projector_status.v1");
        assert_eq!(status_before_replay.lag_events, 1);
        assert_eq!(status_before_replay.lag_delta_events, 1);
        assert_eq!(status_before_replay.stale_trust_rows, 1);
        assert_eq!(status_before_replay.keys_with_events_no_trust_row, 1);
        assert_eq!(status_before_replay.trust_rows_without_events, 0);
        assert_eq!(status_before_replay.max_stale_seq_gap, 1);

        let check_before = must(store.projector_check());
        assert_eq!(check_before.contract_version, "projector_check.v1");
        assert!(!check_before.healthy);
        assert!(!check_before.issues.is_empty());
        assert!(!check_before.stale_key_sample.is_empty());

        let _ = must(store.replay(None));
        let status_after_replay = must(store.projector_status());
        assert_eq!(status_after_replay.lag_events, 0);
        assert_eq!(status_after_replay.stale_trust_rows, 0);
        assert_eq!(status_after_replay.tracked_keys, 1);
        assert_eq!(status_after_replay.trust_rows, 1);

        let check_after = must(store.projector_check());
        assert!(check_after.healthy);
        assert!(check_after.issues.is_empty());
        assert!(check_after.stale_key_sample.is_empty());
    }

    #[test]
    fn projector_stale_keys_lists_out_of_date_identities() {
        let mut store = fixture_store();
        seed_memory_row(&store);

        let _ = must(store.append_event(&fixture_event_input(OutcomeEventType::Success)));

        let stale_keys = must(store.projector_stale_keys(None));
        assert_eq!(stale_keys.len(), 1);
        assert_eq!(stale_keys[0].memory_id, fixture_memory_id());
        assert_eq!(stale_keys[0].version, 1);
        assert_eq!(stale_keys[0].projected_event_seq, None);

        let _ = must(store.replay(None));
        let stale_after = must(store.projector_stale_keys(None));
        assert!(stale_after.is_empty());
    }

    #[test]
    fn long_stream_replay_stays_deterministic() {
        let mut store = fixture_store();
        seed_memory_row(&store);

        let mut baseline_events = Vec::new();
        for index in 0..750 {
            let event_type = if index % 19 == 0 {
                OutcomeEventType::Failure
            } else if index % 11 == 0 {
                OutcomeEventType::Ignored
            } else if index % 13 == 0 {
                OutcomeEventType::Unknown
            } else {
                OutcomeEventType::Success
            };

            let inserted = must(store.append_event(&fixture_event_input(event_type)));
            baseline_events.push(inserted);
        }

        let _ = must(store.replay(None));
        let trust_after_full = match must(store.get_memory_trust(fixture_memory_id(), 1, None)) {
            Some(value) => value,
            None => panic!("expected trust snapshot after full replay"),
        };

        let mut incremental_events = Vec::new();
        for index in 750..900 {
            let event_type = if index % 17 == 0 {
                OutcomeEventType::Failure
            } else if index % 9 == 0 {
                OutcomeEventType::Ignored
            } else if index % 14 == 0 {
                OutcomeEventType::Unknown
            } else {
                OutcomeEventType::Success
            };
            let inserted = must(store.append_event(&fixture_event_input(event_type)));
            incremental_events.push(inserted);
        }

        let first_incremental_seq = match incremental_events.first() {
            Some(event) => event.event_seq,
            None => panic!("expected incremental events"),
        };
        let _ = must(store.replay(Some(first_incremental_seq)));
        let trust_after_incremental =
            match must(store.get_memory_trust(fixture_memory_id(), 1, None)) {
                Some(value) => value,
                None => panic!("expected trust snapshot after incremental replay"),
            };

        let mut fresh = fixture_store();
        seed_memory_row(&fresh);
        for item in baseline_events
            .into_iter()
            .chain(incremental_events.into_iter())
        {
            let mut input = fixture_event_input(item.event_type);
            input.event_id = Some(item.event_id);
            input.occurred_at = item.occurred_at;
            input.edited = item.edited;
            input.escalated = item.escalated;
            input.severity = item.severity;
            input.manual_confidence = item.manual_confidence;
            input.override_cap = item.override_cap;
            input.payload_json = item.payload_json;
            let _ = must(fresh.append_event(&input));
        }

        let _ = must(fresh.replay(None));
        let trust_fresh = match must(fresh.get_memory_trust(fixture_memory_id(), 1, None)) {
            Some(value) => value,
            None => panic!("expected trust snapshot after fresh replay"),
        };

        assert_eq!(trust_after_incremental.memory_id, trust_fresh.memory_id);
        assert_eq!(trust_after_incremental.version, trust_fresh.version);
        assert_eq!(
            trust_after_incremental.confidence_raw,
            trust_fresh.confidence_raw
        );
        assert_eq!(
            trust_after_incremental.confidence_effective,
            trust_fresh.confidence_effective
        );
        assert_eq!(
            trust_after_incremental.baseline_confidence,
            trust_fresh.baseline_confidence
        );
        assert_eq!(
            trust_after_incremental.trust_status,
            trust_fresh.trust_status
        );
        assert_eq!(
            trust_after_incremental.contradiction_cap_active,
            trust_fresh.contradiction_cap_active
        );
        assert_eq!(trust_after_incremental.cap_value, trust_fresh.cap_value);
        assert_eq!(
            trust_after_incremental.manual_override_active,
            trust_fresh.manual_override_active
        );
        assert_eq!(trust_after_incremental.wins_last5, trust_fresh.wins_last5);
        assert_eq!(
            trust_after_incremental.failures_last5,
            trust_fresh.failures_last5
        );
        assert_eq!(
            trust_after_incremental.last_event_seq,
            trust_fresh.last_event_seq
        );
        assert_eq!(
            trust_after_incremental.last_scored_at,
            trust_fresh.last_scored_at
        );
        assert!(trust_after_full.last_event_seq < trust_after_incremental.last_event_seq);
    }

    #[test]
    fn scale_guardrails_append_and_replay_within_budget() {
        let mut store = fixture_store();
        seed_memory_row(&store);

        let event_count = std::env::var("OUTCOME_PERF_EVENT_COUNT")
            .ok()
            .and_then(|raw| raw.parse::<usize>().ok())
            .unwrap_or(2_000);
        let append_budget_ms = std::env::var("OUTCOME_APPEND_BUDGET_MS")
            .ok()
            .and_then(|raw| raw.parse::<u128>().ok())
            .unwrap_or(20_000);
        let replay_budget_ms = std::env::var("OUTCOME_REPLAY_BUDGET_MS")
            .ok()
            .and_then(|raw| raw.parse::<u128>().ok())
            .unwrap_or(10_000);

        let append_start = Instant::now();
        for index in 0..event_count {
            let event_type = if index % 17 == 0 {
                OutcomeEventType::Failure
            } else if index % 11 == 0 {
                OutcomeEventType::Ignored
            } else if index % 13 == 0 {
                OutcomeEventType::Unknown
            } else {
                OutcomeEventType::Success
            };
            let _ = must(store.append_event(&fixture_event_input(event_type)));
        }
        let append_elapsed_ms = append_start.elapsed().as_millis();
        assert!(
            append_elapsed_ms <= append_budget_ms,
            "append performance budget exceeded: elapsed={}ms budget={}ms event_count={}",
            append_elapsed_ms,
            append_budget_ms,
            event_count
        );

        let replay_start = Instant::now();
        let _ = must(store.replay(None));
        let replay_elapsed_ms = replay_start.elapsed().as_millis();
        assert!(
            replay_elapsed_ms <= replay_budget_ms,
            "replay performance budget exceeded: elapsed={}ms budget={}ms event_count={}",
            replay_elapsed_ms,
            replay_budget_ms,
            event_count
        );

        let trust = match must(store.get_memory_trust(fixture_memory_id(), 1, None)) {
            Some(value) => value,
            None => panic!("expected trust snapshot after replay"),
        };
        assert!(trust.last_event_seq > 0);
    }

    #[test]
    fn benchmark_harness_generates_report_and_respects_thresholds() {
        let store = fixture_store();
        let config = BenchmarkConfig {
            volumes: vec![25, 75],
            repetitions: 2,
        };
        let thresholds = BenchmarkThresholds {
            append_p95_ms_max: 1000.0,
            replay_p95_ms_max: 2000.0,
            gate_p95_ms_max: 1000.0,
        };

        let report = must(store.run_benchmark(&config, Some(thresholds.clone())));
        assert_eq!(report.contract_version, "benchmark_report.v1");
        assert_eq!(report.repetitions, 2);
        assert_eq!(report.volumes.len(), 2);
        assert_eq!(report.thresholds, Some(thresholds));
        assert!(report.within_thresholds);
    }

    #[test]
    fn replay_is_idempotent_on_repeated_partial_recovery() {
        let mut store = fixture_store();
        seed_memory_row(&store);

        let event_a = must(store.append_event(&fixture_event_input(OutcomeEventType::Success)));
        let _ = must(store.append_event(&fixture_event_input(OutcomeEventType::Failure)));
        let _ = must(store.append_event(&fixture_event_input(OutcomeEventType::Success)));

        let _ = must(store.replay(Some(event_a.event_seq)));
        let trust_after_first = match must(store.get_memory_trust(fixture_memory_id(), 1, None)) {
            Some(value) => value,
            None => panic!("expected trust after first recovery replay"),
        };

        let _ = must(store.replay(Some(event_a.event_seq)));
        let trust_after_second = match must(store.get_memory_trust(fixture_memory_id(), 1, None)) {
            Some(value) => value,
            None => panic!("expected trust after second recovery replay"),
        };

        assert_eq!(
            trust_after_first.confidence_raw,
            trust_after_second.confidence_raw
        );
        assert_eq!(
            trust_after_first.confidence_effective,
            trust_after_second.confidence_effective
        );
        assert_eq!(
            trust_after_first.last_event_seq,
            trust_after_second.last_event_seq
        );
        assert_eq!(trust_after_first.wins_last5, trust_after_second.wins_last5);
        assert_eq!(
            trust_after_first.failures_last5,
            trust_after_second.failures_last5
        );
    }

    #[test]
    fn version_isolation_prevents_cross_version_trust_smear() {
        let store = fixture_store();
        let memory_id = fixture_memory_id();
        must(seed_minimal_memory_record(store.connection(), memory_id, 1));
        must(seed_minimal_memory_record(store.connection(), memory_id, 2));

        let mut store = store;
        for _ in 0..3 {
            let _ = must(store.append_event(&fixture_event_input_for(
                memory_id,
                1,
                1,
                OutcomeEventType::Success,
            )));
        }
        let mut inherited = fixture_event_input_for(memory_id, 2, 1, OutcomeEventType::Inherited);
        inherited.manual_confidence = Some(0.90);
        let _ = must(store.append_event(&inherited));
        let _ = must(store.append_event(&fixture_event_input_for(
            memory_id,
            2,
            1,
            OutcomeEventType::Failure,
        )));

        let _ = must(store.replay(None));
        let v1 = match must(store.get_memory_trust(memory_id, 1, None)) {
            Some(value) => value,
            None => panic!("missing v1 trust"),
        };
        let v2 = match must(store.get_memory_trust(memory_id, 2, None)) {
            Some(value) => value,
            None => panic!("missing v2 trust"),
        };

        assert_eq!(v1.trust_status, TrustStatus::Validated);
        assert_eq!(v1.wins_last5, 3);
        assert_eq!(v1.failures_last5, 0);
        assert_eq!(v2.trust_status, TrustStatus::Active);
        assert_eq!(v2.failures_last5, 1);
        assert!(v1.last_event_seq != v2.last_event_seq);
    }

    #[test]
    fn mixed_ruleset_versions_replay_deterministically_without_rewrite() {
        let mut store = fixture_store();
        seed_memory_row(&store);

        let mut ruleset_v2 = OutcomeRuleset::v1();
        ruleset_v2.ruleset_version = 2;
        ruleset_v2.alpha = 0.12;
        ruleset_v2.failure_weight = -1.5;
        must(store.upsert_ruleset(&ruleset_v2));

        let _ = must(store.append_event(&fixture_event_input_for(
            fixture_memory_id(),
            1,
            1,
            OutcomeEventType::Success,
        )));
        let _ = must(store.append_event(&fixture_event_input_for(
            fixture_memory_id(),
            1,
            1,
            OutcomeEventType::Success,
        )));
        let _ = must(store.append_event(&fixture_event_input_for(
            fixture_memory_id(),
            1,
            2,
            OutcomeEventType::Failure,
        )));
        let _ = must(store.replay(None));

        let events = must(store.list_events_for_key(fixture_memory_id(), 1, None));
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].ruleset_version, 1);
        assert_eq!(events[1].ruleset_version, 1);
        assert_eq!(events[2].ruleset_version, 2);

        let trust_mixed = match must(store.get_memory_trust(fixture_memory_id(), 1, None)) {
            Some(value) => value,
            None => panic!("missing mixed trust"),
        };

        let mut fresh = fixture_store();
        seed_memory_row(&fresh);
        must(fresh.upsert_ruleset(&ruleset_v2));
        for event in events {
            let mut input = fixture_event_input_for(
                fixture_memory_id(),
                event.version,
                event.ruleset_version,
                event.event_type,
            );
            input.event_id = Some(event.event_id);
            input.occurred_at = event.occurred_at;
            let _ = must(fresh.append_event(&input));
        }
        let _ = must(fresh.replay(None));
        let trust_fresh = match must(fresh.get_memory_trust(fixture_memory_id(), 1, None)) {
            Some(value) => value,
            None => panic!("missing fresh mixed trust"),
        };

        assert_eq!(trust_mixed.confidence_raw, trust_fresh.confidence_raw);
        assert_eq!(
            trust_mixed.confidence_effective,
            trust_fresh.confidence_effective
        );
        assert_eq!(trust_mixed.last_event_seq, trust_fresh.last_event_seq);
    }

    #[test]
    fn invalid_ruleset_json_is_reported_clearly() {
        let store = fixture_store();
        let inserted = store.connection().execute(
            "INSERT INTO outcome_rulesets(ruleset_version, ruleset_json, created_at)
             VALUES (?1, ?2, ?3)",
            params![99_i64, "not-json", "2026-02-07T00:00:00Z"],
        );
        if let Err(err) = inserted {
            panic!("failed to insert invalid ruleset fixture: {err}");
        }

        let err = match store.get_rulesets() {
            Ok(_) => panic!("expected get_rulesets failure on invalid ruleset JSON"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("invalid stored ruleset JSON"));
    }

    #[test]
    fn invalid_event_timestamp_is_reported_clearly() {
        let store = fixture_store();
        seed_memory_row(&store);

        let inserted = store.connection().execute(
            "INSERT INTO outcome_events(
                event_id, ruleset_version, memory_id, version, event_type,
                occurred_at, recorded_at, writer, justification, context_id,
                edited, escalated, severity, manual_confidence, override_cap, payload_json
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15, ?16
             )",
            params![
                Ulid::new().to_string(),
                1_i64,
                fixture_memory_id().to_string(),
                1_i64,
                "success",
                "bad-timestamp",
                "2026-02-07T00:00:00Z",
                "tester",
                "fixture",
                "ctx",
                0_i64,
                0_i64,
                Option::<String>::None,
                Option::<f64>::None,
                0_i64,
                "{}",
            ],
        );
        if let Err(err) = inserted {
            panic!("failed to insert invalid event fixture: {err}");
        }

        let err = match store.list_events_for_key(fixture_memory_id(), 1, None) {
            Ok(_) => panic!("expected list_events_for_key failure on invalid timestamp"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("invalid RFC3339 timestamp"));
    }

    #[test]
    fn schema_contract_contains_expected_tables_columns_and_triggers() {
        let store = fixture_store();
        let table_check = must(table_exists(store.connection(), "outcome_events"));
        assert!(table_check);
        let table_check = must(table_exists(store.connection(), "memory_trust"));
        assert!(table_check);
        let table_check = must(table_exists(store.connection(), "outcome_rulesets"));
        assert!(table_check);
        let table_check = must(table_exists(store.connection(), "outcome_projection_state"));
        assert!(table_check);

        must(ensure_table_has_columns(
            store.connection(),
            "outcome_events",
            &[
                "event_seq",
                "event_id",
                "ruleset_version",
                "memory_id",
                "version",
                "event_type",
                "occurred_at",
                "recorded_at",
                "writer",
                "justification",
            ],
        ));
        must(ensure_table_has_columns(
            store.connection(),
            "memory_trust",
            &[
                "memory_id",
                "version",
                "confidence_raw",
                "confidence_effective",
                "trust_status",
                "last_event_seq",
            ],
        ));

        let trigger_count = match store.connection().query_row(
            "SELECT COUNT(*) FROM sqlite_master
             WHERE type='trigger'
               AND name IN ('trg_outcome_events_no_update', 'trg_outcome_events_no_delete')",
            [],
            |row| row.get::<_, i64>(0),
        ) {
            Ok(value) => value,
            Err(err) => panic!("failed to query trigger contract: {err}"),
        };
        assert_eq!(trigger_count, 2);
    }

    #[test]
    fn migration_is_idempotent_and_preserves_existing_data() {
        let mut store = fixture_store();
        seed_memory_row(&store);
        let _ = must(store.append_event(&fixture_event_input(OutcomeEventType::Success)));
        let _ = must(store.replay(None));
        let trust_before = match must(store.get_memory_trust(fixture_memory_id(), 1, None)) {
            Some(value) => value,
            None => panic!("expected trust before second migrate"),
        };

        must(store.migrate());
        let trust_after = match must(store.get_memory_trust(fixture_memory_id(), 1, None)) {
            Some(value) => value,
            None => panic!("expected trust after second migrate"),
        };
        assert_eq!(trust_before.last_event_seq, trust_after.last_event_seq);
        assert_eq!(trust_before.confidence_raw, trust_after.confidence_raw);
    }

    #[test]
    fn sqlite_busy_timeout_allows_append_after_lock_release() {
        let db_path =
            std::env::temp_dir().join(format!("outcome-lock-test-{}.sqlite3", Ulid::new()));

        let setup_store = must(SqliteOutcomeStore::open(&db_path));
        must(seed_minimal_memory_record(
            setup_store.connection(),
            fixture_memory_id(),
            1,
        ));
        must(setup_store.migrate());
        drop(setup_store);

        let lock_conn = match Connection::open(&db_path) {
            Ok(value) => value,
            Err(err) => panic!("failed to open lock connection: {err}"),
        };
        if let Err(err) = lock_conn.execute_batch("BEGIN IMMEDIATE;") {
            panic!("failed to acquire write lock: {err}");
        }

        let append_path = db_path.clone();
        let append_handle = std::thread::spawn(move || {
            let mut append_store = match SqliteOutcomeStore::open(&append_path) {
                Ok(value) => value,
                Err(err) => panic!("failed to open append store: {err}"),
            };
            append_store.append_event(&benchmark_event_input(
                fixture_memory_id(),
                1,
                1,
                OutcomeEventType::Success,
            ))
        });

        std::thread::sleep(std::time::Duration::from_millis(150));
        if let Err(err) = lock_conn.execute_batch("COMMIT;") {
            panic!("failed to release write lock: {err}");
        }

        let append_result = match append_handle.join() {
            Ok(result) => result,
            Err(err) => panic!("append thread join failed: {err:?}"),
        };
        assert!(
            append_result.is_ok(),
            "append should succeed after lock release: {:?}",
            append_result.err()
        );

        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn ready_to_integrate_gate_suite_passes() {
        let mut store = fixture_store();
        let memory_id = fixture_memory_id();
        must(seed_minimal_memory_record(store.connection(), memory_id, 1));
        must(seed_minimal_memory_record(store.connection(), memory_id, 2));

        let mut ruleset_v2 = OutcomeRuleset::v1();
        ruleset_v2.ruleset_version = 2;
        ruleset_v2.alpha = 0.10;
        must(store.upsert_ruleset(&ruleset_v2));

        let _ = must(store.append_event(&fixture_event_input_for(
            memory_id,
            1,
            1,
            OutcomeEventType::Success,
        )));
        let _ = must(store.append_event(&fixture_event_input_for(
            memory_id,
            1,
            1,
            OutcomeEventType::Success,
        )));
        let mut inherited = fixture_event_input_for(memory_id, 2, 1, OutcomeEventType::Inherited);
        inherited.manual_confidence = Some(0.90);
        let _ = must(store.append_event(&inherited));
        let _ = must(store.append_event(&fixture_event_input_for(
            memory_id,
            2,
            2,
            OutcomeEventType::Failure,
        )));

        let _ = must(store.replay(None));
        let check = must(store.projector_check());
        assert!(check.healthy);

        let trust_v1 = match must(store.get_memory_trust(memory_id, 1, None)) {
            Some(value) => value,
            None => panic!("missing v1 trust"),
        };
        let trust_v2 = match must(store.get_memory_trust(memory_id, 2, None)) {
            Some(value) => value,
            None => panic!("missing v2 trust"),
        };
        assert!(trust_v1.confidence_effective > trust_v2.confidence_effective);
        assert!(must(store.projector_stale_keys(None)).is_empty());
    }
}
