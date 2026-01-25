//! Common test utilities for AssistSupport integration tests
//!
//! Provides helper functions for creating test databases, temp directories,
//! and mock data for integration testing.

use assistsupport_lib::db::Database;
use assistsupport_lib::security::MasterKey;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test context holding temporary resources
#[allow(dead_code)]
pub struct TestContext {
    pub temp_dir: TempDir,
    pub db: Database,
    pub db_path: PathBuf,
}

#[allow(dead_code)]
impl TestContext {
    /// Create a new test context with an encrypted database
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let key = MasterKey::generate();
        let db = Database::open(&db_path, &key)?;
        db.initialize()?;

        Ok(Self {
            temp_dir,
            db,
            db_path,
        })
    }

    /// Create a test context with a specific database name
    pub fn with_name(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join(format!("{}.db", name));
        let key = MasterKey::generate();
        let db = Database::open(&db_path, &key)?;
        db.initialize()?;

        Ok(Self {
            temp_dir,
            db,
            db_path,
        })
    }

    /// Get the temp directory path
    pub fn temp_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Create a test file with content
    pub fn create_test_file(
        &self,
        name: &str,
        content: &str,
    ) -> Result<PathBuf, std::io::Error> {
        let path = self.temp_dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        Ok(path)
    }

    /// Create multiple test files in a subdirectory
    pub fn create_test_files(
        &self,
        dir_name: &str,
        files: &[(&str, &str)],
    ) -> Result<PathBuf, std::io::Error> {
        let dir_path = self.temp_dir.path().join(dir_name);
        std::fs::create_dir_all(&dir_path)?;

        for (name, content) in files {
            let file_path = dir_path.join(name);
            std::fs::write(file_path, content)?;
        }

        Ok(dir_path)
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new().expect("Failed to create test context")
    }
}

/// Create a simple test database without full context
#[allow(dead_code)]
pub fn create_test_db() -> Result<(TempDir, Database), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let key = MasterKey::generate();
    let db = Database::open(&db_path, &key)?;
    db.initialize()?;
    Ok((temp_dir, db))
}

/// Generate test markdown content
#[allow(dead_code)]
pub fn generate_test_markdown(title: &str, content: &str) -> String {
    format!("# {}\n\n{}", title, content)
}

/// Generate a large test document for stress testing
pub fn generate_large_document(words: usize) -> String {
    let base = "This is test content for stress testing the indexer. ";
    let word_count = base.split_whitespace().count();
    let repetitions = words / word_count + 1;
    base.repeat(repetitions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = TestContext::new().unwrap();
        assert!(ctx.db_path.exists());
    }

    #[test]
    fn test_file_creation() {
        let ctx = TestContext::new().unwrap();
        let path = ctx.create_test_file("test.md", "# Test\nContent").unwrap();
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# Test"));
    }

    #[test]
    fn test_generate_large_document() {
        let doc = generate_large_document(10000);
        let word_count = doc.split_whitespace().count();
        assert!(word_count >= 10000);
    }
}
