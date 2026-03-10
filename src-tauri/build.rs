use std::collections::BTreeSet;
use std::fs;

fn parse_app_commands() -> Result<Vec<String>, String> {
    let lib_rs = fs::read_to_string("src/lib.rs").map_err(|e| e.to_string())?;
    let mut commands = BTreeSet::new();
    let mut in_handler = false;

    for line in lib_rs.lines() {
        if line.contains("invoke_handler(tauri::generate_handler![") {
            in_handler = true;
            continue;
        }

        if in_handler && line.contains("])") {
            break;
        }

        if !in_handler {
            continue;
        }

        if let Some(index) = line.find("commands::") {
            let remainder = &line[index + "commands::".len()..];
            let entry = remainder
                .split(|ch: char| ch == ',' || ch.is_whitespace() || ch == ']')
                .next()
                .unwrap_or_default()
                .trim();
            let command = entry.rsplit("::").next().unwrap_or_default().trim();
            if !command.is_empty() {
                commands.insert(command.to_string());
            }
        }
    }

    if commands.is_empty() {
        return Err("no app commands found in src/lib.rs".to_string());
    }

    Ok(commands.into_iter().collect())
}

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");

    let commands = parse_app_commands().unwrap_or_else(|error| {
        eprintln!("Failed to parse Tauri command list: {}", error);
        std::process::exit(1);
    });

    let leaked_commands: Vec<&'static str> = commands
        .into_iter()
        .map(|command| Box::leak(command.into_boxed_str()) as &'static str)
        .collect();
    let leaked_commands: &'static [&'static str] = Box::leak(leaked_commands.into_boxed_slice());

    let attributes = tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(leaked_commands),
    );

    if let Err(error) = tauri_build::try_build(attributes) {
        let error = format!("{error:#}");
        println!("{error}");
        if error.starts_with("unknown field") {
            println!(
                "found an unknown configuration field. This usually means the CLI and tauri-build crates are out of sync."
            );
        }
        std::process::exit(1);
    }
}
