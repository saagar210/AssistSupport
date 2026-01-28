//! Integration tests for disk folder ingestion pipeline
//!
//! Tests the DiskIngester end-to-end: ingestion with source tracking,
//! search ranking with policy boost, incremental re-indexing, and
//! full pipeline verification.

mod common;

use assistsupport_lib::kb::ingest::disk::DiskIngester;
use assistsupport_lib::kb::search::{HybridSearch, SearchOptions};

// ============================================================================
// Basic Ingestion Tests
// ============================================================================

#[test]
fn test_disk_ingest_creates_documents_and_chunks() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    // Create KB folder with POLICIES/PROCEDURES/REFERENCE structure
    let kb_dir = ctx.temp_path().join("kb");
    std::fs::create_dir_all(kb_dir.join("POLICIES")).unwrap();
    std::fs::create_dir_all(kb_dir.join("PROCEDURES")).unwrap();
    std::fs::create_dir_all(kb_dir.join("REFERENCE")).unwrap();

    std::fs::write(
        kb_dir.join("POLICIES/removable_media.md"),
        r#"# Removable Media Policy

## Policy Statement

USB flash drives and external storage devices are strictly prohibited on all company endpoints.
This policy applies to all employees, contractors, and visitors without exception.

## Rationale

Removable media poses significant security risks including data exfiltration and malware introduction.

## Enforcement

Violations will be reported to Information Security and may result in disciplinary action.
"#,
    )
    .unwrap();

    std::fs::write(
        kb_dir.join("PROCEDURES/laptop_request.md"),
        r#"# Laptop Request Procedure

## How to Request a New Laptop

1. Open a ticket in the IT portal under "Hardware Requests"
2. Select your department and manager for approval
3. Choose from the approved laptop models
4. Wait for manager approval (typically 1-2 business days)
5. IT will contact you for setup and delivery

## Approved Models

- MacBook Pro 14" (Engineering)
- ThinkPad X1 Carbon (Business)
- Dell Latitude 5540 (General)
"#,
    )
    .unwrap();

    std::fs::write(
        kb_dir.join("REFERENCE/device_specs.md"),
        r#"# Device Specifications Reference

## Standard Laptop Specs

All standard laptops include:
- 16GB RAM minimum
- 512GB SSD
- Wi-Fi 6E
- USB-C ports
- 3-year warranty

## Peripherals

Standard peripherals provided: external monitor, keyboard, mouse.
"#,
    )
    .unwrap();

    ctx.db.ensure_namespace_exists("default").unwrap();

    let ingester = DiskIngester::new();
    let result = ingester
        .ingest_folder(&ctx.db, &kb_dir, "default")
        .unwrap();

    // Verify counts
    assert_eq!(result.total_files, 3, "Should find 3 files");
    assert_eq!(result.ingested, 3, "Should ingest 3 files");
    assert_eq!(result.errors, 0, "Should have no errors");
    assert_eq!(result.documents.len(), 3, "Should return 3 documents");

    // Verify documents in DB
    let doc_count: i64 = ctx
        .db
        .conn()
        .query_row("SELECT COUNT(*) FROM kb_documents", [], |row| row.get(0))
        .unwrap();
    assert_eq!(doc_count, 3, "Should have 3 documents in DB");

    // Verify chunks exist
    let chunk_count: i64 = ctx
        .db
        .conn()
        .query_row("SELECT COUNT(*) FROM kb_chunks", [], |row| row.get(0))
        .unwrap();
    assert!(chunk_count >= 3, "Should have at least 3 chunks");

    // Verify ingest_sources entries
    let source_count: i64 = ctx
        .db
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM ingest_sources WHERE source_type = 'file'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(source_count, 3, "Should have 3 ingest sources");

    // Verify ingest_runs entries
    let run_count: i64 = ctx
        .db
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM ingest_runs WHERE status = 'completed'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(run_count, 3, "Should have 3 completed ingest runs");

    // Verify documents have correct source_type and namespace_id
    let disk_docs: i64 = ctx
        .db
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM kb_documents WHERE source_type = 'file' AND namespace_id = 'default'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(disk_docs, 3, "All documents should have source_type='file' and namespace_id='default'");
}

// ============================================================================
// Search Ranking Tests
// ============================================================================

