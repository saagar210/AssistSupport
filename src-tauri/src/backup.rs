//! Backup and restore functionality for AssistSupport
//!
//! Exports app data (drafts, templates, variables, custom trees, settings, KB config)
//! as a ZIP file. Imports restore data from a ZIP file.
//!
//! Supports optional password-based encryption using Argon2id + AES-256-GCM.

use crate::db::{CustomVariable, Database, DecisionTree, ResponseTemplate, SavedDraft};
use crate::security::ExportCrypto;
use crate::validation::validate_within_home;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

// Backup imports are an untrusted input surface. These limits prevent
// decompression bombs and absurdly large JSON payloads from exhausting memory.
const MAX_ZIP_ENTRIES: usize = 10_000;
const MAX_ZIP_TOTAL_UNCOMPRESSED_BYTES: u64 = 500 * 1024 * 1024; // 500MB
const MAX_ZIP_ENTRY_UNCOMPRESSED_BYTES: u64 = 100 * 1024 * 1024; // 100MB
const MAX_ZIP_JSON_ENTRY_BYTES: u64 = 20 * 1024 * 1024; // 20MB per JSON file
const MAX_BACKUP_FILE_BYTES: u64 = 512 * 1024 * 1024; // 512MB on-disk input cap

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
    pub encrypted: bool,
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
    pub encrypted: bool,
    /// Path to the backup file (for subsequent import with password)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Encrypted backup metadata (stored as first bytes of encrypted file)
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedBackupHeader {
    pub magic: String, // "ASSISTSUPPORT_ENCRYPTED_BACKUP"
    pub version: String,
    pub salt: [u8; 32],
    pub nonce: [u8; 12],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plaintext_size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ciphertext_size: Option<u64>,
}

const ENCRYPTED_MAGIC: &str = "ASSISTSUPPORT_ENCRYPTED_BACKUP";

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
    #[error("Backup is encrypted - password required")]
    EncryptionRequired,
    #[error("Decryption failed - incorrect password or corrupted backup")]
    DecryptionFailed,
    #[error("Encryption error: {0}")]
    Encryption(String),
}

/// Export all app data to a ZIP file (optionally encrypted with password)
pub fn export_backup(
    db: &Database,
    output_path: &Path,
    password: Option<&str>,
) -> Result<ExportSummary, BackupError> {
    // Create ZIP in memory first
    let mut zip_buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut zip_buffer);
        let mut zip = ZipWriter::new(cursor);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

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
    }

    // Get counts for summary (re-query is simpler than tracking during export)
    let drafts_count = db.list_drafts(10000).map(|d| d.len()).unwrap_or(0);
    let templates_count = db.list_templates().map(|t| t.len()).unwrap_or(0);
    let variables_count = db.list_custom_variables().map(|v| v.len()).unwrap_or(0);
    let trees_count = db
        .list_decision_trees()
        .map(|t| t.into_iter().filter(|tree| tree.source == "custom").count())
        .unwrap_or(0);

    // If password provided, encrypt the ZIP data
    let encrypted = password.is_some();
    if let Some(pwd) = password {
        let (ciphertext, salt, nonce) = ExportCrypto::encrypt_for_export(&zip_buffer, pwd)
            .map_err(|e| BackupError::Encryption(e.to_string()))?;

        // Write encrypted backup file
        let header = EncryptedBackupHeader {
            magic: ENCRYPTED_MAGIC.to_string(),
            version: BACKUP_VERSION.to_string(),
            salt,
            nonce,
            plaintext_size: Some(zip_buffer.len() as u64),
            ciphertext_size: Some(ciphertext.len() as u64),
        };
        let header_json = serde_json::to_vec(&header)?;
        let header_len = (header_json.len() as u32).to_le_bytes();

        let mut file = File::create(output_path)?;
        file.write_all(&header_len)?;
        file.write_all(&header_json)?;
        file.write_all(&ciphertext)?;
    } else {
        // Write unencrypted ZIP
        let mut file = File::create(output_path)?;
        file.write_all(&zip_buffer)?;
    }

    Ok(ExportSummary {
        drafts_count,
        templates_count,
        variables_count,
        trees_count,
        path: output_path.display().to_string(),
        encrypted,
    })
}

