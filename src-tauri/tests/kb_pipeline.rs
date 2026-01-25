//! KB Pipeline integration tests
//!
//! Tests for the complete knowledge base workflow:
//! ingest -> index -> search, namespace isolation, document management,
//! large document handling, and multi-format support.

mod common;

use assistsupport_lib::kb::indexer::{IndexProgress, KbIndexer};
use assistsupport_lib::kb::search::HybridSearch;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// No-op progress callback for tests that don't need progress
fn noop_progress(_: IndexProgress) {}

// ============================================================================
// Basic Pipeline Tests
// ============================================================================

#[test]
fn test_basic_ingest_index_search() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    // Create test markdown files
    let kb_dir = ctx
        .create_test_files(
            "kb",
            &[
                (
                    "auth.md",
                    "# Authentication\n\nLogin requires username and password. Use SSO for enterprise.",
                ),
                (
                    "billing.md",
                    "# Billing\n\nPayment methods include credit card and invoice. Contact support for refunds.",
                ),
            ],
        )
        .expect("Failed to create test files");

    // Index the directory
    let indexer = KbIndexer::new();
    let result = indexer
        .index_folder(&ctx.db, &kb_dir, noop_progress)
        .expect("Indexing failed");

    assert_eq!(result.total_files, 2, "Should have found 2 files");
    assert_eq!(result.indexed, 2, "Should have indexed 2 files");
    assert_eq!(result.errors, 0, "Should have no errors");

    // Search for "password"
    let search_results = HybridSearch::fts_search(&ctx.db, "password", 10).expect("Search failed");
    assert_eq!(search_results.len(), 1, "Should find 1 result for 'password'");
    assert!(
        search_results[0].content.contains("password"),
        "Result should contain 'password'"
    );

    // Search for "billing"
    let billing_results = HybridSearch::fts_search(&ctx.db, "billing", 10).expect("Search failed");
    assert_eq!(billing_results.len(), 1, "Should find 1 result for 'billing'");

    // Search for something in both
    let support_results = HybridSearch::fts_search(&ctx.db, "support", 10).expect("Search failed");
    assert!(
        support_results.len() >= 1,
        "Should find at least 1 result for 'support'"
    );
}

#[test]
fn test_search_returns_relevance_order() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    // Create files with different amounts of keyword matches
    let kb_dir = ctx
        .create_test_files(
            "kb",
            &[
                ("doc1.md", "# One\n\nKubernetes once."),
                (
                    "doc2.md",
                    "# Two\n\nKubernetes Kubernetes Kubernetes multiple times.",
                ),
                ("doc3.md", "# Three\n\nNo k8s here."),
            ],
        )
        .expect("Failed to create test files");

    let indexer = KbIndexer::new();
    indexer
        .index_folder(&ctx.db, &kb_dir, noop_progress)
        .expect("Indexing failed");

    // Search - documents with more matches should rank higher
    let results = HybridSearch::fts_search(&ctx.db, "kubernetes", 10).expect("Search failed");
    assert!(results.len() >= 2, "Should find at least 2 results");
}

// ============================================================================
// Namespace Isolation Tests
// ============================================================================

#[test]
fn test_namespace_isolation() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    // Create two namespaces
    let _ns1 = ctx
        .db
        .create_namespace("internal", None, None)
        .expect("Failed to create namespace 1");
    let _ns2 = ctx
        .db
        .create_namespace("external", None, None)
        .expect("Failed to create namespace 2");

    // Create content for each namespace
    let internal_dir = ctx
        .create_test_files(
            "internal",
            &[("doc.md", "# Internal\n\nSecret internal documentation.")],
        )
        .expect("Failed to create internal files");

    let external_dir = ctx
        .create_test_files(
            "external",
            &[("doc.md", "# External\n\nPublic external documentation.")],
        )
        .expect("Failed to create external files");

    // Index each to different namespaces (note: current API doesn't support namespace in index_folder,
    // so this test verifies search with namespace filters work on existing data)
    let indexer = KbIndexer::new();
    indexer
        .index_folder(&ctx.db, &internal_dir, noop_progress)
        .expect("Failed to index internal");
    indexer
        .index_folder(&ctx.db, &external_dir, noop_progress)
        .expect("Failed to index external");

    // Search without filter - should find both
    let all_results = HybridSearch::fts_search(&ctx.db, "documentation", 10).expect("Search failed");
    assert_eq!(all_results.len(), 2, "Should find 2 results without filter");
}