#[test]
fn test_policy_query_returns_policy_first() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    let kb_dir = ctx.temp_path().join("kb");
    std::fs::create_dir_all(kb_dir.join("POLICIES")).unwrap();
    std::fs::create_dir_all(kb_dir.join("PROCEDURES")).unwrap();
    std::fs::create_dir_all(kb_dir.join("REFERENCE")).unwrap();

    std::fs::write(
        kb_dir.join("POLICIES/flash_drives_forbidden.md"),
        r#"# Flash Drive and Removable Media Policy

## Policy

USB flash drives and removable storage devices are strictly forbidden on all company systems.
No exceptions are granted for flash drives, USB sticks, or external storage media.
This is a mandatory security policy enforced company-wide.
"#,
    )
    .unwrap();

    std::fs::write(
        kb_dir.join("PROCEDURES/file_sharing.md"),
        r#"# File Sharing Procedure

## How to Share Files

Use the company-approved cloud storage platform to share files securely.
If you need to transfer a flash drive's contents, upload via the secure portal instead.
Flash drive alternatives include OneDrive and SharePoint.
"#,
    )
    .unwrap();

    std::fs::write(
        kb_dir.join("REFERENCE/storage_devices.md"),
        r#"# Storage Devices Reference

## Approved Storage

Company-approved storage solutions:
- OneDrive for Business
- SharePoint
- Network file shares

USB flash drives are not on the approved list.
"#,
    )
    .unwrap();

    ctx.db.ensure_namespace_exists("default").unwrap();

    let ingester = DiskIngester::new();
    ingester
        .ingest_folder(&ctx.db, &kb_dir, "default")
        .unwrap();

    // Use fts_search for basic retrieval, then post_process for policy boost
    let results = HybridSearch::fts_search(&ctx.db, "flash drive forbidden", 10).unwrap();
    assert!(!results.is_empty(), "Should return search results");

    // Apply policy boost via post-processing
    let options = SearchOptions::new(10).with_query_text("Can I use a flash drive?");
    let results = HybridSearch::post_process_results(results, &options);

    // The first result should be from POLICIES/ due to policy boost
    let first_path = &results[0].file_path;
    assert!(
        first_path.contains("POLICIES"),
        "First result should be from POLICIES/ directory, got: {}",
        first_path
    );
}

#[test]
fn test_procedure_query_returns_procedure_first() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    let kb_dir = ctx.temp_path().join("kb");
    std::fs::create_dir_all(kb_dir.join("POLICIES")).unwrap();
    std::fs::create_dir_all(kb_dir.join("PROCEDURES")).unwrap();

    std::fs::write(
        kb_dir.join("POLICIES/hardware_policy.md"),
        r#"# Hardware Policy

## Approved Equipment

Only IT-approved hardware may be connected to the corporate network.
Laptop requests must follow the standard procurement process.
"#,
    )
    .unwrap();

    std::fs::write(
        kb_dir.join("PROCEDURES/laptop_request.md"),
        r#"# How to Request a Laptop

## Laptop Request Steps

1. Open a ticket in the IT portal
2. Select "Hardware Request" category
3. Choose laptop model
4. Get manager approval
5. IT will prepare and deliver the laptop

## Standard Processing Time

Laptop requests are typically fulfilled within 3-5 business days.
"#,
    )
    .unwrap();

    ctx.db.ensure_namespace_exists("default").unwrap();

    let ingester = DiskIngester::new();
    ingester
        .ingest_folder(&ctx.db, &kb_dir, "default")
        .unwrap();

    // Procedural query — should NOT trigger policy boost
    let results = HybridSearch::fts_search(&ctx.db, "request laptop", 10).unwrap();
    assert!(!results.is_empty(), "Should return search results");

    // Apply post-processing (policy boost should NOT fire for this query)
    let options = SearchOptions::new(10).with_query_text("How do I request a laptop?");
    let results = HybridSearch::post_process_results(results, &options);

    // For a procedural "how to" query, the procedure document should rank first
    let first_path = &results[0].file_path;
    assert!(
        first_path.contains("PROCEDURES"),
        "First result for procedural query should be from PROCEDURES/, got: {}",
        first_path
    );
}