/// Check if a file is an encrypted backup
fn is_encrypted_backup(path: &Path) -> Result<Option<EncryptedBackupHeader>, BackupError> {
    let mut file = File::open(path)?;
    let mut header_len_bytes = [0u8; 4];

    if file.read_exact(&mut header_len_bytes).is_err() {
        return Ok(None);
    }

    let header_len = u32::from_le_bytes(header_len_bytes) as usize;

    // Sanity check: header shouldn't be huge
    if header_len > 1024 {
        return Ok(None);
    }

    let mut header_json = vec![0u8; header_len];
    if file.read_exact(&mut header_json).is_err() {
        return Ok(None);
    }

    match serde_json::from_slice::<EncryptedBackupHeader>(&header_json) {
        Ok(header) if header.magic == ENCRYPTED_MAGIC => Ok(Some(header)),
        _ => Ok(None),
    }
}

/// Decrypt an encrypted backup file and return the ZIP data
fn decrypt_backup(path: &Path, password: &str) -> Result<Vec<u8>, BackupError> {
    validate_backup_file_size(path)?;
    let mut file = File::open(path)?;

    // Read header length
    let mut header_len_bytes = [0u8; 4];
    file.read_exact(&mut header_len_bytes)?;
    let header_len = u32::from_le_bytes(header_len_bytes) as usize;

    // Read header
    let mut header_json = vec![0u8; header_len];
    file.read_exact(&mut header_json)?;
    let header: EncryptedBackupHeader = serde_json::from_slice(&header_json)?;

    if header.magic != ENCRYPTED_MAGIC {
        return Err(BackupError::InvalidBackup("Not an encrypted backup".into()));
    }

    if let Some(plaintext_size) = header.plaintext_size {
        if plaintext_size > MAX_ZIP_TOTAL_UNCOMPRESSED_BYTES {
            return Err(BackupError::InvalidBackup(format!(
                "Encrypted backup payload too large: {} bytes (max {} bytes)",
                plaintext_size, MAX_ZIP_TOTAL_UNCOMPRESSED_BYTES
            )));
        }
    }

    // Read ciphertext
    let ciphertext_len = file.metadata()?.len().saturating_sub(4 + header_len as u64);
    if ciphertext_len > MAX_BACKUP_FILE_BYTES {
        return Err(BackupError::InvalidBackup(format!(
            "Encrypted backup file too large: {} bytes (max {} bytes)",
            ciphertext_len, MAX_BACKUP_FILE_BYTES
        )));
    }
    if let Some(expected_ciphertext_size) = header.ciphertext_size {
        if expected_ciphertext_size > MAX_BACKUP_FILE_BYTES {
            return Err(BackupError::InvalidBackup(format!(
                "Encrypted backup ciphertext too large: {} bytes (max {} bytes)",
                expected_ciphertext_size, MAX_BACKUP_FILE_BYTES
            )));
        }
        if expected_ciphertext_size != ciphertext_len {
            return Err(BackupError::InvalidBackup(format!(
                "Encrypted backup size mismatch: header={} bytes, file={} bytes",
                expected_ciphertext_size, ciphertext_len
            )));
        }
    }

    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)?;

    // Decrypt
    let zip_data = ExportCrypto::decrypt_export(&ciphertext, &header.salt, &header.nonce, password)
        .map_err(|_| BackupError::DecryptionFailed)?;

    if zip_data.len() as u64 > MAX_ZIP_TOTAL_UNCOMPRESSED_BYTES {
        return Err(BackupError::InvalidBackup(format!(
            "Decrypted backup payload too large: {} bytes (max {} bytes)",
            zip_data.len(),
            MAX_ZIP_TOTAL_UNCOMPRESSED_BYTES
        )));
    }

    Ok(zip_data)
}

fn validate_backup_file_size(path: &Path) -> Result<(), BackupError> {
    let file_size = fs::metadata(path)?.len();
    if file_size > MAX_BACKUP_FILE_BYTES {
        return Err(BackupError::InvalidBackup(format!(
            "Backup file too large: {} bytes (max {} bytes)",
            file_size, MAX_BACKUP_FILE_BYTES
        )));
    }
    Ok(())
}

