# Week 2 Summary: KB Ingestion & End-to-End Testing

## What Was Built

### Disk Ingestion Pipeline (`src-tauri/src/kb/ingest/disk.rs`)

A new `DiskIngester` module that wraps the existing `KbIndexer` with proper ingest source/run tracking. Previously, disk-indexed articles (via `set_kb_folder` + `index_kb`) did not create `ingest_sources` or `ingest_runs` entries, meaning they were invisible in the source management UI.

The `DiskIngester` fixes this by:
- Creating `IngestSource` entries (source_type="file") for each file
- Creating `IngestRun` entries to track each ingestion operation
- Setting `namespace_id`, `source_type`, and `source_id` on documents and chunks
- Using file hash comparison (SHA-256) for incremental re-ingestion
- Skipping unchanged files automatically

### Tauri Command (`ingest_kb_from_disk`)

New Tauri command exposed to the frontend:
- Takes `folder_path` and `namespace_id`
- Validates the path is within the home directory
- Ensures the namespace exists
- Returns `DiskIngestResultResponse` with document list

### Integration Tests (`src-tauri/tests/kb_disk_ingestion.rs`)

Six integration tests covering the full pipeline:

1. **test_disk_ingest_creates_documents_and_chunks** — Verifies documents, chunks, ingest_sources, and ingest_runs are all created correctly
2. **test_policy_query_returns_policy_first** — Verifies POLICIES/ results surface first for policy queries
3. **test_procedure_query_returns_procedure_first** — Verifies PROCEDURES/ results surface first for how-to queries
4. **test_policy_boost_with_real_indexed_data** — Verifies policy boost scores are applied correctly
5. **test_incremental_reindex_skips_unchanged** — Verifies unchanged files are skipped, changed files re-indexed
6. **test_full_pipeline_policy_enforcement** — Full E2E test: 5-file KB with 3 query types

### Pilot Documentation

- `docs/TEAM_PILOT_VALIDATION.md` — 20 test queries across 4 groups
- `docs/PILOT_QUICK_START.md` — Team setup and testing instructions

## Files Changed

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/kb/ingest/disk.rs` | CREATE | Disk folder ingester with source tracking |
| `src-tauri/src/kb/ingest/mod.rs` | EDIT | Added `pub mod disk;` |
| `src-tauri/src/commands/mod.rs` | EDIT | Added `ingest_kb_from_disk` command + response type |
| `src-tauri/src/lib.rs` | EDIT | Registered new command |
| `src-tauri/tests/kb_disk_ingestion.rs` | CREATE | 6 integration tests |
| `docs/TEAM_PILOT_VALIDATION.md` | CREATE | 20 pilot test queries |
| `docs/PILOT_QUICK_START.md` | CREATE | Quick start guide |
| `docs/WEEK2_SUMMARY.md` | CREATE | This file |
| `CHANGELOG.md` | EDIT | v0.6.0 entry |

## Architecture

```
Frontend                    Backend
   │                           │
   ├─ invoke('ingest_kb_      │
   │   from_disk', {path,     │
   │   namespace})             │
   │                           ├─ validate_within_home(path)
   │                           ├─ ensure_namespace_exists()
   │                           ├─ DiskIngester::ingest_folder()
   │                           │   ├─ KbIndexer::scan_folder()
   │                           │   ├─ For each file:
   │                           │   │   ├─ file_hash() → compare
   │                           │   │   ├─ find/create IngestSource
   │                           │   │   ├─ create IngestRun
   │                           │   │   ├─ parse_document()
   │                           │   │   ├─ chunk_document()
   │                           │   │   ├─ INSERT kb_documents
   │                           │   │   ├─ INSERT kb_chunks
   │                           │   │   └─ complete IngestRun
   │                           │   └─ Return DiskIngestResult
   │                           │
   ◄─ DiskIngestResultResponse─┤
```
