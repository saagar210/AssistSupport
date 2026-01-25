//! Data migration module for AssistSupport
//!
//! Handles automatic migration of data files from old storage paths to new ones.
//! The app previously used `com.d.assistsupport` as the data directory name but
//! changed to `AssistSupport` for consistency.
//!
//! This module migrates:
//! - assistsupport.db (SQLite database)
//! - vectors/ directory (LanceDB vector store)
//! - attachments/ directory (file attachments)
//! - audit.log (security audit log)

use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Old application data directory name (before standardization)
const OLD_APP_DIR_NAME: &str = "com.d.assistsupport";

/// New application data directory name (current)
const NEW_APP_DIR_NAME: &str = "AssistSupport";

/// Files and directories to migrate
const MIGRATE_ITEMS: &[&str] = &[
    "assistsupport.db",
    "assistsupport.db-shm",
    "assistsupport.db-wal",
    "vectors",
    "attachments",
    "audit.log",
    "models",
];

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Data directory not found")]
    DataDirNotFound,
    #[error("Conflict: both old and new paths have data")]
    Conflict(String),
}

/// Result of a migration operation
#[derive(Debug, Default)]
pub struct MigrationReport {
    /// Items that were successfully migrated
    pub migrated: Vec<MigratedItem>,
    /// Items that were skipped (already migrated or not found)
    pub skipped: Vec<SkippedItem>,
    /// Items that had conflicts
    pub conflicts: Vec<ConflictItem>,
    /// Whether any migration actually occurred
    pub migration_performed: bool,
}

#[derive(Debug)]
pub struct MigratedItem {
    pub name: String,
    pub old_path: PathBuf,
    pub new_path: PathBuf,
}

#[derive(Debug)]
pub struct SkippedItem {
    pub name: String,
    pub reason: String,
}

#[derive(Debug)]
pub struct ConflictItem {
    pub name: String,
    pub old_path: PathBuf,
    pub new_path: PathBuf,
    pub reason: String,
}

/// Get the old application data directory path
fn get_old_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join(OLD_APP_DIR_NAME))
}

/// Get the new application data directory path
fn get_new_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join(NEW_APP_DIR_NAME))
}

/// Check if a path has any data (exists and is not empty)
fn has_data(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    if path.is_file() {
        return true;
    }
    if path.is_dir() {
        if let Ok(mut entries) = fs::read_dir(path) {
            return entries.next().is_some();
        }
    }
    false
}