fn preview_import_archive<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    encrypted: bool,
    path_str: String,
) -> Result<ImportPreview, BackupError> {
    validate_zip_archive_limits(archive)?;

    let version = read_json_from_zip::<BackupVersion, _>(archive, "version.json")?;
    if version.version != BACKUP_VERSION {
        return Err(BackupError::InvalidBackup(format!(
            "Unsupported backup version: {}",
            version.version
        )));
    }

    let drafts: Vec<SavedDraft> = read_json_from_zip(archive, "drafts.json")?;
    let templates: Vec<ResponseTemplate> = read_json_from_zip(archive, "templates.json")?;
    let variables: Vec<CustomVariable> = read_json_from_zip(archive, "variables.json")?;
    let trees: Vec<DecisionTree> = read_json_from_zip(archive, "trees.json")?;

    Ok(ImportPreview {
        version: version.version,
        drafts_count: drafts.len(),
        templates_count: templates.len(),
        variables_count: variables.len(),
        trees_count: trees.len(),
        encrypted,
        path: Some(path_str),
    })
}

/// Preview what will be imported from a backup file (with optional password for encrypted backups)
pub fn preview_import(
    backup_path: &Path,
    password: Option<&str>,
) -> Result<ImportPreview, BackupError> {
    validate_backup_file_size(backup_path)?;
    let path_str = backup_path.display().to_string();

    if is_encrypted_backup(backup_path)?.is_some() {
        let pwd = password.ok_or(BackupError::EncryptionRequired)?;
        let zip_data = decrypt_backup(backup_path, pwd)?;
        let cursor = Cursor::new(zip_data);
        let mut archive = ZipArchive::new(cursor)?;
        return preview_import_archive(&mut archive, true, path_str);
    }

    let file = File::open(backup_path)?;
    let mut archive = ZipArchive::new(file)?;
    preview_import_archive(&mut archive, false, path_str)
}