#[test]
fn test_policy_boost_with_real_indexed_data() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    let kb_dir = ctx.temp_path().join("kb");
    std::fs::create_dir_all(kb_dir.join("POLICIES")).unwrap();
    std::fs::create_dir_all(kb_dir.join("PROCEDURES")).unwrap();

    std::fs::write(
        kb_dir.join("POLICIES/usb_prohibited.md"),
        r#"# USB Device Policy

USB flash drives, thumb drives, and external USB storage are prohibited.
This mandatory security policy prevents data loss and malware infection.
No exceptions for USB removable media. Use cloud storage instead.
"#,
    )
    .unwrap();

    std::fs::write(
        kb_dir.join("PROCEDURES/data_transfer.md"),
        r#"# Data Transfer Procedure

To transfer files without USB drives:
1. Upload to OneDrive
2. Share via secure link
3. Download on target device

USB drives are not permitted. Use approved cloud platforms.
"#,
    )
    .unwrap();

    ctx.db.ensure_namespace_exists("default").unwrap();

    let ingester = DiskIngester::new();
    ingester
        .ingest_folder(&ctx.db, &kb_dir, "default")
        .unwrap();

    // Search with policy boost
    let options_with_boost =
        SearchOptions::new(10).with_query_text("Are USB drives allowed?");
    let results_boosted =
        HybridSearch::search_with_options(&ctx.db, "USB drives allowed", options_with_boost)
            .unwrap();

    // Search without policy boost (no query_text)
    let options_no_boost = SearchOptions::new(10);
    let results_no_boost =
        HybridSearch::search_with_options(&ctx.db, "USB drives allowed", options_no_boost).unwrap();

    assert!(!results_boosted.is_empty(), "Boosted search should have results");
    assert!(!results_no_boost.is_empty(), "Non-boosted search should have results");

    // With policy boost, POLICIES/ result should be first
    assert!(
        results_boosted[0].file_path.contains("POLICIES"),
        "With policy boost, first result should be from POLICIES/, got: {}",
        results_boosted[0].file_path
    );

    // Verify the policy result has a boosted score compared to no-boost
    let boosted_policy_score = results_boosted
        .iter()
        .find(|r| r.file_path.contains("POLICIES"))
        .map(|r| r.score)
        .unwrap_or(0.0);

    let unboosted_policy_score = results_no_boost
        .iter()
        .find(|r| r.file_path.contains("POLICIES"))
        .map(|r| r.score)
        .unwrap_or(0.0);

    assert!(
        boosted_policy_score >= unboosted_policy_score,
        "Policy score should be boosted: {} >= {}",
        boosted_policy_score,
        unboosted_policy_score
    );
}

// ============================================================================
// Incremental Re-indexing Tests
// ============================================================================

#[test]
fn test_incremental_reindex_skips_unchanged() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    let kb_dir = ctx.temp_path().join("kb");
    std::fs::create_dir_all(&kb_dir).unwrap();

    let stable_path = kb_dir.join("stable.md");
    let mutable_path = kb_dir.join("mutable.md");

    std::fs::write(&stable_path, "# Stable\n\nThis content will not change.").unwrap();
    std::fs::write(&mutable_path, "# Mutable v1\n\nOriginal content here.").unwrap();

    ctx.db.ensure_namespace_exists("default").unwrap();

    let ingester = DiskIngester::new();

    // First ingestion — both files indexed
    let result1 = ingester
        .ingest_folder(&ctx.db, &kb_dir, "default")
        .unwrap();
    assert_eq!(result1.ingested, 2, "First run: both files ingested");
    assert_eq!(result1.skipped, 0, "First run: none skipped");

    // Second ingestion — no changes, both skipped
    let result2 = ingester
        .ingest_folder(&ctx.db, &kb_dir, "default")
        .unwrap();
    assert_eq!(result2.ingested, 0, "Second run: none ingested (unchanged)");
    assert_eq!(result2.skipped, 2, "Second run: both skipped");

    // Modify one file
    std::fs::write(&mutable_path, "# Mutable v2\n\nUpdated content with new information.").unwrap();

    // Third ingestion — only the changed file re-indexed
    let result3 = ingester
        .ingest_folder(&ctx.db, &kb_dir, "default")
        .unwrap();
    assert_eq!(result3.ingested, 1, "Third run: one file re-ingested");
    assert_eq!(result3.skipped, 1, "Third run: one file skipped");

    // Verify the re-indexed document has updated content
    let search_results =
        HybridSearch::fts_search(&ctx.db, "updated new information", 10).unwrap();
    assert!(
        !search_results.is_empty(),
        "Should find updated content after re-index"
    );
}

// ============================================================================
// Full E2E Pipeline Test
// ============================================================================

