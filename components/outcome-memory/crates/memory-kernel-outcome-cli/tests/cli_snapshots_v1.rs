#![allow(clippy::single_match_else, clippy::uninlined_format_args)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use memory_kernel_core::MemoryId;
use memory_kernel_outcome_store_sqlite::seed_minimal_memory_record;
use rusqlite::Connection;
use serde_json::Value;
use ulid::Ulid;

fn fixture_memory_id() -> MemoryId {
    let parsed = match Ulid::from_string("01J0SQQP7M70P6Y3R4T8D8G8M2") {
        Ok(value) => value,
        Err(err) => panic!("invalid fixture ULID: {err}"),
    };
    MemoryId(parsed)
}

fn mk_binary_path() -> PathBuf {
    match std::env::var("CARGO_BIN_EXE_mk_outcome") {
        Ok(value) => PathBuf::from(value),
        Err(_) => {
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/mk-outcome");
            if !path.exists() {
                let status = Command::new("cargo")
                    .args([
                        "build",
                        "-p",
                        "memory-kernel-outcome-cli",
                        "--bin",
                        "mk-outcome",
                    ])
                    .status();
                match status {
                    Ok(value) if value.success() => {}
                    Ok(value) => panic!("failed to build mk-outcome binary (status={value})"),
                    Err(err) => panic!("failed to invoke cargo build: {err}"),
                }
            }
            path
        }
    }
}

fn mk_output(db_path: &Path, args: &[&str]) -> Output {
    let mut command = Command::new(mk_binary_path());
    command.arg("--db").arg(db_path);
    for arg in args {
        command.arg(arg);
    }

    match command.output() {
        Ok(output) => output,
        Err(err) => panic!("failed to execute mk-outcome command {:?}: {err}", args),
    }
}

fn parse_json(output: &Output) -> Value {
    match serde_json::from_slice::<Value>(&output.stdout) {
        Ok(value) => value,
        Err(err) => panic!(
            "failed to parse stdout json: {err}\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    }
}

#[test]
fn snapshot_projector_status_json_v1() {
    let db_path =
        std::env::temp_dir().join(format!("outcome-snapshot-status-{}.sqlite3", Ulid::new()));
    let memory_id = fixture_memory_id();

    let setup_conn = match Connection::open(&db_path) {
        Ok(value) => value,
        Err(err) => panic!("failed to open setup db: {err}"),
    };
    if let Err(err) = seed_minimal_memory_record(&setup_conn, memory_id, 1) {
        panic!("failed to seed memory row: {err}");
    }

    let output = mk_output(&db_path, &["outcome", "projector", "status", "--json"]);
    assert!(output.status.success());

    let mut payload = parse_json(&output);
    payload["updated_at"] = Value::String("<timestamp>".to_string());

    let snapshot = match serde_json::to_string_pretty(&payload) {
        Ok(value) => value,
        Err(err) => panic!("failed to serialize normalized status payload: {err}"),
    };

    let expected = r#"{
  "contract_version": "projector_status.v1",
  "projector_name": "trust_v0",
  "ruleset_version": 1,
  "projected_event_seq": 0,
  "latest_event_seq": 0,
  "lag_events": 0,
  "lag_delta_events": 0,
  "tracked_keys": 0,
  "trust_rows": 0,
  "stale_trust_rows": 0,
  "keys_with_events_no_trust_row": 0,
  "trust_rows_without_events": 0,
  "max_stale_seq_gap": 0,
  "updated_at": "<timestamp>"
}"#;

    assert_eq!(snapshot, expected);
    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn snapshot_projector_check_error_stderr_v1() {
    let db_path =
        std::env::temp_dir().join(format!("outcome-snapshot-check-{}.sqlite3", Ulid::new()));
    let memory_id = fixture_memory_id();

    let setup_conn = match Connection::open(&db_path) {
        Ok(value) => value,
        Err(err) => panic!("failed to open setup db: {err}"),
    };
    if let Err(err) = seed_minimal_memory_record(&setup_conn, memory_id, 1) {
        panic!("failed to seed memory row: {err}");
    }

    let log_output = mk_output(
        &db_path,
        &[
            "outcome",
            "log",
            "--memory-id",
            &memory_id.to_string(),
            "--version",
            "1",
            "--event",
            "success",
            "--writer",
            "tester",
            "--justification",
            "snapshot",
            "--occurred-at",
            "2026-02-07T12:00:00Z",
        ],
    );
    assert!(log_output.status.success());

    let check_output = mk_output(&db_path, &["outcome", "projector", "check", "--json"]);
    assert!(!check_output.status.success());

    let stderr = String::from_utf8_lossy(&check_output.stderr).to_string();
    assert_eq!(
        stderr,
        "Error: projector consistency check failed: projection_lag:projection lag detected: 1 events behind; stale_trust_rows:stale trust rows detected: 1 keys out of date; key_snapshot_mismatch:key/snapshot mismatch: tracked_keys=1 trust_rows=0\n"
    );

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn snapshot_gate_preview_json_v1() {
    let db_path =
        std::env::temp_dir().join(format!("outcome-snapshot-gate-{}.sqlite3", Ulid::new()));
    let memory_id = fixture_memory_id();

    let setup_conn = match Connection::open(&db_path) {
        Ok(value) => value,
        Err(err) => panic!("failed to open setup db: {err}"),
    };
    if let Err(err) = seed_minimal_memory_record(&setup_conn, memory_id, 1) {
        panic!("failed to seed memory row: {err}");
    }

    for _ in 0..3 {
        let output = mk_output(
            &db_path,
            &[
                "outcome",
                "log",
                "--memory-id",
                &memory_id.to_string(),
                "--version",
                "1",
                "--event",
                "success",
                "--writer",
                "tester",
                "--justification",
                "snapshot",
                "--occurred-at",
                "2026-02-07T12:00:00Z",
            ],
        );
        assert!(output.status.success());
    }

    let replay_output = mk_output(&db_path, &["outcome", "replay"]);
    assert!(replay_output.status.success());

    let gate_output = mk_output(
        &db_path,
        &[
            "outcome",
            "gate",
            "preview",
            "--mode",
            "safe",
            "--as-of",
            "2026-02-07T12:00:00Z",
            "--context-id",
            "ctx-1",
            "--candidate",
            &format!("{}:1", memory_id),
            "--json",
        ],
    );
    assert!(gate_output.status.success());

    let snapshot = match serde_json::to_string_pretty(&parse_json(&gate_output)) {
        Ok(value) => value,
        Err(err) => panic!("failed to serialize gate payload: {err}"),
    };

    let expected = format!(
        "{{\n  \"contract_version\": \"gate_preview.v1\",\n  \"mode\": \"safe\",\n  \"as_of\": \"2026-02-07T12:00:00Z\",\n  \"context_id\": \"ctx-1\",\n  \"candidates\": [\n    {{\n      \"memory_id\": \"{}\",\n      \"version\": 1\n    }}\n  ],\n  \"decisions\": [\n    {{\n      \"memory_id\": \"{}\",\n      \"version\": 1,\n      \"include\": true,\n      \"confidence_effective\": 0.73052734,\n      \"trust_status\": \"validated\",\n      \"capped\": false,\n      \"reason_codes\": [\n        \"included.safe.validated_threshold\"\n      ]\n    }}\n  ]\n}}",
        memory_id,
        memory_id
    );

    assert_eq!(snapshot, expected);

    let _ = std::fs::remove_file(&db_path);
}
