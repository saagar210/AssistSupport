use std::collections::BTreeSet;
use std::fs;

fn tauri_commands() -> BTreeSet<String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let lib_rs = fs::read_to_string(format!("{manifest_dir}/src/lib.rs")).expect("read src/lib.rs");
    let body = lib_rs
        .split("invoke_handler(tauri::generate_handler![")
        .nth(1)
        .and_then(|rest| rest.split("])").next())
        .expect("invoke handler block");

    body.lines()
        .filter_map(|line| {
            line.find("commands::")
                .map(|index| &line[index + "commands::".len()..])
        })
        .filter_map(|line| {
            line.split(|ch: char| ch == ',' || ch.is_whitespace() || ch == ']')
                .next()
        })
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            entry
                .rsplit("::")
                .next()
                .unwrap_or(entry)
                .trim()
                .to_string()
        })
        .collect()
}

fn permission_file_commands() -> BTreeSet<String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let permissions = fs::read_to_string(format!("{manifest_dir}/permissions/default.toml"))
        .expect("read permissions/default.toml");
    let mut commands = BTreeSet::new();
    let mut in_allow_list = false;

    for raw_line in permissions.lines() {
        let line = raw_line.trim();
        if line.starts_with("commands.allow = [") {
            in_allow_list = true;
            continue;
        }

        if in_allow_list && line == "]" {
            in_allow_list = false;
            continue;
        }

        if !in_allow_list || !line.starts_with('"') {
            continue;
        }

        if let Some(command) = line
            .trim_end_matches(',')
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
        {
            commands.insert(command.to_string());
        }
    }

    commands
}

#[test]
fn permission_manifest_covers_all_tauri_commands() {
    let expected = tauri_commands();
    let actual = permission_file_commands();

    let missing: Vec<_> = expected.difference(&actual).cloned().collect();
    let unexpected: Vec<_> = actual.difference(&expected).cloned().collect();

    assert!(
        missing.is_empty(),
        "permission manifest is missing commands: {}",
        missing.join(", ")
    );
    assert!(
        unexpected.is_empty(),
        "permission manifest references unknown commands: {}",
        unexpected.join(", ")
    );
}

#[test]
fn default_capability_references_expected_permission_groups() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let capability = fs::read_to_string(format!("{manifest_dir}/capabilities/default.json"))
        .expect("read capability");

    for identifier in [
        "startup-core",
        "diagnostics-and-recovery",
        "vector-and-model-runtime",
        "knowledge-base",
        "drafts-and-templates",
        "customization-and-workspace",
        "integrations-and-secrets",
        "search-sidecar",
        "jobs-and-evals",
        "operations-and-analytics",
    ] {
        assert!(
            capability.contains(&format!("\"{identifier}\"")),
            "default capability must reference permission group {identifier}"
        );
    }
}