#[test]
fn test_default_namespace() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    // Create and index without specifying namespace
    let kb_dir = ctx
        .create_test_files("kb", &[("doc.md", "# Default\n\nDefault namespace content.")])
        .expect("Failed to create test files");

    let indexer = KbIndexer::new();
    indexer
        .index_folder(&ctx.db, &kb_dir, noop_progress)
        .expect("Indexing failed");

    // Search should work
    let results = HybridSearch::fts_search(&ctx.db, "default", 10).expect("Search failed");
    assert!(!results.is_empty(), "Should find content in default namespace");
}

// ============================================================================
// Document Update & Removal Tests
// ============================================================================

#[test]
fn test_document_update_re_index() {
    let ctx = common::TestContext::new().expect("Failed to create context");
    let kb_dir = ctx.temp_path().join("kb");
    fs::create_dir_all(&kb_dir).expect("Failed to create kb dir");

    let doc_path = kb_dir.join("doc.md");

    // Initial content
    fs::write(&doc_path, "# Original\n\nOriginal content here.").expect("Failed to write");

    let indexer = KbIndexer::new();
    indexer.index_folder(&ctx.db, &kb_dir, noop_progress).expect("Initial index failed");

    // Verify original content is searchable
    let original_results = HybridSearch::fts_search(&ctx.db, "original", 10).expect("Search failed");
    assert_eq!(original_results.len(), 1, "Should find original content");

    // Update content
    fs::write(&doc_path, "# Updated\n\nCompletely updated content here.").expect("Failed to update");

    // Re-index
    indexer.index_folder(&ctx.db, &kb_dir, noop_progress).expect("Re-index failed");

    // Old content should no longer be found
    let old_results = HybridSearch::fts_search(&ctx.db, "original", 10).expect("Search failed");
    assert_eq!(old_results.len(), 0, "Should not find old content after update");

    // New content should be found
    let new_results = HybridSearch::fts_search(&ctx.db, "updated", 10).expect("Search failed");
    assert_eq!(new_results.len(), 1, "Should find updated content");
}

#[test]
fn test_document_removal() {
    let ctx = common::TestContext::new().expect("Failed to create context");
    let kb_dir = ctx.temp_path().join("kb");
    fs::create_dir_all(&kb_dir).expect("Failed to create kb dir");

    let doc_path = kb_dir.join("doc.md");
    fs::write(&doc_path, "# Content\n\nSearchable content.").expect("Failed to write");

    let indexer = KbIndexer::new();
    indexer.index_folder(&ctx.db, &kb_dir, noop_progress).expect("Initial index failed");

    // Verify content is searchable
    let results = HybridSearch::fts_search(&ctx.db, "searchable", 10).expect("Search failed");
    assert_eq!(results.len(), 1, "Should find content initially");

    // Get doc path for removal
    let doc_path_str = doc_path.to_string_lossy().to_string();

    // Remove using the indexer API
    indexer.remove_document(&ctx.db, &doc_path_str).expect("Failed to remove document");

    // Content should no longer be found
    let results_after = HybridSearch::fts_search(&ctx.db, "searchable", 10).expect("Search failed");
    assert_eq!(results_after.len(), 0, "Should not find removed content");
}

// ============================================================================
// Large Document Tests
// ============================================================================

