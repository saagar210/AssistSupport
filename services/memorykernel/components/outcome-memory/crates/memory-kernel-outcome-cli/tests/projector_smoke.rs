#![allow(
    clippy::single_match_else,
    clippy::match_wild_err_arm,
    clippy::manual_let_else,
    clippy::uninlined_format_args
)]

use std::path::Path;
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

fn mk_output(db_path: &Path, args: &[&str]) -> Output {
    let binary = match std::env::var("CARGO_BIN_EXE_mk_outcome") {
        Ok(value) => value,
        Err(_) => {
            let workspace_binary =
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/mk-outcome");
            if !workspace_binary.exists() {
                let build_status = Command::new("cargo")
                    .args([
                        "build",
                        "-p",
                        "memory-kernel-outcome-cli",
                        "--bin",
                        "mk-outcome",
                    ])
                    .status();
                match build_status {
                    Ok(status) if status.success() => {}
                    Ok(status) => {
                        panic!(
                            "failed building mk-outcome binary for smoke tests (status={status})"
                        )
                    }
                    Err(err) => panic!("failed invoking cargo build for smoke tests: {err}"),
                }
            }
            match workspace_binary.into_os_string().into_string() {
                Ok(value) => value,
                Err(_) => panic!("workspace mk-outcome binary path is not valid UTF-8"),
            }
        }
    };

    let mut command = Command::new(binary);
    command.arg("--db").arg(db_path);
    for arg in args {
        command.arg(arg);
    }

    match command.output() {
        Ok(output) => output,
        Err(err) => panic!("failed to execute mk-outcome command {:?}: {err}", args),
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
fn projector_check_smoke_after_replay() {
    let db_path =
        std::env::temp_dir().join(format!("outcome-projector-smoke-{}.sqlite3", Ulid::new()));
    let memory_id = fixture_memory_id();

    let connection = match Connection::open(&db_path) {
        Ok(value) => value,
        Err(err) => panic!("failed to open setup sqlite db: {err}"),
    };
    if let Err(err) = seed_minimal_memory_record(&connection, memory_id, 1) {
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
                "smoke",
                "--occurred-at",
                "2026-02-07T12:00:00Z",
            ],
        );
        assert!(
            output.status.success(),
            "log command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let check_before = mk_output(&db_path, &["outcome", "projector", "check", "--json"]);
    assert!(
        !check_before.status.success(),
        "projector check should fail before replay"
    );
    let check_before_json = stdout_json(&check_before);
    assert_eq!(check_before_json["healthy"], Value::Bool(false));

    let stale_before = mk_output(&db_path, &["outcome", "projector", "stale-keys", "--json"]);
    assert!(
        stale_before.status.success(),
        "stale-keys command failed: {}",
        String::from_utf8_lossy(&stale_before.stderr)
    );
    let stale_before_json = stdout_json(&stale_before);
    let stale_before_list = match stale_before_json.as_array() {
        Some(value) => value,
        None => panic!("expected stale-keys response to be an array"),
    };
    assert_eq!(stale_before_list.len(), 1);

    let replay_output = mk_output(&db_path, &["outcome", "replay"]);
    assert!(
        replay_output.status.success(),
        "replay command failed: {}",
        String::from_utf8_lossy(&replay_output.stderr)
    );

    let check_after = mk_output(&db_path, &["outcome", "projector", "check", "--json"]);
    assert!(
        check_after.status.success(),
        "projector check should pass after replay: {}",
        String::from_utf8_lossy(&check_after.stderr)
    );
    let check_after_json = stdout_json(&check_after);
    assert_eq!(check_after_json["healthy"], Value::Bool(true));

    let stale_after = mk_output(&db_path, &["outcome", "projector", "stale-keys", "--json"]);
    assert!(
        stale_after.status.success(),
        "stale-keys command failed after replay: {}",
        String::from_utf8_lossy(&stale_after.stderr)
    );
    let stale_after_json = stdout_json(&stale_after);
    let stale_after_list = match stale_after_json.as_array() {
        Some(value) => value,
        None => panic!("expected stale-keys response to be an array"),
    };
    assert!(stale_after_list.is_empty());

    let _ = std::fs::remove_file(&db_path);
}
