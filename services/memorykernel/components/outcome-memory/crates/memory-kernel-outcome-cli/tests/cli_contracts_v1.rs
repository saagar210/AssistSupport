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
        Err(err) => panic!("failed to run mk-outcome command {:?}: {err}", args),
    }
}

fn stdout_json(output: &Output) -> Value {
    match serde_json::from_slice::<Value>(&output.stdout) {
        Ok(value) => value,
        Err(err) => panic!(
            "failed to parse stdout as JSON: {err}\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    }
}

#[test]
fn outcome_help_contract_lists_expected_subcommands() {
    let output = match Command::new(mk_binary_path())
        .args(["outcome", "--help"])
        .output()
    {
        Ok(value) => value,
        Err(err) => panic!("failed to run help command: {err}"),
    };

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for required in [
        "log",
        "manual",
        "system",
        "trust",
        "replay",
        "benchmark",
        "projector",
        "gate",
        "events",
    ] {
        assert!(
            stdout.contains(required),
            "expected help output to contain subcommand {required}; output={stdout}"
        );
    }
}

#[test]
fn error_shape_for_missing_trust_snapshot_is_stable() {
    let db_path = std::env::temp_dir().join(format!(
        "outcome-contract-missing-trust-{}.sqlite3",
        Ulid::new()
    ));
    let memory_id = fixture_memory_id();

    let setup_conn = match Connection::open(&db_path) {
        Ok(value) => value,
        Err(err) => panic!("failed to open setup db: {err}"),
    };
    if let Err(err) = seed_minimal_memory_record(&setup_conn, memory_id, 1) {
        panic!("failed to seed memory row: {err}");
    }

    let output = mk_output(
        &db_path,
        &[
            "outcome",
            "trust",
            "show",
            "--memory-id",
            &memory_id.to_string(),
            "--version",
            "1",
        ],
    );
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("trust snapshot not found"),
        "expected stable error shape, got stderr={stderr}"
    );

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn benchmark_command_emits_contract_json_and_report_artifact() {
    let db_path =
        std::env::temp_dir().join(format!("outcome-contract-bench-{}.sqlite3", Ulid::new()));
    let report_path =
        std::env::temp_dir().join(format!("outcome-bench-report-{}.json", Ulid::new()));

    let output = mk_output(
        &db_path,
        &[
            "outcome",
            "benchmark",
            "run",
            "--volume",
            "10",
            "--volume",
            "20",
            "--repetitions",
            "2",
            "--append-p95-max-ms",
            "5000",
            "--replay-p95-max-ms",
            "5000",
            "--gate-p95-max-ms",
            "5000",
            "--output",
            report_path.to_str().unwrap_or(""),
            "--json",
        ],
    );
    assert!(
        output.status.success(),
        "benchmark command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload = stdout_json(&output);
    assert_eq!(
        payload["contract_version"],
        Value::String("benchmark_report.v1".to_string())
    );
    assert_eq!(payload["repetitions"], Value::Number(2_u64.into()));
    assert!(payload["volumes"].is_array());

    let file_text = match std::fs::read_to_string(&report_path) {
        Ok(value) => value,
        Err(err) => panic!("failed reading benchmark report artifact: {err}"),
    };
    let file_json: Value = match serde_json::from_str(&file_text) {
        Ok(value) => value,
        Err(err) => panic!("failed parsing benchmark report artifact json: {err}"),
    };
    assert_eq!(
        file_json["contract_version"],
        Value::String("benchmark_report.v1".to_string())
    );

    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(&report_path);
}

#[test]
fn benchmark_command_exits_non_zero_on_threshold_violation() {
    let db_path = std::env::temp_dir().join(format!(
        "outcome-contract-bench-violation-{}.sqlite3",
        Ulid::new()
    ));

    let output = mk_output(
        &db_path,
        &[
            "outcome",
            "benchmark",
            "run",
            "--volume",
            "25",
            "--repetitions",
            "1",
            "--append-p95-max-ms",
            "0",
            "--replay-p95-max-ms",
            "0",
            "--gate-p95-max-ms",
            "0",
            "--json",
        ],
    );
    assert!(
        !output.status.success(),
        "expected non-zero exit on threshold violation"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("benchmark thresholds violated"),
        "expected stable violation error shape, got stderr={stderr}"
    );

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn projector_json_contract_contains_versioned_payloads() {
    let db_path = std::env::temp_dir().join(format!(
        "outcome-contract-projector-{}.sqlite3",
        Ulid::new()
    ));
    let memory_id = fixture_memory_id();

    let setup_conn = match Connection::open(&db_path) {
        Ok(value) => value,
        Err(err) => panic!("failed to open setup db: {err}"),
    };
    if let Err(err) = seed_minimal_memory_record(&setup_conn, memory_id, 1) {
        panic!("failed to seed memory row: {err}");
    }

    let status_output = mk_output(&db_path, &["outcome", "projector", "status", "--json"]);
    assert!(status_output.status.success());
    let status_payload = stdout_json(&status_output);
    assert_eq!(
        status_payload["contract_version"],
        Value::String("projector_status.v1".to_string())
    );

    let check_output = mk_output(&db_path, &["outcome", "projector", "check", "--json"]);
    assert!(check_output.status.success());
    let check_payload = stdout_json(&check_output);
    assert_eq!(
        check_payload["contract_version"],
        Value::String("projector_check.v1".to_string())
    );

    let _ = std::fs::remove_file(&db_path);
}