#[test]
fn test_full_pipeline_policy_enforcement() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    // Create realistic KB structure
    let kb_dir = ctx.temp_path().join("knowledge_base");
    std::fs::create_dir_all(kb_dir.join("POLICIES")).unwrap();
    std::fs::create_dir_all(kb_dir.join("PROCEDURES")).unwrap();
    std::fs::create_dir_all(kb_dir.join("REFERENCE")).unwrap();

    // Policy documents
    std::fs::write(
        kb_dir.join("POLICIES/removable_media_policy.md"),
        r#"# Removable Media Policy

## Policy Statement

All removable storage media — including USB flash drives, external hard drives,
SD cards, and optical media — are strictly prohibited on company systems.

## Scope

This policy applies to all employees, contractors, and third-party vendors.

## No Exceptions

There are no exceptions to this policy. Executive approval does not override
this security requirement. All data transfers must use approved cloud platforms.

## Enforcement

Violations are logged automatically by endpoint protection software and
reported to the Security Operations Center. Repeat violations may result
in disciplinary action up to and including termination.
"#,
    )
    .unwrap();

    std::fs::write(
        kb_dir.join("POLICIES/software_installation_policy.md"),
        r#"# Software Installation Policy

## Policy Statement

Only IT-approved software may be installed on company devices.
Users must not install personal software, browser extensions, or
development tools without prior IT approval.

## Approved Software List

See the IT portal for the current approved software catalog.
Requests for new software go through the Software Request process.
"#,
    )
    .unwrap();

    // Procedure documents
    std::fs::write(
        kb_dir.join("PROCEDURES/file_transfer_procedure.md"),
        r#"# Secure File Transfer Procedure

## Overview

Use these steps to transfer files securely without removable media.

## Steps

1. Log into OneDrive for Business
2. Upload the file to your designated folder
3. Generate a sharing link with appropriate permissions
4. Send the link to the recipient via email or Teams
5. The link expires automatically after 7 days

## Alternatives to Flash Drives

- OneDrive for Business (primary)
- SharePoint document libraries
- Microsoft Teams file sharing
- Approved SFTP server (for large transfers)
"#,
    )
    .unwrap();

    std::fs::write(
        kb_dir.join("PROCEDURES/software_request_procedure.md"),
        r#"# Software Request Procedure

## How to Request Software

1. Open the IT Self-Service Portal
2. Navigate to "Software Requests"
3. Search the approved catalog
4. Submit request with business justification
5. Manager approval required
6. IT will deploy within 2 business days
"#,
    )
    .unwrap();

    // Reference documents
    std::fs::write(
        kb_dir.join("REFERENCE/cloud_storage_options.md"),
        r#"# Cloud Storage Options Reference

## Available Platforms

| Platform | Storage | Use Case |
|----------|---------|----------|
| OneDrive | 1TB | Personal files |
| SharePoint | 25TB | Team collaboration |
| Azure Blob | Unlimited | Engineering data |

## Access

All employees have OneDrive and SharePoint access by default.
Azure Blob requires a separate access request.
"#,
    )
    .unwrap();

    ctx.db.ensure_namespace_exists("default").unwrap();

    // Ingest the entire KB
    let ingester = DiskIngester::new();
    let result = ingester
        .ingest_folder(&ctx.db, &kb_dir, "default")
        .unwrap();

    assert_eq!(result.total_files, 5, "Should find all 5 KB files");
    assert_eq!(result.ingested, 5, "Should ingest all 5 files");
    assert_eq!(result.errors, 0, "Should have no errors");

    // ---- Query 1: Policy question (should return POLICIES first) ----
    let policy_fts = HybridSearch::fts_search(&ctx.db, "USB flash drive prohibited", 10).unwrap();
    assert!(!policy_fts.is_empty(), "Policy query should return results");

    let policy_opts = SearchOptions::new(10).with_query_text("Can I use a USB flash drive?");
    let policy_results = HybridSearch::post_process_results(policy_fts, &policy_opts);

    assert!(
        policy_results[0].file_path.contains("POLICIES"),
        "Policy query should rank POLICIES first, got: {}",
        policy_results[0].file_path
    );

    // ---- Query 2: Procedure question (should return PROCEDURES first) ----
    let proc_fts = HybridSearch::fts_search(&ctx.db, "request software", 10).unwrap();
    assert!(!proc_fts.is_empty(), "Procedure query should return results");

    let proc_opts = SearchOptions::new(10).with_query_text("How do I request new software?");
    let proc_results = HybridSearch::post_process_results(proc_fts, &proc_opts);

    assert!(
        proc_results[0].file_path.contains("PROCEDURES") || proc_results[0].file_path.contains("software"),
        "Procedure query should rank relevant procedure first, got: {}",
        proc_results[0].file_path
    );

    // ---- Query 3: Reference question (should return relevant results) ----
    let ref_fts = HybridSearch::fts_search(&ctx.db, "cloud storage options", 10).unwrap();
    assert!(!ref_fts.is_empty(), "Reference query should return results");

    let has_storage_ref = ref_fts
        .iter()
        .any(|r| r.file_path.contains("cloud_storage") || r.content.contains("OneDrive"));
    assert!(
        has_storage_ref,
        "Reference query should include cloud storage document"
    );

    // Verify all ingest sources were tracked
    let source_count: i64 = ctx
        .db
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM ingest_sources WHERE source_type = 'file'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(source_count, 5, "Should have 5 disk ingest sources tracked");

    // Verify all runs completed
    let run_count: i64 = ctx
        .db
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM ingest_runs WHERE status = 'completed'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(run_count, 5, "Should have 5 completed ingest runs");
}
