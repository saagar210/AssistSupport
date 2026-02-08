use std::fs;
use std::path::{Path, PathBuf};

use jsonschema::JSONSchema;
use serde_json::Value;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|err| panic!("failed to canonicalize repo root: {err}"))
}

fn read_json(path: &Path) -> Value {
    let body = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    serde_json::from_str(&body)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()))
}

fn assert_schema(schema_path: &Path, value: &Value) {
    let schema = read_json(schema_path);
    let compiled = JSONSchema::compile(&schema)
        .unwrap_or_else(|err| panic!("failed to compile {}: {err}", schema_path.display()));
    if let Some(errors) = compiled
        .validate(value)
        .err()
        .map(|iter| iter.map(|err| err.to_string()).collect::<Vec<_>>())
    {
        panic!(
            "schema validation failed for {}:\n{}",
            schema_path.display(),
            errors.join("\n")
        );
    }
}

#[test]
fn integration_contract_pack_validates_fixtures() {
    let repo = repo_root();
    let schema_dir = repo.join("contracts/integration/v1/schemas");
    let fixture_dir = repo.join("contracts/integration/v1/fixtures");

    let trust_attachment = read_json(&fixture_dir.join("trust-gate-attachment.sample.json"));
    assert_schema(
        &schema_dir.join("trust-gate-attachment.schema.json"),
        &trust_attachment,
    );

    let error_envelope = serde_json::json!({
        "code": "outcome.not_found",
        "message": "trust snapshot not found"
    });
    assert_schema(
        &schema_dir.join("error-envelope.schema.json"),
        &error_envelope,
    );

    let compatibility = read_json(&repo.join("trilogy-compatibility.v1.json"));
    assert_eq!(
        compatibility["artifact_version"],
        serde_json::json!("trilogy_compatibility.v1")
    );
    assert_eq!(
        compatibility["supported_memorykernel_contract_baseline"],
        serde_json::json!("integration/v1")
    );
    assert_eq!(
        compatibility["required_stable_embed_api"],
        serde_json::json!([
            "run_cli",
            "run_outcome_with_db",
            "run_outcome",
            "run_benchmark"
        ])
    );
    assert_eq!(
        compatibility["benchmark_threshold_semantics"]["threshold_triplet_required"],
        serde_json::json!(true)
    );
    assert_eq!(
        compatibility["benchmark_threshold_semantics"]["non_zero_exit_on_any_violation"],
        serde_json::json!(true)
    );
}
