use std::ffi::OsStr;
use std::fs;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use jsonschema::JSONSchema;
use serde_json::Value;

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|err| panic!("clock should be >= UNIX_EPOCH: {err}"))
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{prefix}-{now}"));
    fs::create_dir_all(&dir)
        .unwrap_or_else(|err| panic!("failed to create temp dir {}: {err}", dir.display()));
    dir
}

fn run_mk<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(env!("CARGO_BIN_EXE_mk"))
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("failed to execute mk binary: {err}"))
}

fn run_json<I, S>(args: I) -> Value
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = run_mk(args);
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "mk command failed (status={}):\nstdout:\n{}\nstderr:\n{}",
            output.status, stdout, stderr
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    serde_json::from_str(&stdout)
        .unwrap_or_else(|err| panic!("stdout is not valid JSON: {err}\nstdout:\n{stdout}"))
}

fn as_i64(value: &Value, key: &str) -> i64 {
    value
        .get(key)
        .and_then(Value::as_i64)
        .unwrap_or_else(|| panic!("missing integer field `{key}` in payload: {value}"))
}

fn as_str<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing string field `{key}` in payload: {value}"))
}

fn path_str(path: &Path) -> &str {
    path.to_str().unwrap_or_else(|| panic!("path should be valid UTF-8: {}", path.display()))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|err| panic!("failed to canonicalize repo root: {err}"))
}

fn read_json_file(path: &Path) -> Value {
    let body = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read JSON file {}: {err}", path.display()));
    serde_json::from_str(&body)
        .unwrap_or_else(|err| panic!("failed to parse JSON file {}: {err}", path.display()))
}

fn validate_schema(schema_file: &str, instance: &Value) {
    let schema_path = repo_root().join("contracts/v1/schemas").join(schema_file);
    let schema_json = read_json_file(&schema_path);
    let compiled = JSONSchema::compile(&schema_json)
        .unwrap_or_else(|err| panic!("failed to compile schema {}: {err}", schema_path.display()));

    let errors = compiled
        .validate(instance)
        .err()
        .map(|iter| iter.map(|err| err.to_string()).collect::<Vec<_>>());
    if let Some(errors) = errors {
        panic!("schema validation failed for {}:\n{}", schema_file, errors.join("\n"));
    }
}

fn validate_schema_in(schema_dir: &Path, schema_file: &str, instance: &Value) {
    let schema_path = schema_dir.join(schema_file);
    let schema_json = read_json_file(&schema_path);
    let compiled = JSONSchema::compile(&schema_json)
        .unwrap_or_else(|err| panic!("failed to compile schema {}: {err}", schema_path.display()));

    let errors = compiled
        .validate(instance)
        .err()
        .map(|iter| iter.map(|err| err.to_string()).collect::<Vec<_>>());
    if let Some(errors) = errors {
        panic!("schema validation failed for {}:\n{}", schema_file, errors.join("\n"));
    }
}

