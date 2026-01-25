//! Backup and restore functionality for AssistSupport
//!
//! Exports app data (drafts, templates, variables, custom trees, settings, KB config)
//! as a ZIP file. Imports restore data from a ZIP file.

use crate::db::{CustomVariable, Database, DecisionTree, ResponseTemplate, SavedDraft};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

/// Current backup format version
const BACKUP_VERSION: &str = "1";

/// Backup version metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupVersion {
    pub version: String,
    pub created_at: String,
    pub app_version: String,
}

/// KB configuration (folder path only, not files)
#[derive(Debug, Serialize, Deserialize)]
pub struct KbConfig {
    pub folder_path: Option<String>,
}

/// Settings key-value pairs
#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsExport {
    pub entries: Vec<SettingEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingEntry {
    pub key: String,
    pub value: String,
}

/// Summary of export operation
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportSummary {
    pub drafts_count: usize,
    pub templates_count: usize,
    pub variables_count: usize,
    pub trees_count: usize,
    pub path: String,
}

/// Summary of import operation
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportSummary {
    pub drafts_imported: usize,
    pub templates_imported: usize,
    pub variables_imported: usize,
    pub trees_imported: usize,
}

/// Preview of what will be imported
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportPreview {
    pub version: String,
    pub drafts_count: usize,
    pub templates_count: usize,
    pub variables_count: usize,
    pub trees_count: usize,
}

/// Backup error type
#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Invalid backup: {0}")]
    InvalidBackup(String),
}

/// Export all app data to a ZIP file
pub fn export_backup(db: &Database, output_path: &Path) -> Result<ExportSummary, BackupError> {
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Export version info
    let version = BackupVersion {
        version: BACKUP_VERSION.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    zip.start_file("version.json", options)?;
    zip.write_all(serde_json::to_string_pretty(&version)?.as_bytes())?;

    // Export drafts (excluding autosaves)
    let drafts = db
        .list_drafts(10000)
        .map_err(|e| BackupError::Database(e.to_string()))?;
    zip.start_file("drafts.json", options)?;
    zip.write_all(serde_json::to_string_pretty(&drafts)?.as_bytes())?;

    // Export templates
    let templates = db
        .list_templates()
        .map_err(|e| BackupError::Database(e.to_string()))?;
    zip.start_file("templates.json", options)?;
    zip.write_all(serde_json::to_string_pretty(&templates)?.as_bytes())?;

    // Export custom variables
    let variables = db
        .list_custom_variables()
        .map_err(|e| BackupError::Database(e.to_string()))?;
    zip.start_file("variables.json", options)?;
    zip.write_all(serde_json::to_string_pretty(&variables)?.as_bytes())?;

    // Export custom decision trees only (source='custom')
    let all_trees = db
        .list_decision_trees()
        .map_err(|e| BackupError::Database(e.to_string()))?;
    let custom_trees: Vec<_> = all_trees
        .into_iter()
        .filter(|t| t.source == "custom")
        .collect();
    zip.start_file("trees.json", options)?;
    zip.write_all(serde_json::to_string_pretty(&custom_trees)?.as_bytes())?;

    // Export settings
    let settings = export_settings(db)?;
    zip.start_file("settings.json", options)?;
    zip.write_all(serde_json::to_string_pretty(&settings)?.as_bytes())?;

    // Export KB config (folder path only)
    let kb_config = export_kb_config(db)?;
    zip.start_file("kb_config.json", options)?;
    zip.write_all(serde_json::to_string_pretty(&kb_config)?.as_bytes())?;

    zip.finish()?;

    Ok(ExportSummary {
        drafts_count: drafts.len(),
        templates_count: templates.len(),
        variables_count: variables.len(),
        trees_count: custom_trees.len(),
        path: output_path.display().to_string(),
    })
}

/// Preview what will be imported from a ZIP file
pub fn preview_import(zip_path: &Path) -> Result<ImportPreview, BackupError> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Read version
    let version = read_json_from_zip::<BackupVersion>(&mut archive, "version.json")?;
    if version.version != BACKUP_VERSION {
        return Err(BackupError::InvalidBackup(format!(
            "Unsupported backup version: {}",
            version.version
        )));
    }

    // Count entries
    let drafts: Vec<SavedDraft> = read_json_from_zip(&mut archive, "drafts.json")?;
    let templates: Vec<ResponseTemplate> = read_json_from_zip(&mut archive, "templates.json")?;
    let variables: Vec<CustomVariable> = read_json_from_zip(&mut archive, "variables.json")?;
    let trees: Vec<DecisionTree> = read_json_from_zip(&mut archive, "trees.json")?;

    Ok(ImportPreview {
        version: version.version,
        drafts_count: drafts.len(),
        templates_count: templates.len(),
        variables_count: variables.len(),
        trees_count: trees.len(),
    })
}