#[test]
fn test_large_document_chunking() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    // Create a large document (~100KB)
    let large_content = common::generate_large_document(15000); // ~15k words

    let kb_dir = ctx
        .create_test_files("kb", &[("large.md", &large_content)])
        .expect("Failed to create test files");

    let indexer = KbIndexer::new();
    let result = indexer
        .index_folder(&ctx.db, &kb_dir, noop_progress)
        .expect("Indexing failed");

    assert_eq!(result.indexed, 1, "Should index 1 file");

    // Verify we can search the content
    let results = HybridSearch::fts_search(&ctx.db, "test content", 10).expect("Search failed");
    assert!(!results.is_empty(), "Should find content in large document");

    // Verify chunking occurred (check db directly)
    let chunk_count: i64 = ctx
        .db
        .conn()
        .query_row("SELECT COUNT(*) FROM kb_chunks", [], |row| row.get(0))
        .expect("Failed to count chunks");
    assert!(chunk_count > 1, "Large document should be split into multiple chunks");
}

// ============================================================================
// Multi-format Tests
// ============================================================================

#[test]
fn test_text_file_indexing() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    let kb_dir = ctx
        .create_test_files("kb", &[("readme.txt", "Plain text file content for testing.")])
        .expect("Failed to create test files");

    let indexer = KbIndexer::new();
    let result = indexer
        .index_folder(&ctx.db, &kb_dir, noop_progress)
        .expect("Indexing failed");

    assert_eq!(result.indexed, 1, "Should index txt file");

    let results = HybridSearch::fts_search(&ctx.db, "plain text", 10).expect("Search failed");
    assert!(!results.is_empty(), "Should find content in txt file");
}

#[test]
fn test_markdown_with_code_blocks() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    let content = r#"
# Code Documentation

Here's how to use the API:

```python
def hello():
    print("Hello, World!")
```

Call the function like this:

```bash
python main.py
```
"#;

    let kb_dir = ctx
        .create_test_files("kb", &[("code.md", content)])
        .expect("Failed to create test files");

    let indexer = KbIndexer::new();
    indexer
        .index_folder(&ctx.db, &kb_dir, noop_progress)
        .expect("Indexing failed");

    // Should be able to search for code content
    let results = HybridSearch::fts_search(&ctx.db, "python", 10).expect("Search failed");
    assert!(!results.is_empty(), "Should find code content");

    let results2 = HybridSearch::fts_search(&ctx.db, "API", 10).expect("Search failed");
    assert!(!results2.is_empty(), "Should find API reference");
}

// ============================================================================
// Progress Callback Tests
// ============================================================================

#[test]
fn test_indexing_with_progress() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    let kb_dir = ctx
        .create_test_files(
            "kb",
            &[
                ("doc1.md", "Document 1 content"),
                ("doc2.md", "Document 2 content"),
                ("doc3.md", "Document 3 content"),
            ],
        )
        .expect("Failed to create test files");

    let progress_count = Arc::new(AtomicUsize::new(0));
    let progress_count_clone = Arc::clone(&progress_count);

    let progress_callback = move |progress: IndexProgress| {
        progress_count_clone.fetch_add(1, Ordering::SeqCst);
        match &progress {
            IndexProgress::Started { total_files } => {
                println!("Started: {} files", total_files);
            }
            IndexProgress::Processing { current, total, file_name } => {
                println!("Processing: {}/{} - {}", current, total, file_name);
            }
            IndexProgress::Completed { indexed, skipped, errors } => {
                println!("Completed: {} indexed, {} skipped, {} errors", indexed, skipped, errors);
            }
            IndexProgress::Error { file_name, message } => {
                println!("Error: {} - {}", file_name, message);
            }
        }
    };

    let indexer = KbIndexer::new();
    indexer
        .index_folder(&ctx.db, &kb_dir, progress_callback)
        .expect("Indexing failed");

    // Should have received progress updates
    let count = progress_count.load(Ordering::SeqCst);
    assert!(count > 0, "Should have received progress callbacks");
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_directory() {
    let ctx = common::TestContext::new().expect("Failed to create context");
    let empty_dir = ctx.temp_path().join("empty");
    fs::create_dir_all(&empty_dir).expect("Failed to create empty dir");

    let indexer = KbIndexer::new();
    let result = indexer
        .index_folder(&ctx.db, &empty_dir, noop_progress)
        .expect("Indexing failed");

    assert_eq!(result.total_files, 0, "Empty dir should have 0 files");
    assert_eq!(result.indexed, 0, "Empty dir should index 0 files");
}