fn normalize_for_golden(value: &mut Value) {
    const DYNAMIC_TIME_FIELDS: [&str; 4] = ["generated_at", "as_of", "created_at", "effective_at"];
    const DYNAMIC_ID_FIELDS: [&str; 2] = ["memory_id", "memory_version_id"];

    match value {
        Value::Object(object) => {
            for (key, child) in object.iter_mut() {
                if key == "context_package_id" {
                    *child = Value::String("<context_package_id>".to_string());
                    continue;
                }
                if key == "snapshot_id" {
                    *child = Value::String("<snapshot_id>".to_string());
                    continue;
                }
                if DYNAMIC_TIME_FIELDS.contains(&key.as_str()) {
                    *child = Value::String("<rfc3339>".to_string());
                    continue;
                }
                if DYNAMIC_ID_FIELDS.contains(&key.as_str()) {
                    *child = Value::String("<ulid>".to_string());
                    continue;
                }
                if key == "confidence" && child.is_number() {
                    *child = Value::String("<confidence>".to_string());
                    continue;
                }
                normalize_for_golden(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                normalize_for_golden(item);
            }
        }
        _ => {}
    }
}

fn assert_golden_matches(fixture_name: &str, mut actual: Value) {
    normalize_for_golden(&mut actual);
    let fixture_path = repo_root().join("contracts/v1/fixtures").join(fixture_name);
    let expected = read_json_file(&fixture_path);
    assert_eq!(actual, expected);
}

// Test IDs: TCLI-005, TCLI-006
#[allow(clippy::too_many_lines)]
#[test]
fn db_commands_cover_migrate_integrity_backup_restore_export_import() {
    let sandbox = unique_temp_dir("memorykernel-cli-step8-db");
    let db_a = sandbox.join("a.sqlite3");
    let db_b = sandbox.join("b.sqlite3");
    let export_dir = sandbox.join("export");
    let backup_file = sandbox.join("backup.sqlite3");

    let schema_before = run_json(["--db", path_str(&db_a), "db", "schema-version"]);
    assert_eq!(as_i64(&schema_before, "current_version"), 0);

    let dry_run = run_json(["--db", path_str(&db_a), "db", "migrate", "--dry-run"]);
    assert_eq!(as_i64(&dry_run, "current_version"), 0);
    assert_eq!(
        dry_run
            .get("would_apply_versions")
            .and_then(Value::as_array)
            .map(std::vec::Vec::len)
            .unwrap_or_default(),
        2
    );

    let schema_after_dry_run = run_json(["--db", path_str(&db_a), "db", "schema-version"]);
    assert_eq!(as_i64(&schema_after_dry_run, "current_version"), 0);

    let migrate = run_json(["--db", path_str(&db_a), "db", "migrate"]);
    assert_eq!(as_i64(&migrate, "after_version"), 2);

    let _record = run_json([
        "--db",
        path_str(&db_a),
        "memory",
        "add",
        "constraint",
        "--actor",
        "user",
        "--action",
        "use",
        "--resource",
        "usb_drive",
        "--effect",
        "deny",
        "--writer",
        "tester",
        "--justification",
        "seed",
        "--source-uri",
        "file:///policy.md",
        "--truth-status",
        "asserted",
        "--authority",
        "authoritative",
        "--confidence",
        "0.9",
    ]);

    let package = run_json([
        "--db",
        path_str(&db_a),
        "query",
        "ask",
        "--text",
        "Am I allowed to use a USB drive?",
        "--actor",
        "user",
        "--action",
        "use",
        "--resource",
        "usb_drive",
    ]);
    let context_package_id = as_str(&package, "context_package_id").to_string();

    let integrity = run_json(["--db", path_str(&db_a), "db", "integrity-check"]);
    assert!(integrity.get("quick_check_ok").and_then(Value::as_bool).unwrap_or(false));

    let backup =
        run_json(["--db", path_str(&db_a), "db", "backup", "--out", path_str(&backup_file)]);
    assert_eq!(as_str(&backup, "status"), "ok");
    assert!(Path::new(as_str(&backup, "backup_path")).exists());

    let export =
        run_json(["--db", path_str(&db_a), "db", "export", "--out", path_str(&export_dir)]);
    let manifest = export
        .get("manifest")
        .unwrap_or_else(|| panic!("export should include manifest: {export}"));
    let files = manifest
        .get("files")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("manifest.files should be an array: {manifest}"));
    assert_eq!(files.len(), 2);
    assert!(export_dir.join("manifest.json").exists());

    let import = run_json([
        "--db",
        path_str(&db_b),
        "db",
        "import",
        "--in",
        path_str(&export_dir),
        "--allow-unsigned",
    ]);
    let summary =
        import.get("summary").unwrap_or_else(|| panic!("import should include summary: {import}"));
    assert!(summary.get("imported_records").and_then(Value::as_i64).unwrap_or(0) >= 1);
    assert!(summary.get("imported_context_packages").and_then(Value::as_i64).unwrap_or(0) >= 1);

    let shown = run_json([
        "--db",
        path_str(&db_b),
        "context",
        "show",
        "--context-package-id",
        &context_package_id,
    ]);
    assert_eq!(as_str(&shown, "context_package_id"), context_package_id);

    let restore =
        run_json(["--db", path_str(&db_b), "db", "restore", "--in", path_str(&backup_file)]);
    assert_eq!(as_i64(&restore, "current_version"), 2);

    let _ = fs::remove_dir_all(&sandbox);
}

