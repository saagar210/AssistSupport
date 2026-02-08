use std::collections::BTreeSet;
use std::env;
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

fn collect_relative_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(current) = stack.pop() {
        let entries = fs::read_dir(&current)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", current.display()));
        for entry in entries {
            let entry = entry.unwrap_or_else(|err| panic!("failed to read directory entry: {err}"));
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or_else(|err| {
                        panic!(
                            "failed to strip prefix {} from {}: {err}",
                            root.display(),
                            path.display()
                        )
                    })
                    .to_path_buf();
                out.push(rel);
            }
        }
    }
    out.sort();
    out
}

fn assert_dir_parity(local_root: &Path, canonical_root: &Path) {
    let local_files = collect_relative_files(local_root);
    let canonical_files = collect_relative_files(canonical_root);

    let local_set: BTreeSet<PathBuf> = local_files.iter().cloned().collect();
    let canonical_set: BTreeSet<PathBuf> = canonical_files.iter().cloned().collect();
    assert_eq!(
        local_set,
        canonical_set,
        "contract file-set drift detected:\nlocal={} canonical={}\n\
         changed files require explicit version bump (e.g. v2) rather than mutating v1",
        local_root.display(),
        canonical_root.display()
    );

    for rel in local_files {
        let local = local_root.join(&rel);
        let canonical = canonical_root.join(&rel);
        let local_body = fs::read_to_string(&local)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", local.display()));
        let canonical_body = fs::read_to_string(&canonical)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", canonical.display()));
        assert_eq!(
            local_body,
            canonical_body,
            "contract drift detected for {}.\n\
             v1 contracts must remain byte-identical to MemoryKernel canonical pack.\n\
             use explicit version bump for changes.",
            rel.display()
        );
    }
}

fn canonical_contract_root(repo: &Path) -> PathBuf {
    if let Ok(raw) = env::var("MEMORYKERNEL_CANONICAL_CONTRACTS") {
        return PathBuf::from(raw);
    }
    repo.join("../MemoryKernel/contracts/integration")
}

fn read_required_str<'a>(value: &'a Value, path: &str) -> &'a str {
    value
        .as_str()
        .unwrap_or_else(|| panic!("expected string at {path}"))
}

fn read_required_string_array(value: &Value, path: &str) -> Vec<String> {
    let items = value
        .as_array()
        .unwrap_or_else(|| panic!("expected array at {path}"));
    items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            item.as_str()
                .unwrap_or_else(|| panic!("expected string at {path}[{idx}]"))
                .to_string()
        })
        .collect()
}

#[test]
fn integration_contract_pack_validates_fixtures() {
    let repo = repo_root();
    let schema_dir = repo.join("contracts/integration/v1/schemas");
    let fixture_dir = repo.join("contracts/integration/v1/fixtures");

    let context_envelope = read_json(&fixture_dir.join("context-package-envelope.sample.json"));
    assert_schema(
        &schema_dir.join("context-package-envelope.schema.json"),
        &context_envelope,
    );

    let trust_attachment = read_json(&fixture_dir.join("trust-gate-attachment.sample.json"));
    assert_schema(
        &schema_dir.join("trust-gate-attachment.schema.json"),
        &trust_attachment,
    );

    let proposed_write = read_json(&fixture_dir.join("proposed-memory-write.sample.json"));
    assert_schema(
        &schema_dir.join("proposed-memory-write.schema.json"),
        &proposed_write,
    );
}

#[test]
fn integration_contract_v1_parity_matches_memorykernel_canonical_pack() {
    let repo = repo_root();
    let local_v1 = repo.join("contracts/integration/v1");
    let canonical_root = canonical_contract_root(&repo);
    let canonical_v1 = canonical_root.join("v1");

    assert!(
        canonical_v1.exists(),
        "missing canonical pack at {}. set MEMORYKERNEL_CANONICAL_CONTRACTS or provide sibling MemoryKernel checkout",
        canonical_v1.display()
    );

    assert_dir_parity(&local_v1.join("schemas"), &canonical_v1.join("schemas"));
    assert_dir_parity(&local_v1.join("fixtures"), &canonical_v1.join("fixtures"));
}

#[test]
fn trilogy_compatibility_artifact_v1_shape_and_content_is_valid() {
    let repo = repo_root();
    let artifact = read_json(&repo.join("trilogy-compatibility.v1.json"));

    assert_eq!(
        read_required_str(&artifact["artifact_version"], "artifact_version"),
        "v1"
    );
    assert_eq!(
        read_required_str(&artifact["project"]["name"], "project.name"),
        "MultiAgentCenter"
    );
    assert_eq!(
        read_required_str(&artifact["project"]["version"], "project.version"),
        "0.1.0"
    );
    assert_eq!(
        read_required_str(
            &artifact["memory_kernel"]["contract_baseline"],
            "memory_kernel.contract_baseline"
        ),
        "integration/v1"
    );
    assert_eq!(
        read_required_string_array(
            &artifact["memory_kernel"]["api_retrieval"]["context_queries_mode"]["supported"],
            "memory_kernel.api_retrieval.context_queries_mode.supported"
        ),
        vec!["policy".to_string(), "recall".to_string()]
    );
    assert_eq!(
        read_required_str(
            &artifact["memory_kernel"]["api_retrieval"]["context_queries_mode"]["default"],
            "memory_kernel.api_retrieval.context_queries_mode.default"
        ),
        "policy"
    );
    assert_eq!(
        read_required_string_array(
            &artifact["memory_kernel"]["api_retrieval"]["recall"]["default_record_types"],
            "memory_kernel.api_retrieval.recall.default_record_types"
        ),
        vec![
            "decision".to_string(),
            "preference".to_string(),
            "event".to_string(),
            "outcome".to_string()
        ]
    );
    assert_eq!(
        read_required_str(
            &artifact["memory_kernel"]["api_retrieval"]["recall"]["missing_record_types_behavior"],
            "memory_kernel.api_retrieval.recall.missing_record_types_behavior"
        ),
        "use_default_scope"
    );
    assert_eq!(
        read_required_str(
            &artifact["memory_kernel"]["api_retrieval"]["recall"]["empty_record_types_behavior"],
            "memory_kernel.api_retrieval.recall.empty_record_types_behavior"
        ),
        "use_default_scope"
    );
    assert_eq!(
        read_required_str(
            &artifact["memory_kernel"]["api_retrieval"]["recall"]["invalid_record_types_behavior"],
            "memory_kernel.api_retrieval.recall.invalid_record_types_behavior"
        ),
        "validation_error"
    );
    assert_eq!(
        read_required_string_array(
            &artifact["trust"]["memory_ref_identity_required_fields"],
            "trust.memory_ref_identity_required_fields"
        ),
        vec![
            "memory_id".to_string(),
            "version".to_string(),
            "memory_version_id".to_string()
        ]
    );
}
