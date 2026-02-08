use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use memory_kernel_api::{AddConstraintRequest, AddSummaryRequest, MemoryKernelApi};
use memory_kernel_core::{Authority, ConstraintEffect, RecordType, TruthStatus};
use multi_agent_center_domain::RunId;
use multi_agent_center_trace_core::TraceStore;
use multi_agent_center_trace_sqlite::SqliteTraceStore;
use rusqlite::Connection;
use ulid::Ulid;

type SelectedSignatureItem = (usize, u32, RecordType, Vec<String>);
type SelectedSignature = Vec<Vec<SelectedSignatureItem>>;

fn temp_path(name: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!("mac-cli-test-{}-{}.{}", name, Ulid::new(), ext))
}

fn extract_run_id(stdout: &str) -> Option<RunId> {
    for token in stdout.split_whitespace() {
        if let Some(raw) = token.strip_prefix("run_id=") {
            let parsed = Ulid::from_string(raw).ok()?;
            return Some(RunId(parsed));
        }
    }
    None
}

fn extract_replay_run_id(stdout: &str) -> Option<RunId> {
    for token in stdout.split_whitespace() {
        if let Some(raw) = token.strip_prefix("replay_run_id=") {
            let parsed = Ulid::from_string(raw).ok()?;
            return Some(RunId(parsed));
        }
    }
    None
}