// Test IDs: TSEC-001
#[test]
fn signed_snapshot_import_requires_and_validates_signature() {
    let sandbox = unique_temp_dir("memorykernel-cli-signed");
    let db_source = sandbox.join("source.sqlite3");
    let db_target = sandbox.join("target.sqlite3");
    let export_dir = sandbox.join("export");
    let key_path = sandbox.join("signing.key");
    fs::write(&key_path, "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff")
        .unwrap_or_else(|err| panic!("failed to write key file {}: {err}", key_path.display()));

    let _record = run_json([
        "--db",
        path_str(&db_source),
        "memory",
        "add",
        "constraint",
        "--actor",
        "user",
        "--action",
        "use",
        "--resource",
        "usb_drive",
        "--effect",
        "deny",
        "--writer",
        "tester",
        "--justification",
        "signed export fixture",
        "--source-uri",
        "file:///policy.md",
        "--truth-status",
        "asserted",
        "--authority",
        "authoritative",
        "--confidence",
        "0.9",
    ]);

    let _export = run_json([
        "--db",
        path_str(&db_source),
        "db",
        "export",
        "--out",
        path_str(&export_dir),
        "--signing-key-file",
        path_str(&key_path),
    ]);
    assert!(export_dir.join("manifest.sig").exists());

    let _import = run_json([
        "--db",
        path_str(&db_target),
        "db",
        "import",
        "--in",
        path_str(&export_dir),
        "--verify-key-file",
        path_str(&key_path),
    ]);

    let manifest_path = export_dir.join("manifest.json");
    fs::write(&manifest_path, "{\"tampered\":true}").unwrap_or_else(|err| {
        panic!("failed to tamper manifest {}: {err}", manifest_path.display())
    });
    let output = run_mk([
        "--db",
        path_str(&db_target),
        "db",
        "import",
        "--in",
        path_str(&export_dir),
        "--verify-key-file",
        path_str(&key_path),
    ]);
    assert!(!output.status.success());

    let _ = fs::remove_dir_all(&sandbox);
}

// Test IDs: TSEC-002
#[test]
fn encrypted_snapshot_round_trip_requires_explicit_decrypt_key() {
    let sandbox = unique_temp_dir("memorykernel-cli-encrypted");
    let db_source = sandbox.join("source.sqlite3");
    let db_target = sandbox.join("target.sqlite3");
    let export_dir = sandbox.join("export");
    let key_path = sandbox.join("encryption.key");
    fs::write(&key_path, "ffeeddccbbaa99887766554433221100ffeeddccbbaa99887766554433221100")
        .unwrap_or_else(|err| panic!("failed to write key file {}: {err}", key_path.display()));

    let _record = run_json([
        "--db",
        path_str(&db_source),
        "memory",
        "add",
        "decision",
        "--summary",
        "Decision: USB media use requires approval",
        "--writer",
        "tester",
        "--justification",
        "encrypted export fixture",
        "--source-uri",
        "file:///decision.md",
        "--truth-status",
        "observed",
        "--authority",
        "authoritative",
        "--confidence",
        "0.8",
    ]);

    let _export = run_json([
        "--db",
        path_str(&db_source),
        "db",
        "export",
        "--out",
        path_str(&export_dir),
        "--encrypt-key-file",
        path_str(&key_path),
    ]);
    assert!(export_dir.join("manifest.security.json").exists());

    let output_without_key = run_mk([
        "--db",
        path_str(&db_target),
        "db",
        "import",
        "--in",
        path_str(&export_dir),
        "--allow-unsigned",
    ]);
    assert!(!output_without_key.status.success());

    let _import = run_json([
        "--db",
        path_str(&db_target),
        "db",
        "import",
        "--in",
        path_str(&export_dir),
        "--allow-unsigned",
        "--decrypt-key-file",
        path_str(&key_path),
    ]);

    let _ = fs::remove_dir_all(&sandbox);
}