/// Import data from a ZIP file
/// Merge strategy: insert new, skip existing (by ID)
pub fn import_backup(db: &Database, zip_path: &Path) -> Result<ImportSummary, BackupError> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Verify version
    let version = read_json_from_zip::<BackupVersion>(&mut archive, "version.json")?;
    if version.version != BACKUP_VERSION {
        return Err(BackupError::InvalidBackup(format!(
            "Unsupported backup version: {}",
            version.version
        )));
    }

    let mut drafts_imported = 0;
    let mut templates_imported = 0;
    let mut variables_imported = 0;
    let mut trees_imported = 0;

    // Import drafts (skip existing by ID)
    let drafts: Vec<SavedDraft> = read_json_from_zip(&mut archive, "drafts.json")?;
    for draft in drafts {
        if db.get_draft(&draft.id).is_err() {
            db.save_draft(&draft)
                .map_err(|e| BackupError::Database(e.to_string()))?;
            drafts_imported += 1;
        }
    }

    // Import templates (skip existing by ID)
    let templates: Vec<ResponseTemplate> = read_json_from_zip(&mut archive, "templates.json")?;
    for template in templates {
        if db.get_template(&template.id).is_err() {
            db.save_template(&template)
                .map_err(|e| BackupError::Database(e.to_string()))?;
            templates_imported += 1;
        }
    }

    // Import variables (skip existing by name)
    let variables: Vec<CustomVariable> = read_json_from_zip(&mut archive, "variables.json")?;
    let existing_vars = db
        .list_custom_variables()
        .map_err(|e| BackupError::Database(e.to_string()))?;
    let existing_names: std::collections::HashSet<_> =
        existing_vars.iter().map(|v| v.name.clone()).collect();
    for var in variables {
        if !existing_names.contains(&var.name) {
            db.save_custom_variable(&var)
                .map_err(|e| BackupError::Database(e.to_string()))?;
            variables_imported += 1;
        }
    }

    // Import custom trees (skip existing by ID)
    let trees: Vec<DecisionTree> = read_json_from_zip(&mut archive, "trees.json")?;
    for tree in trees {
        if db.get_decision_tree(&tree.id).is_err() {
            db.save_decision_tree(&tree)
                .map_err(|e| BackupError::Database(e.to_string()))?;
            trees_imported += 1;
        }
    }

    // Import settings (merge, skip schema_version)
    if let Ok(settings) = read_json_from_zip::<SettingsExport>(&mut archive, "settings.json") {
        for entry in settings.entries {
            if entry.key != "schema_version" {
                import_setting(db, &entry.key, &entry.value)?;
            }
        }
    }

    // Import KB config
    if let Ok(kb_config) = read_json_from_zip::<KbConfig>(&mut archive, "kb_config.json") {
        if let Some(folder_path) = kb_config.folder_path {
            import_setting(db, "kb_folder", &folder_path)?;
        }
    }

    Ok(ImportSummary {
        drafts_imported,
        templates_imported,
        variables_imported,
        trees_imported,
    })
}

/// Helper: Read JSON from a ZIP file
fn read_json_from_zip<T: serde::de::DeserializeOwned>(
    archive: &mut ZipArchive<File>,
    filename: &str,
) -> Result<T, BackupError> {
    let mut file = archive.by_name(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(serde_json::from_str(&contents)?)
}

/// Export settings from database
fn export_settings(db: &Database) -> Result<SettingsExport, BackupError> {
    let conn = db.conn();
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| BackupError::Database(e.to_string()))?;

    let entries = stmt
        .query_map([], |row| {
            Ok(SettingEntry {
                key: row.get(0)?,
                value: row.get(1)?,
            })
        })
        .map_err(|e| BackupError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(SettingsExport { entries })
}

/// Export KB config from settings
fn export_kb_config(db: &Database) -> Result<KbConfig, BackupError> {
    let conn = db.conn();
    let folder_path: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'kb_folder'",
            [],
            |row| row.get(0),
        )
        .ok();

    Ok(KbConfig { folder_path })
}

/// Import a setting into the database
fn import_setting(db: &Database, key: &str, value: &str) -> Result<(), BackupError> {
    let conn = db.conn();
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
        rusqlite::params![key, value],
    )
    .map_err(|e| BackupError::Database(e.to_string()))?;
    Ok(())
}