fn extract_selected_signature(trace_store: &SqliteTraceStore, run_id: RunId) -> SelectedSignature {
    let packages = trace_store
        .get_step_context_packages(run_id)
        .unwrap_or_else(|err| panic!("failed to load step context packages: {err}"));
    packages
        .iter()
        .map(|row| {
            row.envelope
                .context_package
                .selected_items
                .iter()
                .map(|item| {
                    (
                        item.rank,
                        item.version,
                        item.record_type,
                        item.why.reasons.clone(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn seed_memory_kernel_db(memory_db: &Path) {
    let api = MemoryKernelApi::new(memory_db.to_path_buf());
    assert!(api.migrate(false).is_ok());
    assert!(api
        .add_constraint(AddConstraintRequest {
            actor: "dev".to_string(),
            action: "read".to_string(),
            resource: "repo".to_string(),
            effect: ConstraintEffect::Allow,
            note: None,
            memory_id: None,
            version: 1,
            writer: "test".to_string(),
            justification: "seed constraint".to_string(),
            source_uri: "file:///constraint.md".to_string(),
            source_hash: None,
            evidence: Vec::new(),
            confidence: Some(0.9),
            truth_status: TruthStatus::Observed,
            authority: Authority::Authoritative,
            created_at: None,
            effective_at: None,
            supersedes: Vec::new(),
            contradicts: Vec::new(),
        })
        .is_ok());
    assert!(api
        .add_summary(AddSummaryRequest {
            record_type: RecordType::Decision,
            summary: "repo policy decision".to_string(),
            memory_id: None,
            version: 1,
            writer: "test".to_string(),
            justification: "seed decision".to_string(),
            source_uri: "file:///decision.md".to_string(),
            source_hash: None,
            evidence: Vec::new(),
            confidence: Some(0.8),
            truth_status: TruthStatus::Observed,
            authority: Authority::Derived,
            created_at: None,
            effective_at: None,
            supersedes: Vec::new(),
            contradicts: Vec::new(),
        })
        .is_ok());
}

fn write_policy_recall_workflow(workflow_path: &Path, workflow_name: &str) {
    let workflow_yaml = format!(
        r#"
workflow_name: {workflow_name}
workflow_version: v1
normalization_version: 0
agents:
  - agent_name: analyst
    role: analysis
    provider:
      provider_name: mock
      model_id: mock-model-v1
steps:
  - step_key: step_mem
    agent_name: analyst
    task:
      context_queries:
        - mode: policy
          text: "Can dev read repo?"
          actor: "dev"
          action: "read"
          resource: "repo"
        - mode: recall
          text: "repo policy decision"
          record_types: [decision]
    depends_on: []
    gate_points: []
gates: []
defaults:
  non_interactive: true
"#
    );
    assert!(fs::write(workflow_path, workflow_yaml).is_ok());
}

#[test]
#[allow(clippy::too_many_lines)]
fn run_with_memory_db_uses_api_policy_and_recall_queries() {
    let memory_db_first = temp_path("memory-first", "sqlite");
    let memory_db_second = temp_path("memory-second", "sqlite");
    let trace_db_first = temp_path("trace-first", "sqlite");
    let trace_db_second = temp_path("trace-second", "sqlite");
    let workflow_path = temp_path("workflow", "yaml");

    seed_memory_kernel_db(&memory_db_first);
    seed_memory_kernel_db(&memory_db_second);
    write_policy_recall_workflow(&workflow_path, "cli_api_flow");

    let as_of = "2026-02-07T00:00:00Z";

    let output = Command::new(env!("CARGO_BIN_EXE_multi-agent-center-cli"))
        .arg("run")
        .arg("--workflow")
        .arg(&workflow_path)
        .arg("--trace-db")
        .arg(&trace_db_first)
        .arg("--memory-db")
        .arg(&memory_db_first)
        .arg("--as-of")
        .arg(as_of)
        .arg("--non-interactive")
        .output();
    assert!(output.is_ok());
    let output = output.unwrap_or_else(|_| unreachable!());
    assert!(
        output.status.success(),
        "stdout={}; stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(stdout.contains("status=succeeded"));

    let run_id = extract_run_id(&stdout);
    assert!(run_id.is_some());
    let run_id = run_id.unwrap_or_else(|| unreachable!());

    let trace_store_first = SqliteTraceStore::open(&trace_db_first);
    assert!(trace_store_first.is_ok());
    let trace_store_first = trace_store_first.unwrap_or_else(|_| unreachable!());
    let packages = trace_store_first.get_step_context_packages(run_id);
    assert!(packages.is_ok());
    let packages = packages.unwrap_or_else(|_| unreachable!());
    assert_eq!(packages.len(), 2);

    assert_eq!(packages[0].envelope.context_package.query.actor, "dev");
    assert_eq!(packages[0].envelope.context_package.query.action, "read");
    assert_eq!(packages[0].envelope.context_package.query.resource, "repo");
    assert!(packages[1]
        .envelope
        .context_package
        .selected_items
        .iter()
        .all(|item| item.record_type == RecordType::Decision));

    let first_signature = extract_selected_signature(&trace_store_first, run_id);

    let rerun = Command::new(env!("CARGO_BIN_EXE_multi-agent-center-cli"))
        .arg("run")
        .arg("--workflow")
        .arg(&workflow_path)
        .arg("--trace-db")
        .arg(&trace_db_second)
        .arg("--memory-db")
        .arg(&memory_db_second)
        .arg("--as-of")
        .arg(as_of)
        .arg("--non-interactive")
        .output();
    assert!(rerun.is_ok());
    let rerun = rerun.unwrap_or_else(|_| unreachable!());
    assert!(
        rerun.status.success(),
        "stdout={}; stderr={}",
        String::from_utf8_lossy(&rerun.stdout),
        String::from_utf8_lossy(&rerun.stderr)
    );
    let rerun_stdout = String::from_utf8_lossy(&rerun.stdout).to_string();
    let second_run_id = extract_run_id(&rerun_stdout)
        .unwrap_or_else(|| panic!("failed to parse second run_id from output: {rerun_stdout}"));
    let trace_store_second = SqliteTraceStore::open(&trace_db_second);
    assert!(trace_store_second.is_ok());
    let trace_store_second = trace_store_second.unwrap_or_else(|_| unreachable!());
    let second_signature = extract_selected_signature(&trace_store_second, second_run_id);

    assert_eq!(
        first_signature, second_signature,
        "policy+recall selected ordering/reasons must be deterministic across repeated API-backed runs"
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn replay_rerun_keeps_trust_memory_ref_identity_fields() {
    let memory_db = temp_path("memory-replay", "sqlite");
    let trace_db = temp_path("trace-replay", "sqlite");
    let workflow_path = temp_path("workflow-replay", "yaml");
    let as_of = "2026-02-07T00:00:00Z";

    seed_memory_kernel_db(&memory_db);
    write_policy_recall_workflow(&workflow_path, "replay_identity");

    let run_output = Command::new(env!("CARGO_BIN_EXE_multi-agent-center-cli"))
        .arg("run")
        .arg("--workflow")
        .arg(&workflow_path)
        .arg("--trace-db")
        .arg(&trace_db)
        .arg("--memory-db")
        .arg(&memory_db)
        .arg("--as-of")
        .arg(as_of)
        .arg("--non-interactive")
        .output();
    assert!(run_output.is_ok());
    let run_output = run_output.unwrap_or_else(|_| unreachable!());
    assert!(run_output.status.success());
    let run_stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
    let source_run_id = extract_run_id(&run_stdout)
        .unwrap_or_else(|| panic!("failed to parse source run id: {run_stdout}"));

    let replay_output = Command::new(env!("CARGO_BIN_EXE_multi-agent-center-cli"))
        .arg("replay")
        .arg("--trace-db")
        .arg(&trace_db)
        .arg("--run-id")
        .arg(source_run_id.to_string())
        .arg("--rerun-provider")
        .output();
    assert!(replay_output.is_ok());
    let replay_output = replay_output.unwrap_or_else(|_| unreachable!());
    assert!(replay_output.status.success());
    let replay_stdout = String::from_utf8_lossy(&replay_output.stdout).to_string();
    let replay_run_id = extract_replay_run_id(&replay_stdout)
        .unwrap_or_else(|| panic!("failed to parse replay run id: {replay_stdout}"));

    let conn = Connection::open(&trace_db)
        .unwrap_or_else(|err| panic!("failed to open trace sqlite {}: {err}", trace_db.display()));
    for run_id in [source_run_id, replay_run_id] {
        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM step_gate_decisions
                 WHERE run_id = ?1 AND gate_kind = 'trust' AND subject_type = 'memory_ref'",
                rusqlite::params![run_id.to_string()],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| unreachable!());
        let missing_identity: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM step_gate_decisions
                 WHERE run_id = ?1
                   AND gate_kind = 'trust'
                   AND subject_type = 'memory_ref'
                   AND (memory_id IS NULL OR version IS NULL OR memory_version_id IS NULL)",
                rusqlite::params![run_id.to_string()],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| unreachable!());
        assert!(
            total > 0,
            "expected trust memory_ref decisions for run {run_id}",
        );
        assert_eq!(
            missing_identity, 0,
            "run {run_id} has trust memory_ref rows without full identity",
        );
    }
}