// Test IDs: TCLI-001, TCLI-002, TCLI-003
#[test]
fn memory_add_query_and_context_show_flow_is_consistent() {
    let sandbox = unique_temp_dir("memorykernel-cli-step8-e2e");
    let db = sandbox.join("kernel.sqlite3");

    let first = run_json([
        "--db",
        path_str(&db),
        "memory",
        "add",
        "constraint",
        "--actor",
        "user",
        "--action",
        "use",
        "--resource",
        "usb_drive",
        "--effect",
        "deny",
        "--writer",
        "tester",
        "--justification",
        "v1 policy",
        "--source-uri",
        "file:///policy.md#v1",
        "--truth-status",
        "asserted",
        "--authority",
        "authoritative",
        "--confidence",
        "0.8",
    ]);
    let memory_id = as_str(&first, "memory_id").to_string();
    let first_version_id = as_str(&first, "memory_version_id").to_string();

    let second = run_json([
        "--db",
        path_str(&db),
        "memory",
        "add",
        "constraint",
        "--memory-id",
        &memory_id,
        "--version",
        "2",
        "--actor",
        "user",
        "--action",
        "use",
        "--resource",
        "usb_drive",
        "--effect",
        "deny",
        "--writer",
        "tester",
        "--justification",
        "v2 policy",
        "--source-uri",
        "file:///policy.md#v2",
        "--truth-status",
        "asserted",
        "--authority",
        "authoritative",
        "--confidence",
        "0.95",
        "--supersedes",
        &first_version_id,
    ]);
    assert_eq!(as_str(&second, "memory_id"), memory_id);

    let package = run_json([
        "--db",
        path_str(&db),
        "query",
        "ask",
        "--text",
        "Am I allowed to use a USB drive?",
        "--actor",
        "user",
        "--action",
        "use",
        "--resource",
        "usb_drive",
    ]);

    let selected = package
        .get("selected_items")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("selected_items should be an array: {package}"));
    let excluded = package
        .get("excluded_items")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("excluded_items should be an array: {package}"));
    assert_eq!(selected.len(), 1);
    assert_eq!(excluded.len(), 1);

    let context_package_id = as_str(&package, "context_package_id").to_string();
    let shown = run_json([
        "--db",
        path_str(&db),
        "context",
        "show",
        "--context-package-id",
        &context_package_id,
    ]);
    assert_eq!(as_str(&shown, "context_package_id"), context_package_id);

    let _ = fs::remove_dir_all(&sandbox);
}

// Test IDs: TCLI-007
#[test]
fn query_recall_returns_persisted_mixed_record_context_package() {
    let sandbox = unique_temp_dir("memorykernel-cli-recall");
    let db = sandbox.join("kernel.sqlite3");

    let _decision = run_json([
        "--db",
        path_str(&db),
        "memory",
        "add",
        "decision",
        "--summary",
        "Decision: USB usage requires manager approval",
        "--writer",
        "tester",
        "--justification",
        "recall fixture decision",
        "--source-uri",
        "file:///decision.md",
        "--truth-status",
        "observed",
        "--authority",
        "authoritative",
        "--confidence",
        "0.8",
    ]);

    let _outcome = run_json([
        "--db",
        path_str(&db),
        "memory",
        "add",
        "outcome",
        "--summary",
        "Outcome: USB policy compliance improved",
        "--writer",
        "tester",
        "--justification",
        "recall fixture outcome",
        "--source-uri",
        "file:///outcome.md",
        "--truth-status",
        "observed",
        "--authority",
        "authoritative",
        "--confidence",
        "0.9",
    ]);

    let package = run_json([
        "--db",
        path_str(&db),
        "query",
        "recall",
        "--text",
        "usb policy",
        "--record-type",
        "decision",
        "--record-type",
        "outcome",
    ]);
    assert_eq!(
        package
            .get("determinism")
            .and_then(|value| value.get("ruleset_version"))
            .and_then(Value::as_str),
        Some("recall-ordering.v1")
    );
    let context_package_id = as_str(&package, "context_package_id").to_string();

    let shown = run_json([
        "--db",
        path_str(&db),
        "context",
        "show",
        "--context-package-id",
        &context_package_id,
    ]);
    assert_eq!(as_str(&shown, "context_package_id"), context_package_id);

    let _ = fs::remove_dir_all(&sandbox);
}