fn import_backup_archive<R: Read + std::io::Seek>(
    db: &Database,
    archive: &mut ZipArchive<R>,
) -> Result<ImportSummary, BackupError> {
    validate_zip_archive_limits(archive)?;

    let version = read_json_from_zip::<BackupVersion, _>(archive, "version.json")?;
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

    let drafts: Vec<SavedDraft> = read_json_from_zip(archive, "drafts.json")?;
    for draft in drafts {
        if db.get_draft(&draft.id).is_err() {
            db.save_draft(&draft)
                .map_err(|e| BackupError::Database(e.to_string()))?;
            drafts_imported += 1;
        }
    }

    let templates: Vec<ResponseTemplate> = read_json_from_zip(archive, "templates.json")?;
    for template in templates {
        if db.get_template(&template.id).is_err() {
            db.save_template(&template)
                .map_err(|e| BackupError::Database(e.to_string()))?;
            templates_imported += 1;
        }
    }

    let variables: Vec<CustomVariable> = read_json_from_zip(archive, "variables.json")?;
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

    let trees: Vec<DecisionTree> = read_json_from_zip(archive, "trees.json")?;
    for tree in trees {
        if db.get_decision_tree(&tree.id).is_err() {
            db.save_decision_tree(&tree)
                .map_err(|e| BackupError::Database(e.to_string()))?;
            trees_imported += 1;
        }
    }

    if let Ok(settings) = read_json_from_zip::<SettingsExport, _>(archive, "settings.json") {
        for entry in settings.entries {
            if entry.key != "schema_version" {
                import_setting(db, &entry.key, &entry.value)?;
            }
        }
    }

    if let Ok(kb_config) = read_json_from_zip::<KbConfig, _>(archive, "kb_config.json") {
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

/// Import data from a backup file (with optional password for encrypted backups)
/// Merge strategy: insert new, skip existing (by ID)
pub fn import_backup(
    db: &Database,
    backup_path: &Path,
    password: Option<&str>,
) -> Result<ImportSummary, BackupError> {
    validate_backup_file_size(backup_path)?;

    if is_encrypted_backup(backup_path)?.is_some() {
        let pwd = password.ok_or(BackupError::EncryptionRequired)?;
        let zip_data = decrypt_backup(backup_path, pwd)?;
        let cursor = Cursor::new(zip_data);
        let mut archive = ZipArchive::new(cursor)?;
        return import_backup_archive(db, &mut archive);
    }

    let file = File::open(backup_path)?;
    let mut archive = ZipArchive::new(file)?;
    import_backup_archive(db, &mut archive)
}

#[derive(Debug)]
struct ArchivedDatabaseFile {
    original_path: PathBuf,
    archived_path: PathBuf,
}

fn archived_database_paths(db_path: &Path) -> Vec<PathBuf> {
    let mut paths = vec![db_path.to_path_buf()];

    if let Some(path_str) = db_path.to_str() {
        paths.push(PathBuf::from(format!("{}-wal", path_str)));
        paths.push(PathBuf::from(format!("{}-shm", path_str)));
    }

    paths
}

fn archive_existing_database_files(db_path: &Path) -> Result<Vec<ArchivedDatabaseFile>, BackupError> {
    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
    let mut archived = Vec::new();

    for path in archived_database_paths(db_path) {
        if !path.exists() {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| BackupError::InvalidBackup("Invalid database file path".to_string()))?;
        let archived_path = path.with_file_name(format!("{}.recovery-{}", file_name, timestamp));
        fs::rename(&path, &archived_path)?;
        archived.push(ArchivedDatabaseFile {
            original_path: path,
            archived_path,
        });
    }

    Ok(archived)
}

fn restore_archived_database_files(
    archived_files: &[ArchivedDatabaseFile],
) -> Result<(), BackupError> {
    for archived in archived_files.iter().rev() {
        if archived.original_path.exists() {
            fs::remove_file(&archived.original_path)?;
        }
        fs::rename(&archived.archived_path, &archived.original_path)?;
    }
    Ok(())
}

fn cleanup_database_files(db_path: &Path) -> Result<(), BackupError> {
    for path in archived_database_paths(db_path) {
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

/// Restore backup data into a fresh encrypted database, preserving the previous files
/// under a timestamped `.recovery-*` suffix for manual rollback if needed.
pub fn restore_backup_into_fresh_database(
    db_path: &Path,
    master_key: &crate::security::MasterKey,
    backup_path: &Path,
    password: Option<&str>,
) -> Result<ImportSummary, BackupError> {
    let archived = archive_existing_database_files(db_path)?;

    let restore_result = (|| {
        let db = Database::open(db_path, master_key).map_err(|e| BackupError::Database(e.to_string()))?;
        db.initialize()
            .map_err(|e| BackupError::Database(e.to_string()))?;
        import_backup(&db, backup_path, password)
    })();

    match restore_result {
        Ok(summary) => Ok(summary),
        Err(error) => {
            let _ = cleanup_database_files(db_path);
            let _ = restore_archived_database_files(&archived);
            Err(error)
        }
    }
}

/// Helper: Read JSON from a ZIP archive (generic over reader type)
fn read_json_from_zip<T: serde::de::DeserializeOwned, R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    filename: &str,
) -> Result<T, BackupError> {
    let mut file = archive.by_name(filename)?;
    if file.size() > MAX_ZIP_JSON_ENTRY_BYTES {
        return Err(BackupError::InvalidBackup(format!(
            "Backup entry too large: {} ({} bytes)",
            filename,
            file.size()
        )));
    }
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    if contents.len() as u64 > MAX_ZIP_JSON_ENTRY_BYTES {
        return Err(BackupError::InvalidBackup(format!(
            "Backup entry too large after decompression: {} ({} bytes)",
            filename,
            contents.len()
        )));
    }
    Ok(serde_json::from_str(&contents)?)
}

fn is_suspicious_zip_path(path: &str) -> bool {
    // ZIP paths are always `/` separated. Keep this conservative: no absolute paths,
    // no parent traversal, and no backslashes (Windows separator) to avoid ambiguity.
    //
    // Defense in depth: percent-decode first to catch encoded traversal like %2e%2e.
    let decoded = percent_decode_lossy(path);

    if decoded.starts_with('/') || decoded.contains('\\') {
        return true;
    }
    decoded.split('/').any(|part| part == "..")
}

fn percent_decode_lossy(input: &str) -> String {
    // Minimal percent-decoder for ZIP entry names. This is intentionally small
    // (no new dependency) and only exists to catch encoded traversal like `%2e%2e`.
    fn hex_val(b: u8) -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        }
    }

    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h1), Some(h2)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push((h1 << 4) | h2);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn validate_zip_archive_limits<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<(), BackupError> {
    let entry_count = archive.len();
    if entry_count > MAX_ZIP_ENTRIES {
        return Err(BackupError::InvalidBackup(format!(
            "Backup ZIP has too many entries: {} (max {})",
            entry_count, MAX_ZIP_ENTRIES
        )));
    }

    let mut total_uncompressed: u64 = 0;
    for i in 0..entry_count {
        let file = archive.by_index(i)?;
        let name = file.name().to_string();
        if is_suspicious_zip_path(&name) {
            return Err(BackupError::InvalidBackup(format!(
                "Backup ZIP contains suspicious path: {}",
                name
            )));
        }

        let size = file.size();
        if size > MAX_ZIP_ENTRY_UNCOMPRESSED_BYTES {
            return Err(BackupError::InvalidBackup(format!(
                "Backup ZIP entry too large: {} ({} bytes)",
                name, size
            )));
        }
        total_uncompressed = total_uncompressed.saturating_add(size);
        if total_uncompressed > MAX_ZIP_TOTAL_UNCOMPRESSED_BYTES {
            return Err(BackupError::InvalidBackup(format!(
                "Backup ZIP total uncompressed size too large: {} bytes (max {} bytes)",
                total_uncompressed, MAX_ZIP_TOTAL_UNCOMPRESSED_BYTES
            )));
        }
    }

    Ok(())
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
    // Defense in depth: reject imported KB paths that violate home/sensitive-dir validation.
    // Backups may come from untrusted sources, so settings that drive filesystem access
    // must be re-validated before persistence.
    let Some(safe_value) = sanitize_imported_setting(key, value) else {
        return Ok(());
    };

    let conn = db.conn();
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
        rusqlite::params![key, safe_value],
    )
    .map_err(|e| BackupError::Database(e.to_string()))?;
    Ok(())
}

