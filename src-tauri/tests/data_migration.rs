//! Data Migration Tests
//!
//! Tests for the automatic migration of data from old storage paths to new ones.
//! The app changed from `com.d.assistsupport` to `AssistSupport`.

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// Items that should be migrated
const MIGRATE_ITEMS: &[&str] = &[
    "assistsupport.db",
    "assistsupport.db-shm",
    "assistsupport.db-wal",
    "vectors",
    "attachments",
    "audit.log",
    "models",
];

/// Helper function to check if a path has data
fn has_data(path: &std::path::Path) -> bool {
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

/// Simulates the migration logic
fn simulate_migration(
    old_dir: &std::path::Path,
    new_dir: &std::path::Path,
) -> MigrationResult {
    let mut result = MigrationResult::default();

    if !old_dir.exists() {
        result.skipped.push(("all".to_string(), "Old directory does not exist".to_string()));
        return result;
    }

    // Create new directory if needed
    if !new_dir.exists() {
        fs::create_dir_all(new_dir).expect("Failed to create new directory");
    }

    for item_name in MIGRATE_ITEMS {
        let old_path = old_dir.join(item_name);
        let new_path = new_dir.join(item_name);

        if !old_path.exists() {
            result.skipped.push((item_name.to_string(), "Does not exist".to_string()));
            continue;
        }

        if has_data(&new_path) && has_data(&old_path) {
            result.conflicts.push((item_name.to_string(), "Both locations have data".to_string()));
            continue;
        }

        if new_path.exists() && !has_data(&old_path) {
            result.skipped.push((item_name.to_string(), "Already at new location".to_string()));
            continue;
        }

        // Perform migration
        if old_path.is_dir() {
            copy_dir_recursive(&old_path, &new_path).expect("Failed to copy directory");
            fs::remove_dir_all(&old_path).expect("Failed to remove old directory");
        } else {
            fs::copy(&old_path, &new_path).expect("Failed to copy file");
            fs::remove_file(&old_path).expect("Failed to remove old file");
        }

        result.migrated.push((item_name.to_string(), new_path.clone()));
    }

    // Remove old directory if empty
    if let Ok(entries) = fs::read_dir(old_dir) {
        if entries.count() == 0 {
            let _ = fs::remove_dir(old_dir);
            result.old_dir_removed = true;
        }
    }

    result
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
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

#[derive(Default)]
struct MigrationResult {
    migrated: Vec<(String, PathBuf)>,
    skipped: Vec<(String, String)>,
    conflicts: Vec<(String, String)>,
    old_dir_removed: bool,
}

// ============================================================================
// Basic Migration Tests
// ============================================================================

#[test]
fn test_migration_moves_database() {
    let temp = TempDir::new().unwrap();
    let old_dir = temp.path().join("com.d.assistsupport");
    let new_dir = temp.path().join("AssistSupport");

    // Create old structure with database
    fs::create_dir_all(&old_dir).unwrap();
    let mut db_file = File::create(old_dir.join("assistsupport.db")).unwrap();
    db_file.write_all(b"SQLite database content").unwrap();

    // Run migration
    let result = simulate_migration(&old_dir, &new_dir);

    // Verify
    assert_eq!(result.migrated.len(), 1);
    assert_eq!(result.migrated[0].0, "assistsupport.db");
    assert!(new_dir.join("assistsupport.db").exists());
    assert!(!old_dir.join("assistsupport.db").exists());
}

#[test]
fn test_migration_moves_vectors_directory() {
    let temp = TempDir::new().unwrap();
    let old_dir = temp.path().join("old");
    let new_dir = temp.path().join("new");

    // Create old vectors directory with content
    let vectors_dir = old_dir.join("vectors");
    fs::create_dir_all(&vectors_dir).unwrap();
    File::create(vectors_dir.join("data.lance")).unwrap().write_all(b"lance data").unwrap();
    File::create(vectors_dir.join("index.idx")).unwrap().write_all(b"index data").unwrap();

    // Run migration
    let result = simulate_migration(&old_dir, &new_dir);

    // Verify directory and contents migrated
    assert!(result.migrated.iter().any(|(name, _)| name == "vectors"));
    assert!(new_dir.join("vectors").exists());
    assert!(new_dir.join("vectors/data.lance").exists());
    assert!(new_dir.join("vectors/index.idx").exists());
    assert!(!old_dir.join("vectors").exists());
}

#[test]
fn test_migration_moves_all_items() {
    let temp = TempDir::new().unwrap();
    let old_dir = temp.path().join("old");
    let new_dir = temp.path().join("new");

    // Create all migratable items
    fs::create_dir_all(&old_dir).unwrap();
    File::create(old_dir.join("assistsupport.db")).unwrap().write_all(b"db").unwrap();
    File::create(old_dir.join("assistsupport.db-shm")).unwrap().write_all(b"shm").unwrap();
    File::create(old_dir.join("assistsupport.db-wal")).unwrap().write_all(b"wal").unwrap();
    File::create(old_dir.join("audit.log")).unwrap().write_all(b"log").unwrap();

    fs::create_dir_all(old_dir.join("vectors")).unwrap();
    File::create(old_dir.join("vectors/data")).unwrap();

    fs::create_dir_all(old_dir.join("attachments")).unwrap();
    File::create(old_dir.join("attachments/file.pdf")).unwrap();

    fs::create_dir_all(old_dir.join("models")).unwrap();
    File::create(old_dir.join("models/model.gguf")).unwrap();

    // Run migration
    let result = simulate_migration(&old_dir, &new_dir);

    // All items should be migrated
    assert_eq!(result.migrated.len(), 7);
    assert!(result.conflicts.is_empty());

    // Old directory should be removed (was empty after migration)
    assert!(result.old_dir_removed);
}

// ============================================================================
// No-Op Tests
// ============================================================================

#[test]
fn test_migration_noop_when_old_dir_missing() {
    let temp = TempDir::new().unwrap();
    let old_dir = temp.path().join("nonexistent");
    let new_dir = temp.path().join("new");

    // Old directory doesn't exist
    let result = simulate_migration(&old_dir, &new_dir);

    assert!(result.migrated.is_empty());
    assert!(result.skipped.iter().any(|(name, _)| name == "all"));
}

#[test]
fn test_migration_noop_when_already_migrated() {
    let temp = TempDir::new().unwrap();
    let old_dir = temp.path().join("old");
    let new_dir = temp.path().join("new");

    // Create empty old directory
    fs::create_dir_all(&old_dir).unwrap();

    // Data already at new location
    fs::create_dir_all(&new_dir).unwrap();
    File::create(new_dir.join("assistsupport.db")).unwrap().write_all(b"data").unwrap();

    let result = simulate_migration(&old_dir, &new_dir);

    // assistsupport.db should be skipped (doesn't exist in old, exists in new)
    // Actually - old_dir/assistsupport.db doesn't exist, so it's skipped as "Does not exist"
    assert!(result.migrated.is_empty());
}

#[test]
fn test_migration_skips_items_not_in_old() {
    let temp = TempDir::new().unwrap();
    let old_dir = temp.path().join("old");
    let new_dir = temp.path().join("new");

    // Create old directory with only audit.log
    fs::create_dir_all(&old_dir).unwrap();
    File::create(old_dir.join("audit.log")).unwrap().write_all(b"log").unwrap();

    let result = simulate_migration(&old_dir, &new_dir);

    // Only audit.log should be migrated
    assert_eq!(result.migrated.len(), 1);
    assert_eq!(result.migrated[0].0, "audit.log");

    // Other items should be skipped (not in old)
    assert!(result.skipped.len() >= 6);
}

// ============================================================================
// Conflict Detection Tests
// ============================================================================

#[test]
fn test_migration_detects_conflict() {
    let temp = TempDir::new().unwrap();
    let old_dir = temp.path().join("old");
    let new_dir = temp.path().join("new");

    // Create same file in both locations
    fs::create_dir_all(&old_dir).unwrap();
    fs::create_dir_all(&new_dir).unwrap();
    File::create(old_dir.join("assistsupport.db")).unwrap().write_all(b"old data").unwrap();
    File::create(new_dir.join("assistsupport.db")).unwrap().write_all(b"new data").unwrap();

    let result = simulate_migration(&old_dir, &new_dir);

    // Should report conflict
    assert!(result.conflicts.iter().any(|(name, _)| name == "assistsupport.db"));

    // Both files should be untouched
    assert!(old_dir.join("assistsupport.db").exists());
    assert!(new_dir.join("assistsupport.db").exists());
}

#[test]
fn test_migration_handles_mixed_conflicts() {
    let temp = TempDir::new().unwrap();
    let old_dir = temp.path().join("old");
    let new_dir = temp.path().join("new");

    fs::create_dir_all(&old_dir).unwrap();
    fs::create_dir_all(&new_dir).unwrap();

    // Conflict: both have database
    File::create(old_dir.join("assistsupport.db")).unwrap().write_all(b"old").unwrap();
    File::create(new_dir.join("assistsupport.db")).unwrap().write_all(b"new").unwrap();

    // No conflict: only old has audit.log
    File::create(old_dir.join("audit.log")).unwrap().write_all(b"old log").unwrap();

    let result = simulate_migration(&old_dir, &new_dir);

    // One conflict, one migration
    assert_eq!(result.conflicts.len(), 1);
    assert_eq!(result.migrated.len(), 1);
    assert_eq!(result.migrated[0].0, "audit.log");
}

// ============================================================================
// Directory Structure Tests
// ============================================================================

#[test]
fn test_migration_preserves_nested_structure() {
    let temp = TempDir::new().unwrap();
    let old_dir = temp.path().join("old");
    let new_dir = temp.path().join("new");

    // Create nested directory structure
    let nested = old_dir.join("attachments/subdir/deep");
    fs::create_dir_all(&nested).unwrap();
    File::create(nested.join("file.txt")).unwrap().write_all(b"nested content").unwrap();
    File::create(old_dir.join("attachments/root.txt")).unwrap().write_all(b"root").unwrap();

    let _result = simulate_migration(&old_dir, &new_dir);

    // Verify structure preserved
    assert!(new_dir.join("attachments/subdir/deep/file.txt").exists());
    assert!(new_dir.join("attachments/root.txt").exists());

    // Verify content
    let content = fs::read_to_string(new_dir.join("attachments/subdir/deep/file.txt")).unwrap();
    assert_eq!(content, "nested content");
}

#[test]
fn test_migration_removes_empty_old_directory() {
    let temp = TempDir::new().unwrap();
    let old_dir = temp.path().join("old");
    let new_dir = temp.path().join("new");

    // Create old directory with just one file
    fs::create_dir_all(&old_dir).unwrap();
    File::create(old_dir.join("audit.log")).unwrap().write_all(b"log").unwrap();

    let result = simulate_migration(&old_dir, &new_dir);

    // Old directory should be removed
    assert!(result.old_dir_removed);
    assert!(!old_dir.exists());
}

#[test]
fn test_migration_keeps_old_directory_if_not_empty() {
    let temp = TempDir::new().unwrap();
    let old_dir = temp.path().join("old");
    let new_dir = temp.path().join("new");

    // Create old directory with migratable and non-migratable items
    fs::create_dir_all(&old_dir).unwrap();
    File::create(old_dir.join("audit.log")).unwrap().write_all(b"log").unwrap();
    File::create(old_dir.join("other_file.txt")).unwrap().write_all(b"other").unwrap();

    let result = simulate_migration(&old_dir, &new_dir);

    // Old directory should remain (still has other_file.txt)
    assert!(!result.old_dir_removed);
    assert!(old_dir.exists());
    assert!(old_dir.join("other_file.txt").exists());
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_has_data_on_nonexistent_path() {
    assert!(!has_data(std::path::Path::new("/nonexistent/path")));
}

#[test]
fn test_has_data_on_empty_directory() {
    let temp = TempDir::new().unwrap();
    let empty_dir = temp.path().join("empty");
    fs::create_dir(&empty_dir).unwrap();
    assert!(!has_data(&empty_dir));
}

#[test]
fn test_has_data_on_file() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("file.txt");
    File::create(&file_path).unwrap();
    assert!(has_data(&file_path));
}

#[test]
fn test_has_data_on_directory_with_contents() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path().join("has_stuff");
    fs::create_dir(&dir).unwrap();
    File::create(dir.join("file.txt")).unwrap();
    assert!(has_data(&dir));
}