// Test IDs: TCLI-004
#[test]
fn memory_link_rejects_non_ulid_version_ids() {
    let sandbox = unique_temp_dir("memorykernel-cli-step8-link-validation");
    let db = sandbox.join("kernel.sqlite3");

    let output = run_mk([
        "--db",
        path_str(&db),
        "memory",
        "link",
        "--from",
        "not-a-ulid",
        "--to",
        "also-not-a-ulid",
        "--relation",
        "supersedes",
        "--writer",
        "tester",
        "--justification",
        "invalid input test",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid ULID"), "unexpected stderr: {stderr}");

    let _ = fs::remove_dir_all(&sandbox);
}

// Test IDs: TDOC-001
#[test]
fn spec_docs_restrict_ambiguous_terms_to_mkr_027_exception_line() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let spec_dir = repo_root.join("docs/spec");
    let mut violations = Vec::new();

    let entries = fs::read_dir(&spec_dir)
        .unwrap_or_else(|err| panic!("failed to read spec dir {}: {err}", spec_dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|err| panic!("failed to read dir entry: {err}"));
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let body = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read spec file {}: {err}", path.display()));
        for (line_no, line) in body.lines().enumerate() {
            let lower = line.to_ascii_lowercase();
            let has_ambiguous_term = lower.contains("usually") || lower.contains("etc.");
            let is_allowed_exception = line.contains("MKR-027");
            if has_ambiguous_term && !is_allowed_exception {
                violations.push(format!("{}:{}: {}", path.display(), line_no + 1, line.trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "found ambiguous term violations outside MKR-027:\n{}",
        violations.join("\n")
    );
}

// Test IDs: TCON-001, TCON-003
#[test]
fn cli_outputs_validate_against_versioned_schemas() {
    let sandbox = unique_temp_dir("memorykernel-contract-schemas");
    let db_path = sandbox.join("schema.sqlite3");

    let schema_version = run_json(["--db", path_str(&db_path), "db", "schema-version"]);
    validate_schema("db-schema-version.response.schema.json", &schema_version);

    let dry_run = run_json(["--db", path_str(&db_path), "db", "migrate", "--dry-run"]);
    validate_schema("db-migrate.response.schema.json", &dry_run);

    let migrate = run_json(["--db", path_str(&db_path), "db", "migrate"]);
    validate_schema("db-migrate.response.schema.json", &migrate);

    let added = run_json([
        "--db",
        path_str(&db_path),
        "memory",
        "add",
        "constraint",
        "--actor",
        "user",
        "--action",
        "use",
        "--resource",
        "usb_drive",
        "--effect",
        "deny",
        "--writer",
        "tester",
        "--justification",
        "contract schema",
        "--source-uri",
        "file:///policy.md",
        "--truth-status",
        "asserted",
        "--authority",
        "authoritative",
        "--confidence",
        "0.9",
    ]);
    validate_schema("memory-add.response.schema.json", &added);

    let asked = run_json([
        "--db",
        path_str(&db_path),
        "query",
        "ask",
        "--text",
        "Am I allowed to use a USB drive?",
        "--actor",
        "user",
        "--action",
        "use",
        "--resource",
        "usb_drive",
    ]);
    validate_schema("context-package.response.schema.json", &asked);

    let context_package_id = as_str(&asked, "context_package_id").to_string();
    let shown = run_json([
        "--db",
        path_str(&db_path),
        "context",
        "show",
        "--context-package-id",
        &context_package_id,
    ]);
    validate_schema("context-package.response.schema.json", &shown);

    let _ = fs::remove_dir_all(&sandbox);
}

// Test IDs: TCON-002
#[test]
fn key_outputs_match_golden_fixtures_after_normalization() {
    let sandbox = unique_temp_dir("memorykernel-contract-golden");
    let db_path = sandbox.join("golden.sqlite3");

    let schema_version = run_json(["--db", path_str(&db_path), "db", "schema-version"]);
    assert_golden_matches("db-schema-version.golden.json", schema_version);

    let _ = run_json(["--db", path_str(&db_path), "db", "migrate"]);
    let _ = run_json([
        "--db",
        path_str(&db_path),
        "memory",
        "add",
        "constraint",
        "--actor",
        "user",
        "--action",
        "use",
        "--resource",
        "usb_drive",
        "--effect",
        "deny",
        "--writer",
        "tester",
        "--justification",
        "golden",
        "--source-uri",
        "file:///policy.md",
        "--truth-status",
        "asserted",
        "--authority",
        "authoritative",
        "--confidence",
        "0.9",
    ]);

    let asked = run_json([
        "--db",
        path_str(&db_path),
        "query",
        "ask",
        "--text",
        "Am I allowed to use a USB drive?",
        "--actor",
        "user",
        "--action",
        "use",
        "--resource",
        "usb_drive",
    ]);
    assert_golden_matches("query-ask.golden.json", asked.clone());

    let shown = run_json([
        "--db",
        path_str(&db_path),
        "context",
        "show",
        "--context-package-id",
        as_str(&asked, "context_package_id"),
    ]);
    assert_golden_matches("context-show.golden.json", shown);

    let _ = fs::remove_dir_all(&sandbox);
}

// Test ID: TCLI-008
#[test]
fn host_cli_exposes_outcome_command_tree() {
    let sandbox = unique_temp_dir("memorykernel-cli-outcome-host");
    let db_path = sandbox.join("memory.sqlite3");

    let _ = run_json(["--db", path_str(&db_path), "db", "migrate"]);
    let decision = run_json([
        "--db",
        path_str(&db_path),
        "memory",
        "add",
        "decision",
        "--summary",
        "USB policy decision seed",
        "--writer",
        "tester",
        "--justification",
        "host outcome integration test seed",
        "--source-uri",
        "file:///decision.md",
        "--truth-status",
        "observed",
        "--authority",
        "derived",
    ]);
    let memory_id = as_str(&decision, "memory_id");

    let projector_status =
        run_json(["--db", path_str(&db_path), "outcome", "projector", "status", "--json"]);
    assert_eq!(as_str(&projector_status, "contract_version"), "projector_status.v1");

    let gate_preview = run_json([
        "--db",
        path_str(&db_path),
        "outcome",
        "gate",
        "preview",
        "--mode",
        "safe",
        "--as-of",
        "2026-02-07T12:00:00Z",
        "--candidate",
        &format!("{memory_id}:1"),
        "--json",
    ]);
    assert_eq!(as_str(&gate_preview, "contract_version"), "gate_preview.v1");
    assert_eq!(as_str(&gate_preview, "mode"), "safe");

    let _ = run_json(["--db", path_str(&db_path), "outcome", "replay"]);
    let _ = fs::remove_dir_all(&sandbox);
}

#[test]
fn integration_contract_schemas_validate_fixtures() {
    let repo = repo_root();
    let schema_dir = repo.join("contracts/integration/v1/schemas");
    let fixture_dir = repo.join("contracts/integration/v1/fixtures");

    let envelope = read_json_file(&fixture_dir.join("context-package-envelope.sample.json"));
    validate_schema_in(&schema_dir, "context-package-envelope.schema.json", &envelope);

    let trust = read_json_file(&fixture_dir.join("trust-gate-attachment.sample.json"));
    validate_schema_in(&schema_dir, "trust-gate-attachment.schema.json", &trust);

    let proposed = read_json_file(&fixture_dir.join("proposed-memory-write.sample.json"));
    validate_schema_in(&schema_dir, "proposed-memory-write.schema.json", &proposed);
}

// Test IDs: TTRI-001
#[test]
fn trilogy_contract_parity_checker_matches_sibling_repos_when_available() {
    let repo = repo_root();
    let script = repo.join("scripts/verify_contract_parity.sh");
    assert!(script.exists(), "contract parity checker script should exist: {}", script.display());

    let output = Command::new(&script)
        .arg("--canonical-root")
        .arg(&repo)
        .arg("--allow-missing")
        .output()
        .unwrap_or_else(|err| panic!("failed to execute {}: {err}", script.display()));

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "contract parity checker failed (status={}):\nstdout:\n{}\nstderr:\n{}",
            output.status, stdout, stderr
        );
    }
}

// Test IDs: TTRI-002
#[test]
fn trilogy_compatibility_artifacts_validate_against_memorykernel_consumer_requirements() {
    let repo = repo_root();
    let script = repo.join("scripts/verify_trilogy_compatibility_artifacts.sh");
    assert!(
        script.exists(),
        "trilogy compatibility artifact checker script should exist: {}",
        script.display()
    );

    let output = Command::new(&script)
        .arg("--memorykernel-root")
        .arg(&repo)
        .arg("--allow-missing")
        .output()
        .unwrap_or_else(|err| panic!("failed to execute {}: {err}", script.display()));

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "trilogy compatibility artifact checker failed (status={}):\nstdout:\n{}\nstderr:\n{}",
            output.status, stdout, stderr
        );
    }
}

// Test IDs: TDOC-002, TDOC-003, TDOC-004, TDOC-005, TDOC-007, TDOC-008
#[allow(clippy::too_many_lines)]
#[test]
fn governance_docs_exist_for_phase_checklists_and_policies() {
    let repo = repo_root();
    let required_paths = [
        "CHANGELOG.md",
        "docs/spec/versioning.md",
        "docs/implementation/adoption-gate.md",
        "docs/implementation/phase-1-contract-freeze.md",
        "docs/implementation/phase-2-integration-surface.md",
        "docs/implementation/phase-3-retrieval-expansion.md",
        "docs/implementation/phase-4-hardening.md",
        "docs/implementation/phase-5-security-and-trust.md",
        "docs/implementation/phase-6-release-and-adoption.md",
        "docs/implementation/phase-7-trilogy-convergence.md",
        "docs/implementation/phase-8-hosted-ci-convergence.md",
        "docs/implementation/phase-9-rc-orchestration.md",
        "docs/implementation/phase-10-soak-and-ops-readiness.md",
        "docs/implementation/phase-11-final-release-and-stabilization.md",
        "docs/implementation/trilogy-closeout-playbook.md",
        "docs/implementation/trilogy-closeout-report-latest.md",
        "docs/implementation/trilogy-compatibility-matrix.md",
        "docs/implementation/trilogy-release-gate.md",
        "docs/implementation/trilogy-release-report-2026-02-07.md",
        "docs/implementation/trilogy-execution-status-2026-02-07.md",
        "docs/implementation/REMAINING_ROADMAP_EXECUTION_PLAN_PRODUCER.md",
        "docs/implementation/SERVICE_V3_CUTOVER_DAY_CHECKLIST.md",
        "docs/implementation/SERVICE_V3_ROLLBACK_COMMUNICATION_PROTOCOL.md",
        ".github/workflows/release.yml",
        ".github/workflows/ci.yml",
        "docs/operations/migration-runbook.md",
        "docs/operations/recovery-runbook.md",
        "docs/implementation/pilot-acceptance.md",
        "docs/implementation/adoption-decisions.md",
        "docs/security/threat-model.md",
        "docs/security/trust-controls.md",
        "scripts/verify_contract_parity.sh",
        "scripts/verify_trilogy_compatibility_artifacts.sh",
        "scripts/verify_producer_handoff_payload.sh",
        "scripts/run_trilogy_smoke.sh",
        "scripts/run_trilogy_soak.sh",
        "scripts/run_trilogy_phase_8_11_closeout.sh",
    ];
    for path in required_paths {
        let full = repo.join(path);
        assert!(full.exists(), "required governance file is missing: {}", full.display());
        let body = fs::read_to_string(&full)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", full.display()));
        if path == "CHANGELOG.md" {
            assert!(
                body.contains("cli.v1"),
                "CHANGELOG must record active contract version, file: {}",
                full.display()
            );
        }
        if path.contains("phase-") {
            let sections = std::collections::BTreeSet::from_iter([
                "## Deliverables",
                "## Non-Goals",
                "## Rollback Criteria",
                "## Exit Checklist",
            ]);
            for section in sections {
                assert!(
                    body.contains(section),
                    "file {} missing required section {}",
                    full.display(),
                    section
                );
            }
        }
        if path == "docs/security/trust-controls.md" {
            assert!(
                body.contains("manifest.sig") && body.contains("--allow-unsigned"),
                "security controls doc must cover signature and unsigned-import policy: {}",
                full.display()
            );
        }
        if path == ".github/workflows/release.yml" {
            assert!(
                body.contains("workflow_dispatch")
                    && body.contains("Validate SemVer input")
                    && body.contains("Producer handoff payload alignment"),
                "release workflow must include semver-gated dispatch: {}",
                full.display()
            );
        }
        if path == ".github/workflows/ci.yml" {
            assert!(
                body.contains("Trilogy Monorepo Gates")
                    && body.contains("Producer handoff payload alignment"),
                "ci workflow must include producer handoff payload alignment gate: {}",
                full.display()
            );
        }
        if path == "docs/operations/migration-runbook.md" {
            assert!(
                body.contains("mk db migrate") && body.contains("mk db backup"),
                "migration runbook must include migration and backup steps: {}",
                full.display()
            );
        }
        if path == "docs/implementation/pilot-acceptance.md" {
            assert!(
                body.contains("Acceptance Criteria") && body.contains("Exit Artifacts"),
                "pilot acceptance doc missing required sections: {}",
                full.display()
            );
        }
        if path == "docs/implementation/trilogy-compatibility-matrix.md" {
            assert!(
                body.contains("Compatibility Table") && body.contains("Verification Commands"),
                "trilogy compatibility matrix missing required sections: {}",
                full.display()
            );
        }
        if path == "docs/implementation/trilogy-release-gate.md" {
            assert!(
                body.contains("Gate Commands") && body.contains("Pass Criteria"),
                "trilogy release gate missing required sections: {}",
                full.display()
            );
        }
        if path == "docs/implementation/trilogy-release-report-2026-02-07.md" {
            assert!(
                body.contains("## Commands and Results") && body.contains("## Exit Decision"),
                "trilogy release report missing required sections: {}",
                full.display()
            );
        }
        if path == "docs/implementation/trilogy-execution-status-2026-02-07.md" {
            assert!(
                body.contains("## Phase Status Summary")
                    && body.contains("## External Dependencies"),
                "trilogy execution status doc missing required sections: {}",
                full.display()
            );
        }
        if path == "docs/implementation/trilogy-closeout-playbook.md" {
            assert!(
                body.contains("## Step 1: Run Deterministic Local Closeout")
                    && body.contains("## Step 3: Capture Hosted Evidence"),
                "trilogy closeout playbook missing required sections: {}",
                full.display()
            );
        }
        if path == "docs/implementation/trilogy-closeout-report-latest.md" {
            assert!(
                body.contains("## Local Gate Results")
                    && body.contains("## Hosted Evidence Checks")
                    && body.contains("## Closeout Summary"),
                "trilogy closeout report missing required sections: {}",
                full.display()
            );
        }
        if path == "docs/implementation/REMAINING_ROADMAP_EXECUTION_PLAN_PRODUCER.md" {
            assert!(
                body.contains("## Phase 4: Rehearsal Package Hardening")
                    && body.contains(
                        "## Phase 5: Producer Cutover-Prep Controls (No Runtime Cutover)"
                    )
                    && body.contains("## Phase 6: Cutover Governance + Rollback Evidence Scaffold"),
                "remaining roadmap execution plan is missing required phase sections: {}",
                full.display()
            );
        }
        if path == "docs/implementation/SERVICE_V3_CUTOVER_DAY_CHECKLIST.md" {
            assert!(
                body.contains("## Producer Cutover Checklist")
                    && body.contains("## Rollback Triggers (immediate)")
                    && body.contains("## Rollback Evidence Requirements"),
                "service v3 cutover-day checklist missing required sections: {}",
                full.display()
            );
        }
        if path == "docs/implementation/SERVICE_V3_ROLLBACK_COMMUNICATION_PROTOCOL.md" {
            assert!(
                body.contains("## Trigger-to-Action Matrix")
                    && body.contains("## Required Reversal Evidence Bundle")
                    && body.contains("## Go/No-Go Communication Protocol"),
                "service v3 rollback communication protocol missing required sections: {}",
                full.display()
            );
        }
        if path == "scripts/verify_producer_handoff_payload.sh" {
            assert!(
                body.contains("service-v3-candidate")
                    && body.contains("Producer handoff payload checks passed."),
                "producer handoff payload checker script missing required candidate-mode checks: {}",
                full.display()
            );
        }
        if path == "scripts/run_trilogy_phase_8_11_closeout.sh" {
            assert!(
                body.contains("--require-hosted") && body.contains("Hosted Evidence Checks"),
                "phase 8-11 closeout script missing hosted evidence controls: {}",
                full.display()
            );
        }
    }
}
