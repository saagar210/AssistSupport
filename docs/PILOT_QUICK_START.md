# Pilot Quick Start Guide

Instructions for the team to test the KB ingestion pipeline and search ranking.

## Prerequisites

- Rust toolchain installed (`rustup`)
- Clone the repository and navigate to `src-tauri/`

## Step 1: Prepare Knowledge Base

Create a KB folder with the standard structure:

```
knowledge_base/
├── POLICIES/          # Security policies, restrictions, rules
│   ├── removable_media_policy.md
│   ├── software_installation_policy.md
│   └── ...
├── PROCEDURES/        # Step-by-step how-to guides
│   ├── laptop_request.md
│   ├── file_transfer_procedure.md
│   └── ...
└── REFERENCE/         # Specs, contacts, general info
    ├── device_specs.md
    ├── cloud_storage_options.md
    └── ...
```

Each file should be a Markdown document with a `# Title` heading and clear content.

## Step 2: Run Integration Tests

```bash
cd src-tauri
cargo test kb_disk_ingestion -- --nocapture
```

This runs 6 integration tests covering:
- Document and chunk creation with source tracking
- Policy query ranking (POLICIES/ first)
- Procedure query ranking (PROCEDURES/ first)
- Policy boost score verification
- Incremental re-indexing (skip unchanged, re-index modified)
- Full E2E pipeline across all query types

## Step 3: Run Validation Queries

Use the 20 test queries from `docs/TEAM_PILOT_VALIDATION.md` and record:

1. Top search result for each query
2. Whether it matches the expected category
3. Relevance score (1-5)

## Step 4: Report Results

Record your findings:

| Query # | Top Result Category | Relevance (1-5) | Notes |
|---------|-------------------|-----------------|-------|
| 1 | POLICIES | 5 | Correct policy surfaced |
| ... | ... | ... | ... |

## Common Issues

**No search results**: Ensure the KB folder has `.md` files and the database is initialized.

**Wrong category ranking**: Policy boost only applies to policy-related queries (permission checks, restriction questions). Procedural queries should not trigger policy boost.

**Incremental re-index not detecting changes**: File hash comparison uses SHA-256. Ensure the file content actually changed (not just metadata like timestamps).

## Running All Tests

```bash
# All backend tests
cd src-tauri && cargo test

# Just the disk ingestion tests
cd src-tauri && cargo test kb_disk_ingestion

# Just the unit tests in disk.rs
cd src-tauri && cargo test disk::tests
```