/// Migrate data directories from old path to new path
///
/// This function:
/// 1. Checks if old data directory exists
/// 2. Creates new data directory if needed
/// 3. Moves each item from old to new location
/// 4. Handles conflicts gracefully
///
/// The migration is idempotent - it can be safely called multiple times.
pub fn migrate_data_directories() -> Result<MigrationReport, MigrationError> {
    let mut report = MigrationReport::default();

    let old_dir = match get_old_data_dir() {
        Some(d) => d,
        None => {
            report.skipped.push(SkippedItem {
                name: "all".to_string(),
                reason: "Could not determine old data directory".to_string(),
            });
            return Ok(report);
        }
    };

    let new_dir = match get_new_data_dir() {
        Some(d) => d,
        None => {
            report.skipped.push(SkippedItem {
                name: "all".to_string(),
                reason: "Could not determine new data directory".to_string(),
            });
            return Ok(report);
        }
    };

    // If old directory doesn't exist, nothing to migrate
    if !old_dir.exists() {
        report.skipped.push(SkippedItem {
            name: "all".to_string(),
            reason: "Old data directory does not exist (already migrated or fresh install)"
                .to_string(),
        });
        return Ok(report);
    }

    // Create new directory if needed
    if !new_dir.exists() {
        fs::create_dir_all(&new_dir)?;
        tracing::info!("Created new data directory: {:?}", new_dir);
    }

    // Migrate each item
    for item_name in MIGRATE_ITEMS {
        let old_path = old_dir.join(item_name);
        let new_path = new_dir.join(item_name);

        // Skip if old item doesn't exist
        if !old_path.exists() {
            report.skipped.push(SkippedItem {
                name: item_name.to_string(),
                reason: "Does not exist in old location".to_string(),
            });
            continue;
        }

        // Check for conflict (both exist with data)
        if has_data(&new_path) && has_data(&old_path) {
            report.conflicts.push(ConflictItem {
                name: item_name.to_string(),
                old_path: old_path.clone(),
                new_path: new_path.clone(),
                reason: "Both old and new locations contain data".to_string(),
            });
            tracing::warn!(
                "Migration conflict for '{}': both {:?} and {:?} have data",
                item_name,
                old_path,
                new_path
            );
            continue;
        }

        // Skip if already exists at new location
        if new_path.exists() && !has_data(&old_path) {
            report.skipped.push(SkippedItem {
                name: item_name.to_string(),
                reason: "Already exists at new location".to_string(),
            });
            continue;
        }

        // Perform migration
        match fs::rename(&old_path, &new_path) {
            Ok(()) => {
                tracing::info!("Migrated '{}': {:?} -> {:?}", item_name, old_path, new_path);
                report.migrated.push(MigratedItem {
                    name: item_name.to_string(),
                    old_path,
                    new_path,
                });
                report.migration_performed = true;
            }
            Err(e) => {
                // If rename fails (e.g., cross-filesystem), try copy+delete
                if old_path.is_dir() {
                    match copy_dir_recursive(&old_path, &new_path) {
                        Ok(()) => {
                            let _ = fs::remove_dir_all(&old_path);
                            tracing::info!(
                                "Migrated '{}' (copy): {:?} -> {:?}",
                                item_name,
                                old_path,
                                new_path
                            );
                            report.migrated.push(MigratedItem {
                                name: item_name.to_string(),
                                old_path,
                                new_path,
                            });
                            report.migration_performed = true;
                        }
                        Err(copy_err) => {
                            tracing::error!(
                                "Failed to migrate '{}': rename={}, copy={}",
                                item_name,
                                e,
                                copy_err
                            );
                            report.conflicts.push(ConflictItem {
                                name: item_name.to_string(),
                                old_path,
                                new_path,
                                reason: format!("Migration failed: {}", copy_err),
                            });
                        }
                    }
                } else {
                    match fs::copy(&old_path, &new_path) {
                        Ok(_) => {
                            let _ = fs::remove_file(&old_path);
                            tracing::info!(
                                "Migrated '{}' (copy): {:?} -> {:?}",
                                item_name,
                                old_path,
                                new_path
                            );
                            report.migrated.push(MigratedItem {
                                name: item_name.to_string(),
                                old_path,
                                new_path,
                            });
                            report.migration_performed = true;
                        }
                        Err(copy_err) => {
                            tracing::error!(
                                "Failed to migrate '{}': rename={}, copy={}",
                                item_name,
                                e,
                                copy_err
                            );
                            report.conflicts.push(ConflictItem {
                                name: item_name.to_string(),
                                old_path,
                                new_path,
                                reason: format!("Migration failed: {}", copy_err),
                            });
                        }
                    }
                }
            }
        }
    }

    // Try to remove old directory if empty
    if old_dir.exists() {
        if let Ok(entries) = fs::read_dir(&old_dir) {
            if entries.count() == 0 {
                let _ = fs::remove_dir(&old_dir);
                tracing::info!("Removed empty old data directory: {:?}", old_dir);
            }
        }
    }

    Ok(report)
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_has_data_empty_dir() {
        let temp = TempDir::new().unwrap();
        let empty_dir = temp.path().join("empty");
        fs::create_dir(&empty_dir).unwrap();
        assert!(!has_data(&empty_dir));
    }

    #[test]
    fn test_has_data_with_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        File::create(&file_path).unwrap().write_all(b"data").unwrap();
        assert!(has_data(&file_path));
    }

    #[test]
    fn test_has_data_dir_with_contents() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("has_stuff");
        fs::create_dir(&dir).unwrap();
        File::create(dir.join("file.txt")).unwrap();
        assert!(has_data(&dir));
    }

    #[test]
    fn test_has_data_nonexistent() {
        assert!(!has_data(Path::new("/nonexistent/path")));
    }

    #[test]
    fn test_copy_dir_recursive() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");

        // Create source structure
        fs::create_dir_all(src.join("subdir")).unwrap();
        File::create(src.join("file1.txt"))
            .unwrap()
            .write_all(b"content1")
            .unwrap();
        File::create(src.join("subdir/file2.txt"))
            .unwrap()
            .write_all(b"content2")
            .unwrap();

        // Copy
        copy_dir_recursive(&src, &dst).unwrap();

        // Verify
        assert!(dst.join("file1.txt").exists());
        assert!(dst.join("subdir/file2.txt").exists());
        assert_eq!(
            fs::read_to_string(dst.join("file1.txt")).unwrap(),
            "content1"
        );
    }
}