fn sanitize_imported_setting(key: &str, value: &str) -> Option<String> {
    if key != "kb_folder" {
        return Some(value.to_string());
    }

    let path = Path::new(value);
    if !path.is_dir() {
        tracing::warn!("Skipped imported kb_folder because directory does not exist");
        return None;
    }

    match validate_within_home(path) {
        Ok(validated) => Some(validated.to_string_lossy().to_string()),
        Err(err) => {
            tracing::warn!("Skipped unsafe imported kb_folder value: {}", err);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::sanitize_imported_setting;
    use super::{validate_zip_archive_limits, MAX_ZIP_ENTRIES};
    use std::io::Cursor;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::{ZipArchive, ZipWriter};

    #[test]
    fn sanitize_imported_setting_accepts_non_kb_keys() {
        let result = sanitize_imported_setting("theme", "dark");
        assert_eq!(result, Some("dark".to_string()));
    }

    #[test]
    fn sanitize_imported_setting_rejects_outside_home_kb_path() {
        let result = sanitize_imported_setting("kb_folder", "/etc");
        assert!(result.is_none());
    }

    #[test]
    fn sanitize_imported_setting_rejects_missing_kb_path() {
        let result = sanitize_imported_setting("kb_folder", "/path/that/does/not/exist");
        assert!(result.is_none());
    }

    #[test]
    fn validate_zip_archive_limits_rejects_too_many_entries() {
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut writer = ZipWriter::new(cursor);
            let opts = SimpleFileOptions::default();
            // Create MAX_ZIP_ENTRIES + 1 empty files without writing large data.
            for i in 0..(MAX_ZIP_ENTRIES + 1) {
                writer.start_file(format!("f{}.txt", i), opts).unwrap();
            }
            writer.finish().unwrap();
        }

        let cursor = Cursor::new(buf);
        let mut archive = ZipArchive::new(cursor).unwrap();
        let result = validate_zip_archive_limits(&mut archive);
        assert!(
            result.is_err(),
            "Expected archive to be rejected for too many entries"
        );
    }

    #[test]
    fn validate_zip_archive_limits_rejects_url_encoded_traversal_path() {
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut writer = ZipWriter::new(cursor);
            let opts = SimpleFileOptions::default();

            // Include required minimal file so ZipArchive is well-formed.
            writer.start_file("version.json", opts).unwrap();
            writer
                .write_all(br#"{"version":"1","created_at":"now","app_version":"1"}"#)
                .unwrap();

            // Encoded ".." should be rejected.
            writer.start_file("%2e%2e/evil.json", opts).unwrap();
            writer.write_all(b"{}").unwrap();

            writer.finish().unwrap();
        }

        let cursor = Cursor::new(buf);
        let mut archive = ZipArchive::new(cursor).unwrap();
        let result = validate_zip_archive_limits(&mut archive);
        assert!(
            result.is_err(),
            "Expected archive to be rejected for encoded traversal"
        );
    }
}