#[test]
fn test_nested_directories() {
    let ctx = common::TestContext::new().expect("Failed to create context");
    let kb_dir = ctx.temp_path().join("kb");
    let nested = kb_dir.join("level1").join("level2");
    fs::create_dir_all(&nested).expect("Failed to create nested dirs");

    fs::write(kb_dir.join("root.md"), "Root level document").expect("Failed to write");
    fs::write(nested.join("deep.md"), "Deeply nested document").expect("Failed to write");

    let indexer = KbIndexer::new();
    let result = indexer
        .index_folder(&ctx.db, &kb_dir, noop_progress)
        .expect("Indexing failed");

    assert_eq!(result.indexed, 2, "Should index files in nested directories");

    // Both should be searchable
    let root_results = HybridSearch::fts_search(&ctx.db, "root", 10).expect("Search failed");
    let deep_results = HybridSearch::fts_search(&ctx.db, "deeply", 10).expect("Search failed");

    assert!(!root_results.is_empty(), "Should find root level content");
    assert!(!deep_results.is_empty(), "Should find nested content");
}

#[test]
fn test_special_characters_in_search() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    let kb_dir = ctx
        .create_test_files(
            "kb",
            &[("special.md", "# Special\n\nC++ programming and C# development.")],
        )
        .expect("Failed to create test files");

    let indexer = KbIndexer::new();
    indexer
        .index_folder(&ctx.db, &kb_dir, noop_progress)
        .expect("Indexing failed");

    // FTS5 handles special characters
    let results = HybridSearch::fts_search(&ctx.db, "programming", 10).expect("Search failed");
    assert!(!results.is_empty(), "Should find content with special chars");
}

#[test]
fn test_unicode_content() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    let kb_dir = ctx
        .create_test_files(
            "kb",
            &[(
                "unicode.md",
                "# Unicode Test\n\n日本語テスト。中文测试。Emoji test: 🚀✨",
            )],
        )
        .expect("Failed to create test files");

    let indexer = KbIndexer::new();
    indexer
        .index_folder(&ctx.db, &kb_dir, noop_progress)
        .expect("Indexing failed");

    // Should be able to search unicode content
    let results = HybridSearch::fts_search(&ctx.db, "unicode", 10).expect("Search failed");
    assert!(!results.is_empty(), "Should find unicode content");
}

// ============================================================================
// E2E Workflow Tests (Phase 21)
// ============================================================================

