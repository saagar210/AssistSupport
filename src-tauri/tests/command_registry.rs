use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn extract_generate_handler_body(lib_src: &str) -> String {
    let marker = "tauri::generate_handler![";
    let start = lib_src
        .find(marker)
        .unwrap_or_else(|| panic!("missing `{marker}` in src/lib.rs"));
    let mut idx = start + marker.len();
    let bytes = lib_src.as_bytes();
    let mut depth = 1usize;

    while idx < bytes.len() {
        match bytes[idx] as char {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return lib_src[start + marker.len()..idx].to_string();
                }
            }
            _ => {}
        }
        idx += 1;
    }

    panic!("unterminated generate_handler! list in src/lib.rs");
}

fn extract_registered_commands(handler_body: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut pos = 0usize;

    while let Some(found) = handler_body[pos..].find("commands::") {
        let start = pos + found;
        let mut end = start;
        for ch in handler_body[start..].chars() {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == ':' {
                end += ch.len_utf8();
            } else {
                break;
            }
        }

        let candidate = &handler_body[start..end];
        if candidate.matches("::").count() >= 2 {
            out.insert(candidate.to_string());
        }
        pos = end;
    }

    out
}

fn extract_module_commands(module_src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lines: Vec<&str> = module_src.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with("#[tauri::command") {
            continue;
        }

        let mut found = None;
        for next in lines.iter().skip(i + 1).take(12) {
            if let Some(fn_pos) = next.find("fn ") {
                let tail = &next[fn_pos + 3..];
                let name: String = tail
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() {
                    found = Some(name);
                    break;
                }
            }
        }

        let name = found.unwrap_or_else(|| {
            panic!(
                "found #[tauri::command] but no function signature nearby: `{}`",
                trimmed
            )
        });
        out.push(name);
    }

    out
}

#[test]
fn tauri_command_registry_matches_generate_handler() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let root = Path::new(&manifest_dir);
    let lib_rs_path = root.join("src/lib.rs");
    let commands_dir = root.join("src/commands");

    let lib_src = fs::read_to_string(&lib_rs_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", lib_rs_path.display()));
    let handler_body = extract_generate_handler_body(&lib_src);
    let registered = extract_registered_commands(&handler_body);

    let mut discovered = BTreeSet::new();
    for entry in fs::read_dir(&commands_dir)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", commands_dir.display()))
    {
        let entry = entry.expect("failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }

        let stem = match path.file_stem().and_then(|stem| stem.to_str()) {
            Some("mod") | None => continue,
            Some(stem) => stem,
        };

        let module_src = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        for fn_name in extract_module_commands(&module_src) {
            discovered.insert(format!("commands::{stem}::{fn_name}"));
        }
    }

    let missing: Vec<_> = discovered.difference(&registered).cloned().collect();
    let extra: Vec<_> = registered.difference(&discovered).cloned().collect();

    assert!(
        missing.is_empty() && extra.is_empty(),
        "Tauri command registration drift detected.\\nMissing from generate_handler!: {:?}\\nRegistered but not discovered as #[tauri::command]: {:?}",
        missing,
        extra
    );
}