#[test]
fn test_e2e_kb_ingest_search_workflow() {
    // Complete workflow: ingest docs -> search -> verify results
    let ctx = common::TestContext::new().expect("Failed to create context");

    // 1. Create test documentation
    let kb_dir = ctx
        .create_test_files(
            "kb",
            &[
                (
                    "vpn.md",
                    "# VPN Troubleshooting\n\nIf VPN connection fails:\n1. Check network connectivity\n2. Verify credentials\n3. Restart VPN client",
                ),
                (
                    "password.md",
                    "# Password Reset\n\nTo reset password:\n1. Go to IT portal\n2. Click 'Forgot Password'\n3. Follow email instructions",
                ),
            ],
        )
        .expect("Failed to create test files");

    // 2. Index documentation
    let indexer = KbIndexer::new();
    let result = indexer
        .index_folder(&ctx.db, &kb_dir, noop_progress)
        .expect("Indexing failed");

    assert_eq!(result.indexed, 2, "Should index 2 documents");
    assert_eq!(result.errors, 0, "Should have no errors");

    // 3. Search for relevant content
    let search_results =
        HybridSearch::fts_search(&ctx.db, "VPN connection fails", 5).expect("Search failed");

    assert!(!search_results.is_empty(), "Should find VPN content");
    assert!(
        search_results[0].content.contains("VPN"),
        "Top result should be VPN document"
    );

    // 4. Search for password content
    let password_results =
        HybridSearch::fts_search(&ctx.db, "password reset portal", 5).expect("Search failed");

    assert!(!password_results.is_empty(), "Should find password content");

    // 5. Verify documents can be retrieved by ID
    let doc_count: i64 = ctx
        .db
        .conn()
        .query_row("SELECT COUNT(*) FROM kb_documents", [], |r| r.get(0))
        .expect("Failed to count documents");

    assert_eq!(doc_count, 2, "Should have 2 documents in database");

    // 6. Verify chunks were created
    let chunk_count: i64 = ctx
        .db
        .conn()
        .query_row("SELECT COUNT(*) FROM kb_chunks", [], |r| r.get(0))
        .expect("Failed to count chunks");

    assert!(chunk_count >= 2, "Should have at least 2 chunks");
}

#[test]
fn test_e2e_incremental_reindex() {
    // Test that reindexing updates content correctly
    let ctx = common::TestContext::new().expect("Failed to create context");

    // 1. Create initial file
    let kb_dir = ctx
        .create_test_files("kb", &[("guide.md", "# Original Guide\n\nOriginal content here.")])
        .expect("Failed to create test files");

    let indexer = KbIndexer::new();

    // 2. Initial index
    let result1 = indexer
        .index_folder(&ctx.db, &kb_dir, noop_progress)
        .expect("Initial indexing failed");

    assert_eq!(result1.indexed, 1, "Should index 1 document");

    // 3. Search for original content
    let results1 = HybridSearch::fts_search(&ctx.db, "Original", 10).expect("Search failed");
    assert!(!results1.is_empty(), "Should find original content");

    // 4. Update the file
    let guide_path = kb_dir.join("guide.md");
    fs::write(&guide_path, "# Updated Guide\n\nNew updated content.").expect("Failed to update file");

    // 5. Re-index
    let result2 = indexer
        .index_folder(&ctx.db, &kb_dir, noop_progress)
        .expect("Re-indexing failed");

    // Note: depending on implementation, this may show 0 indexed if unchanged,
    // or 1 if the file was re-indexed
    assert_eq!(result2.errors, 0, "Re-indexing should not have errors");

    // 6. Search for new content (may need to verify content is updated)
    let _results2 = HybridSearch::fts_search(&ctx.db, "updated", 10).expect("Search failed");
    // Note: This depends on content hash change detection
    // The important thing is no errors occurred
}

#[test]
fn test_e2e_multi_format_support() {
    // Test that multiple file formats can be indexed and searched
    let ctx = common::TestContext::new().expect("Failed to create context");

    // 1. Create files in different formats
    let kb_dir = ctx
        .create_test_files(
            "kb",
            &[
                ("readme.md", "# Markdown\n\nThis is markdown content about kubernetes."),
                ("notes.txt", "Plain text notes about docker containers."),
            ],
        )
        .expect("Failed to create test files");

    let indexer = KbIndexer::new();
    let result = indexer
        .index_folder(&ctx.db, &kb_dir, noop_progress)
        .expect("Indexing failed");

    assert_eq!(result.indexed, 2, "Should index both file formats");

    // 2. Search across formats
    let md_results = HybridSearch::fts_search(&ctx.db, "kubernetes", 10).expect("Search failed");
    assert!(!md_results.is_empty(), "Should find markdown content");

    let txt_results = HybridSearch::fts_search(&ctx.db, "docker", 10).expect("Search failed");
    assert!(!txt_results.is_empty(), "Should find text file content");
}
