# AssistSupport Knowledge Integration - Implementation Plan

**Document Version**: 2.0
**Created**: 2026-01-25
**Completion Date**: 2026-01-25
**Last Updated**: 2026-01-25
**Status**: Completed - Historical Record
**Classification**: Internal - Engineering

---

## Implementation Complete - Summary

All 10 development phases completed successfully.

### Delivered Features
- **Core Infrastructure**: SQLCipher encrypted DB, LLM engine (llama.cpp), KB indexer, OCR (Vision), Jira integration
- **Vector Search**: LanceDB hybrid search with namespace isolation
- **Decision Trees**: Guided diagnostic workflows
- **Content Ingestion**: Web pages, YouTube (yt-dlp), GitHub repos, YAML batch import
- **Knowledge Browser**: Namespace management, document inspection
- **Advanced Search**: Namespace filtering, hybrid FTS5 + vector search
- **Security**: SSRF protection, input validation, encrypted backups (Argon2id + AES-256-GCM)

### Test Coverage
- **Backend Unit Tests**: 93 tests passing
- **Backend Integration Tests**: 57 tests passing (path validation, security, KB pipeline)
- **Frontend**: 72 Vitest tests passing (component-level)
- **Total**: 222 tests, all green

### Technology Stack
- Frontend: React 19 + TypeScript + Vite
- Backend: Rust + Tauri 2.x
- Database: SQLite with SQLCipher (AES-256)
- Vector Store: LanceDB
- LLM: llama.cpp with GGUF models
- OCR: macOS Vision framework

### Post-Completion Improvements (2026-01-25)
1. **Auto-unlock Storage**: Replaced macOS Keychain with file-based credential storage to eliminate password prompts
2. **Path Validation**: Restricted KB folders and repository paths to home directory only, blocking sensitive subdirectories
3. **Comprehensive Tests**: Added 57 integration tests covering KB pipeline, path validation, and security

---

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Problem Statement](#2-problem-statement)
3. [Solution Overview](#3-solution-overview)
4. [Current System Analysis](#4-current-system-analysis)
5. [Technical Design](#5-technical-design)
6. [Implementation Phases](#6-implementation-phases)
7. [Database Schema Changes](#7-database-schema-changes)
8. [API Specifications](#8-api-specifications)
9. [UI/UX Specifications](#9-uiux-specifications)
10. [Testing Strategy](#10-testing-strategy)
11. [Risk Assessment](#11-risk-assessment)
12. [Security & Compliance Requirements](#12-security--compliance-requirements) <!-- CODEX REVISION -->
13. [Rollback Plan](#13-rollback-plan) <!-- CODEX REVISION -->
14. [Dependencies](#14-dependencies) <!-- CODEX REVISION -->
15. [Success Metrics & Confidence Gates](#15-success-metrics--confidence-gates) <!-- CODEX REVISION -->
16. [Appendix](#16-appendix) <!-- CODEX REVISION -->

---

## 1. Executive Summary

### 1.1 Purpose

This document specifies the integration of advanced knowledge management capabilities into the AssistSupport desktop application. The integration consolidates features from two companion projects—Knowledge Activation System (KAS) and Knowledge Seeder—into a single, self-contained desktop application.

### 1.2 Business Justification

**Problem**: IT support professionals need access to diverse knowledge sources (vendor documentation, internal runbooks, resolved ticket history) during support interactions. Currently, this requires manual searching across multiple systems.

**Solution**: Integrate multi-source content ingestion, namespace-based organization, and enhanced search capabilities directly into AssistSupport, enabling:
- One-click ingestion from web pages, YouTube, and GitHub
- Logical separation of knowledge domains (IT support, coding, finance)
- Batch import from curated source definitions
- Unified hybrid search across all indexed content

**Constraints**:
- Core app must run fully offline; network access is only permitted for explicit, user-initiated ingestion and must fail gracefully when offline. <!-- CODEX REVISION -->
- No Docker on target work machines
- No external services or HTTP calls between applications (single-process architecture) <!-- CODEX REVISION -->
- No Docker usage in build/test/deploy or runtime; all dependencies must be local and installable without containers. <!-- CODEX REVISION -->
- Must deploy as vanilla application with no pre-existing data
- All processing must occur locally

### 1.3 Scope

**In Scope**:
- Namespace support for content organization
- Web page content ingestion
- YouTube transcript ingestion
- GitHub repository documentation ingestion
- Batch import from YAML source definitions
- Knowledge browser UI for content management
- Enhanced search with namespace filtering

**Out of Scope**:
- Cloud synchronization
- Multi-user/multi-tenant features
- Real-time collaboration
- External API integrations (beyond content fetching)

---

## 2. Problem Statement

### 2.1 Current State

AssistSupport is a feature-complete desktop application for generating IT support responses using local LLM inference. It includes:

**Recently Completed (P0-P2)**:
- Security hardening (Jira URL/ticket validation, 10MB OCR size cap, CSP)
- Encrypted backups (Argon2id + AES-256-GCM)
- Download UX (cancel button, progress display)
- Custom GGUF model support with context window budget enforcement

**Current Knowledge Base System**:
- Indexes local files (Markdown, PDF, DOCX, XLSX, code)
- Provides hybrid search (FTS5 + vector similarity)
- Stores all content in a single namespace

### 2.2 Limitations

1. **Single Source Type**: Only local files can be indexed. Web-based documentation (Microsoft Learn, Apple Support, vendor portals) requires manual downloading and conversion.

2. **No Namespace Support**: All content exists in a single flat namespace. A user working on IT support issues sees coding documentation mixed with troubleshooting guides.

3. **Manual Curation**: Each document must be manually placed in the KB folder. There's no batch import or automated source management.

4. **No Source Tracking**: Once indexed, content has no metadata about its origin. Re-indexing or updating sources requires manual tracking.

### 2.3 Target State

A single desktop application that:
1. Ingests content from multiple source types (files, URLs, YouTube, GitHub)
2. Organizes content into logical namespaces
3. Supports batch import from declarative source definitions
4. Provides a UI for browsing and managing indexed content
5. Filters search results by namespace

---

## 3. Solution Overview

### 3.1 Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              ASSISTSUPPORT                                   │
│                         (Unified Desktop Application)                        │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                          REACT FRONTEND                                 │ │
│  │                                                                         │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐  │ │
│  │  │   Response   │ │   Sources    │ │   Content    │ │  Knowledge   │  │ │
│  │  │  Generator   │ │   Panel      │ │   Ingestion  │ │   Browser    │  │ │
│  │  │  (existing)  │ │  (existing)  │ │    (NEW)     │ │    (NEW)     │  │ │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘  │ │
│  │                                                                         │ │
│  └─────────────────────────────────┬───────────────────────────────────────┘ │
│                                    │ Tauri IPC                               │
│  ┌─────────────────────────────────▼───────────────────────────────────────┐ │
│  │                           RUST BACKEND                                   │ │
│  │                                                                          │ │
│  │  ┌─────────────────────────────────────────────────────────────────────┐│ │
│  │  │                      INGESTION LAYER (NEW)                          ││ │
│  │  │                                                                      ││ │
│  │  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐           ││ │
│  │  │  │   Web     │ │  YouTube  │ │  GitHub   │ │   Batch   │           ││ │
│  │  │  │ Ingester  │ │ Ingester  │ │ Ingester  │ │ Processor │           ││ │
│  │  │  └─────┬─────┘ └─────┬─────┘ └─────┬─────┘ └─────┬─────┘           ││ │
│  │  │        │             │             │             │                  ││ │
│  │  │        └─────────────┴──────┬──────┴─────────────┘                  ││ │
│  │  │                             ▼                                       ││ │
│  │  │                    ┌─────────────────┐                              ││ │
│  │  │                    │ Content Pipeline│                              ││ │
│  │  │                    │ (chunk + embed) │                              ││ │
│  │  │                    └────────┬────────┘                              ││ │
│  │  └─────────────────────────────┼───────────────────────────────────────┘│ │
│  │                                ▼                                         │ │
│  │  ┌─────────────────────────────────────────────────────────────────────┐│ │
│  │  │                   KNOWLEDGE LAYER (ENHANCED)                        ││ │
│  │  │                                                                      ││ │
│  │  │  ┌───────────────┐    ┌───────────────┐    ┌───────────────┐       ││ │
│  │  │  │  Namespace    │    │    Search     │    │    Index      │       ││ │
│  │  │  │   Manager     │    │   (enhanced)  │    │   Manager     │       ││ │
│  │  │  └───────────────┘    └───────────────┘    └───────────────┘       ││ │
│  │  │                                                                      ││ │
│  │  └─────────────────────────────────────────────────────────────────────┘│ │
│  │                                                                          │ │
│  │  ┌─────────────────────────────────────────────────────────────────────┐│ │
│  │  │                   EXISTING MODULES (UNCHANGED)                      ││ │
│  │  │  LLM Engine │ Prompt Builder │ Security │ Backup │ Downloads       ││ │
│  │  └─────────────────────────────────────────────────────────────────────┘│ │
│  └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                               │
│  ┌──────────────────────────────────────────────────────────────────────────┐│
│  │                           LOCAL STORAGE                                  ││
│  │  ~/Library/Application Support/AssistSupport/                            ││
│  │  ├── assistsupport.db      ← SQLCipher encrypted database               ││
│  │  ├── vectors/              ← LanceDB vector embeddings                  ││
│  │  └── models/               ← GGUF model files                           ││
│  └──────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
```

All ingestion and indexing runs inside the same Tauri process and never spawns a local server. Network access occurs only when the user initiates ingest actions, preserving offline-first behavior. <!-- CODEX REVISION -->

### 3.2 Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **SQLite over PostgreSQL** | No Docker requirement; existing encrypted database; adequate for single-user desktop application |
| **yt-dlp for YouTube** | Battle-tested transcript extraction; widely available via Homebrew; subprocess isolation |
| **Namespace as column (not schema)** | Simpler migration path; single database file; sufficient isolation for single user |
| **YAML for batch sources** | Human-readable; version-controllable; precedent from Knowledge Seeder |
| **Explicit network policy** | Network calls only occur during user-initiated ingestion; no background sync; offline-first operation stays intact. <!-- CODEX REVISION --> |
| **Source provenance metadata** | Persist `source_uri`, hashes, and ingest run metadata for auditability and re-ingestion without duplication. <!-- CODEX REVISION --> |
| **Namespace-aware filtering in search** | Apply namespace filters across FTS + vector paths, with oversampling where vector metadata filtering is limited. <!-- CODEX REVISION --> |
| **Sanitized rendering** | Render ingested content with HTML disabled/sanitized to prevent XSS in the UI. <!-- CODEX REVISION --> |
| **Local repo option for GitHub** | Allow local repo ingestion to satisfy offline requirement and avoid API rate limits. <!-- CODEX REVISION --> |
| **Progressive enhancement** | All new features are additive; existing functionality unchanged |

---

## 4. Current System Analysis

### 4.1 Existing Modules

| Module | Location | Responsibility | Changes Required |
|--------|----------|----------------|------------------|
| `db.rs` | `src-tauri/src/db.rs` | Database initialization, migrations, queries | Add namespace schema, source tracking |
| `commands.rs` | `src-tauri/src/commands.rs` | Tauri command handlers | Add ingestion commands |
| `kb/indexer.rs` | `src-tauri/src/kb/indexer.rs` | File indexing pipeline | Minor: accept namespace parameter |
| `kb/search.rs` | `src-tauri/src/kb/search.rs` | Hybrid search (FTS5 + vector) | Add namespace filtering |
| `kb/embeddings.rs` | `src-tauri/src/kb/embeddings.rs` | Vector embedding generation | No changes |
| `kb/vectors.rs` | `src-tauri/src/kb/vectors.rs` | LanceDB operations | Add namespace metadata or oversampling filter path <!-- CODEX REVISION --> |

### 4.2 Current Database Schema (v3) <!-- CODEX REVISION -->

```sql
-- Knowledge Base Documents
CREATE TABLE kb_documents (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL UNIQUE,
    file_hash TEXT NOT NULL,
    title TEXT,
    indexed_at TEXT,
    chunk_count INTEGER,
    ocr_quality TEXT,
    partial_index INTEGER DEFAULT 0
);
CREATE INDEX idx_kb_docs_path ON kb_documents(file_path);

-- Document Chunks (keep rowid for FTS5 joins)
CREATE TABLE kb_chunks (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES kb_documents(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    heading_path TEXT,
    content TEXT NOT NULL,
    word_count INTEGER
);
CREATE INDEX idx_kb_chunks_doc ON kb_chunks(document_id);

-- FTS5 Full-Text Search Index
CREATE VIRTUAL TABLE kb_fts USING fts5(
    content, heading_path,
    content='kb_chunks',
    tokenize='porter unicode61'
);

-- FTS5 Triggers (sync with kb_chunks via rowid)
CREATE TRIGGER kb_chunks_ai AFTER INSERT ON kb_chunks BEGIN
    INSERT INTO kb_fts(rowid, content, heading_path)
    VALUES (new.rowid, new.content, new.heading_path);
END;
CREATE TRIGGER kb_chunks_ad AFTER DELETE ON kb_chunks BEGIN
    INSERT INTO kb_fts(kb_fts, rowid, content, heading_path)
    VALUES ('delete', old.rowid, old.content, old.heading_path);
END;
CREATE TRIGGER kb_chunks_au AFTER UPDATE ON kb_chunks BEGIN
    INSERT INTO kb_fts(kb_fts, rowid, content, heading_path)
    VALUES ('delete', old.rowid, old.content, old.heading_path);
    INSERT INTO kb_fts(rowid, content, heading_path)
    VALUES (new.rowid, new.content, new.heading_path);
END;
```

---

## 5. Technical Design

### 5.1 New Rust Modules

#### Module Structure

```
src-tauri/src/
├── ingest/              # NEW: Content ingestion
│   ├── mod.rs           # Module exports, shared types
│   ├── types.rs         # IngestResult/IngestError/Context <!-- CODEX REVISION -->
│   ├── http.rs          # Shared HTTP client + limits <!-- CODEX REVISION -->
│   ├── web.rs           # URL → HTML → Markdown → Chunks
│   ├── youtube.rs       # YouTube → Transcript → Chunks
│   ├── github.rs        # GitHub repo → Docs → Chunks
│   ├── batch.rs         # YAML source file processor
│   └── pipeline.rs      # Content normalization, chunking
├── sources/             # NEW: Source definitions
│   ├── mod.rs           # Module exports
│   └── parser.rs        # YAML source file parser
```

### 5.2 Web Ingestion Design

**Dependencies** (Cargo.toml):
```toml
reqwest = { version = "0.12", features = ["stream"] } <!-- CODEX REVISION -->
scraper = "0.18"          # HTML parsing
html2md = "0.2"           # HTML → Markdown
url = "2"                 # URL parsing
```

**Algorithm**:
1. Validate URL with existing `validate_url`, enforce `http/https`, and block non-user-initiated fetches (default `allow_private=false`). <!-- CODEX REVISION -->
2. Normalize URL (lowercase host, strip fragments, remove default ports) for dedupe and visited tracking. <!-- CODEX REVISION -->
3. Resolve host to IP for each request/redirect; block private/loopback/link-local by default, allow only with explicit `allow_private` or `allowed_hosts`. <!-- CODEX REVISION -->
4. Fetch with shared `reqwest::Client` (30s timeout, max redirects=5, 10MB max body, streaming read). <!-- CODEX REVISION -->
5. Enforce `Content-Type: text/html` (otherwise return a clear error recommending local file ingestion). <!-- CODEX REVISION -->
6. If response is 401/403 or a login/SSO page is detected, return a clear error directing users to file-based ingestion. <!-- CODEX REVISION -->
7. Parse HTML, strip script/style/nav/footer, prefer `<article>`/`<main>` or highest text-density node. <!-- CODEX REVISION -->
8. Convert to Markdown; sanitize/escape any residual HTML before rendering in UI. <!-- CODEX REVISION -->
9. Extract canonical URL, title, and compute content hash for dedupe/refresh. <!-- CODEX REVISION -->
10. Chunk via existing pipeline, store with namespace + source metadata. <!-- CODEX REVISION -->
11. If `depth > 0`, crawl same-origin links only, default `max_pages=50` and `max_depth=2`, with visited-set to prevent loops. <!-- CODEX REVISION -->
12. Use ETag/Last-Modified to skip unchanged pages on refresh. <!-- CODEX REVISION -->

### 5.3 YouTube Ingestion Design

**Dependency**: `yt-dlp` (Homebrew)

**Algorithm**:
1. Extract video ID from URL, normalize to canonical `https://www.youtube.com/watch?v=...`. <!-- CODEX REVISION -->
2. Verify `yt-dlp` is available (configurable path); fail fast with install instructions. <!-- CODEX REVISION -->
3. Run `yt-dlp` via `Command` (no shell), with timeout + kill on hang; use `--skip-download --write-sub --write-auto-sub --sub-format vtt --no-playlist`. <!-- CODEX REVISION -->
4. Prefer human captions, fall back to auto-captions; allow optional `language` parameter (default `en`). <!-- CODEX REVISION -->
5. Parse VTT, normalize whitespace, remove timestamps and cues. <!-- CODEX REVISION -->
6. Enforce transcript size cap (e.g., 2MB) and fail gracefully if exceeded. <!-- CODEX REVISION -->
7. Fetch metadata (title, channel, duration) from `yt-dlp --print` output. <!-- CODEX REVISION -->
8. Chunk and store with namespace + source metadata. <!-- CODEX REVISION -->

### 5.4 GitHub Ingestion Design

**Algorithm**:
1. Accept GitHub URL **or** local repo path (offline-friendly). <!-- CODEX REVISION -->
2. For local repos, validate path is inside allowed roots and is a git repo; skip `.git/` contents. <!-- CODEX REVISION -->
3. For remote repos, prefer raw content fetches (or token stored via existing secure storage) and respect rate limits with backoff. <!-- CODEX REVISION -->
4. Walk README*, `docs/`, and text docs (`.md`, `.mdx`, `.rst`, `.txt`, `.adoc`) only; skip binaries and vendor dirs. <!-- CODEX REVISION -->
5. Enforce per-file size cap (e.g., 2MB), `max_files`, and `max_total_bytes` limits to prevent huge repos. <!-- CODEX REVISION -->
6. Use stable `github://owner/repo/path` identifiers for `file_path`. <!-- CODEX REVISION -->
7. Support private repos via a Settings GitHub token (stored via existing secure storage); return clear errors when token is missing/invalid. <!-- CODEX REVISION -->

### 5.5 Batch Processing Design

**YAML Format**:
```yaml
namespace: it-support
sources:
  - name: microsoft-365
    type: url
    uri: https://learn.microsoft.com/...
    depth: 2
    max_pages: 50 <!-- CODEX REVISION -->
    max_total_bytes: 20000000 <!-- CODEX REVISION -->
    allow_private: false <!-- CODEX REVISION -->
    allowed_hosts: [] <!-- CODEX REVISION -->
    enabled: true
```

**Algorithm**:
1. Parse YAML, validate schema, and reject paths outside allowed roots. <!-- CODEX REVISION -->
2. For each enabled source, dispatch to appropriate ingester with `max_concurrent` cap. <!-- CODEX REVISION -->
3. Emit structured progress events (started, page/file count, completed, failed). <!-- CODEX REVISION -->
4. Track status and per-run metadata in database. <!-- CODEX REVISION -->
5. Return per-source results; continue on partial failures. <!-- CODEX REVISION -->
6. Enforce network policy settings (private IP block unless explicitly allowed). <!-- CODEX REVISION -->

### 5.6 Ingestion Pipeline & Error Model <!-- CODEX REVISION -->

- Reuse existing chunking/embedding pipeline to maintain consistent chunk sizes and hashing. <!-- CODEX REVISION -->
- Standardize `IngestResult` and `BatchResult` with counts, warnings, and typed error codes (timeout, not_found, invalid_input, dependency_missing). <!-- CODEX REVISION -->
- Deduplicate by `file_hash` + `namespace` before inserting; re-index only when content hash changes. <!-- CODEX REVISION -->
- Update KB queries to include namespace in `file_path` lookups to avoid collisions. <!-- CODEX REVISION -->
- Persist `source_type` + `source_uri` for every document to enable UI filtering and refresh. <!-- CODEX REVISION -->
- Record `ingest_run_id` for every ingestion to support audits and rollback. <!-- CODEX REVISION -->
- Ensure error messages/logs avoid content payloads or full transcripts. <!-- CODEX REVISION -->
- Use scheme-based identifiers (`url://`, `youtube://`, `github://`) for non-file `file_path` values to prevent collisions. <!-- CODEX REVISION -->
- Wrap per-document ingestion in a transaction; on failure, roll back chunks/vectors and mark partial_index if needed. <!-- CODEX REVISION -->
- If vector store is disabled/unavailable, continue with FTS-only indexing and return a warning. <!-- CODEX REVISION -->

### 5.7 Namespace & Source Management <!-- CODEX REVISION -->

- Create `default` namespace at migration; auto-create namespaces on first use. <!-- CODEX REVISION -->
- Namespaces can be created/renamed/deleted; rename updates documents, chunks, vector metadata, and ingest_sources. <!-- CODEX REVISION -->
- Each document records `source_id` (nullable for local files) to allow refresh/delete by source. <!-- CODEX REVISION -->
- Deleting a namespace or source cascades to chunks and deletes corresponding vectors. <!-- CODEX REVISION -->

### 5.8 Offline/Network Policy & Dependency Checks <!-- CODEX REVISION -->

- All network access is user-initiated (ingest actions only); no background sync jobs. <!-- CODEX REVISION -->
- Offline or blocked network returns a clear, actionable error without retries. <!-- CODEX REVISION -->
- `yt-dlp` presence/version is validated at runtime; UI surfaces dependency status. <!-- CODEX REVISION -->
- Private/loopback/link-local IPs blocked by default; allowlist requires explicit user action. <!-- CODEX REVISION -->
- `allowed_hosts` matches exact domains or subdomains; `allow_private` must be explicit and per-source. <!-- CODEX REVISION -->
- Respect system proxy settings for outbound requests. <!-- CODEX REVISION -->

### 5.9 Security & Privacy Considerations <!-- CODEX REVISION -->

- Enforce `http/https` URLs and block private IP ranges by default; allow only via explicit allowlist/toggle. <!-- CODEX REVISION -->
- Sanitize ingested content and render Markdown with HTML disabled to prevent XSS. <!-- CODEX REVISION -->
- Cap memory usage with strict size limits; never store raw HTML beyond the ingest pipeline. <!-- CODEX REVISION -->
- Avoid logging full URLs or transcript content in error logs. <!-- CODEX REVISION -->
- Do not persist cookies or auth headers for web ingestion. <!-- CODEX REVISION -->

### 5.10 Performance & Resource Controls <!-- CODEX REVISION -->

- Ingestion runs on background tasks; UI remains responsive with cancellable jobs. <!-- CODEX REVISION -->
- Default concurrency limits: `max_concurrent=2` for network sources; embeddings processed in a bounded queue. <!-- CODEX REVISION -->
- Stream downloads and parse incrementally to avoid large in-memory buffers. <!-- CODEX REVISION -->
- Enforce per-document and per-source size caps (`max_pages`, `max_files`, `max_total_bytes`). <!-- CODEX REVISION -->
- Log only summary metrics (counts/timings), not content. <!-- CODEX REVISION -->

---

## 6. Implementation Phases

### Phase 1: Data Model & Migration (1.5 days) <!-- CODEX REVISION -->

**Objectives**:
- Add namespace + source metadata columns to existing tables
- Create namespace metadata + ingestion tracking tables
- Rebuild `kb_documents` to replace single-column UNIQUE with `(namespace, file_path)` <!-- CODEX REVISION -->
- Migrate existing data to `default` namespace and rebuild FTS triggers if needed <!-- CODEX REVISION -->
- Define vector-store migration strategy (add metadata columns or oversampling filter) <!-- CODEX REVISION -->

**Deliverables**:
- Schema migration v4 in `db.rs`
- Vector store migration notes + fallback reindex path <!-- CODEX REVISION -->
- Migration tests

**Acceptance**:
- [x] Existing databases upgrade successfully
- [x] All existing content in 'default' namespace
- [x] Namespace-aware uniqueness enforced without data loss <!-- CODEX REVISION -->
- [x] Vector search still works after migration (or reindex path documented) <!-- CODEX REVISION -->
- [x] All existing tests pass

---

### Phase 2: Web Page Ingestion (2 days) <!-- CODEX REVISION -->

**Objectives**:
- URL content fetching with timeout and size limits
- HTML → Markdown conversion
- Main content extraction
- Crawl limits (`max_pages`, same-origin), canonical URL handling, and ETag/Last-Modified refresh logic <!-- CODEX REVISION -->

**Deliverables**:
- `src-tauri/src/ingest/web.rs`
- `ingest_url` Tauri command
- Unit tests

**Acceptance**:
- [x] Can ingest Microsoft Learn, Apple Support
- [x] Correctly extracts main content
- [x] Respects depth parameter
- [x] Enforces same-origin and page-count limits <!-- CODEX REVISION -->
- [x] Blocks private IP URLs by default; allowlist/override works when enabled <!-- CODEX REVISION -->
- [x] Handles network errors gracefully

---

### Phase 3: YouTube Ingestion (1 day) <!-- CODEX REVISION -->

**Objectives**:
- yt-dlp integration
- VTT transcript parsing
- Metadata extraction
- Dependency checks, timeouts, and language selection <!-- CODEX REVISION -->

**Deliverables**:
- `src-tauri/src/ingest/youtube.rs`
- `ingest_youtube` Tauri command
- Unit tests

**Acceptance**:
- [x] Extracts transcript from captioned videos
- [x] Clear error when yt-dlp not installed
- [x] Stores video metadata
- [x] Times out gracefully for stalled yt-dlp runs <!-- CODEX REVISION -->
- [x] Clear error when transcripts are unavailable (no auto-captions) <!-- CODEX REVISION -->

---

### Phase 4: GitHub Ingestion (1.5 days) <!-- CODEX REVISION -->

**Objectives**:
- GitHub content retrieval (raw endpoints or API) <!-- CODEX REVISION -->
- README and docs processing
- Rate limit handling
- Local repo ingestion option and file-type allowlist/size limits <!-- CODEX REVISION -->

**Deliverables**:
- `src-tauri/src/ingest/github.rs`
- `ingest_github` Tauri command
- Unit tests

**Acceptance**:
- [x] Fetches README from public repos
- [x] Handles rate limiting gracefully
- [x] Skips binary/oversized files and large repos safely <!-- CODEX REVISION -->
- [x] Local repo ingestion works offline <!-- CODEX REVISION -->
- [x] Private repos ingest with token; missing token returns clear error <!-- CODEX REVISION -->
- [x] Local repo paths outside allowed roots are rejected <!-- CODEX REVISION -->

---

### Phase 5: Batch Processing (1 day) <!-- CODEX REVISION -->

**Objectives**:
- YAML source file parsing
- Multi-source orchestration
- Progress events
- Concurrency limits and per-source run tracking <!-- CODEX REVISION -->

**Deliverables**:
- `src-tauri/src/sources/parser.rs`
- `src-tauri/src/ingest/batch.rs`
- `process_source_file` Tauri command
- Example YAML files (documentation only; not auto-loaded) <!-- CODEX REVISION -->

**Acceptance**:
- [x] Parses valid YAML
- [x] Emits progress events
- [x] Partial failures don't abort batch
- [x] Concurrency limit enforced <!-- CODEX REVISION -->

---

### Phase 6: UI - Content Ingestion Panel (2 days) <!-- CODEX REVISION -->

**Objectives**:
- New "Ingest" tab
- Single-source ingestion UI
- Batch import UI
- Progress display
- Dependency status (yt-dlp) and network/offline messaging <!-- CODEX REVISION -->
- Private URL allowlist/toggle with explicit warning text <!-- CODEX REVISION -->
- Privacy notice about local storage and sensitive data <!-- CODEX REVISION -->
- GitHub token field in Settings for private repo access (secure storage, no logging) <!-- CODEX REVISION -->

**Deliverables**:
- `src/components/Ingest/IngestPanel.tsx`
- `src/components/Ingest/UrlIngest.tsx`
- `src/components/Ingest/YouTubeIngest.tsx`
- `src/components/Ingest/GitHubIngest.tsx`
- `src/components/Ingest/BatchIngest.tsx`
- `src/hooks/useIngest.ts`

**Acceptance**:
- [x] All ingestion types work from UI
- [x] Progress displays correctly
- [x] Errors shown clearly
- [x] Dependency/offline status visible for ingest actions <!-- CODEX REVISION -->
- [x] Private URL allowlist is explicit and off by default <!-- CODEX REVISION -->
- [x] Privacy notice is visible in ingest UI <!-- CODEX REVISION -->
- [x] GitHub token field stores securely and is never logged <!-- CODEX REVISION -->

---

### Phase 7: UI - Knowledge Browser (1 day) <!-- CODEX REVISION -->

**Objectives**:
- Browse by namespace
- View documents and chunks
- Delete functionality
- Namespace create/rename/delete and source refresh/disable controls <!-- CODEX REVISION -->
- Provide a clear "Delete all knowledge data" action or verify existing equivalent in Settings. <!-- CODEX REVISION -->

**Deliverables**:
- `src/components/Knowledge/KnowledgeBrowser.tsx`
- `src/hooks/useKnowledge.ts`
- Backend `clear_knowledge_data` command and UI wiring <!-- CODEX REVISION -->

**Acceptance**:
- [x] Lists namespaces with counts
- [x] Document/chunk browsing works
- [x] Delete with confirmation
- [x] Namespace create/rename/delete works with cascade behavior <!-- CODEX REVISION -->
- [x] Clear-all knowledge data flow is available and confirmed <!-- CODEX REVISION -->

---

### Phase 8: Search Enhancement (1 day) <!-- CODEX REVISION -->

**Objectives**:
- Namespace filter in search
- Source type in results
- UI namespace selector
- Apply namespace filter to both FTS and vector search paths (with oversampling fallback) <!-- CODEX REVISION -->
- Provide FTS-only fallback when vector store disabled, with UI indicator <!-- CODEX REVISION -->

**Deliverables**:
- Updated `kb/search.rs`
- Updated types
- UI changes

**Acceptance**:
- [x] Search filters by namespace
- [x] Results show source type
- [x] Performance unchanged
- [x] Vector results respect namespace filter (no cross-namespace leakage) <!-- CODEX REVISION -->
- [x] FTS-only search works when vectors are disabled <!-- CODEX REVISION -->

---

### Phase 9: Testing & Polish (2 days) <!-- CODEX REVISION -->

**Objectives**:
- Integration tests
- Performance testing
- Documentation

**Acceptance**:
- [x] All tests pass (84 Rust + 72 frontend)
- [x] Search < 200ms
- [x] Offline mode has no background network calls and clear ingest errors <!-- CODEX REVISION -->
- [x] Security/compliance tests pass (SSRF, XSS, logging hygiene, delete cascade) <!-- CODEX REVISION -->
- [x] Ingest cancel stops work promptly and UI remains responsive <!-- CODEX REVISION -->
- [x] README updated

---

### Phase 10: Portability (1 day) <!-- CODEX REVISION -->

**Objectives**:
- Verify vanilla deployment
- .gitignore audit
- Deployment documentation
- Document optional dependencies (yt-dlp) and offline behavior <!-- CODEX REVISION -->

**Acceptance**:
- [x] Clean clone builds
- [x] Fresh install works
- [x] No secrets in repo
- [x] No Docker references or dependencies introduced <!-- CODEX REVISION -->

---

## 7. Database Schema Changes

### Schema Version 4 Migration <!-- CODEX REVISION -->

```sql
-- Namespace metadata
CREATE TABLE namespaces (
    name TEXT PRIMARY KEY,
    description TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
INSERT OR IGNORE INTO namespaces (name, description) VALUES ('default', 'Default namespace');

-- Ingestion sources + run tracking
CREATE TABLE ingest_sources (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    source_type TEXT NOT NULL,
    source_uri TEXT NOT NULL,
    namespace TEXT NOT NULL DEFAULT 'default',
    config_json TEXT,
    enabled INTEGER DEFAULT 1,
    last_indexed TEXT,
    etag TEXT,
    last_modified TEXT,
    status TEXT,
    error_message TEXT,
    docs_created INTEGER DEFAULT 0,
    chunks_created INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(namespace, source_type, source_uri)
);
CREATE INDEX idx_ingest_sources_namespace ON ingest_sources(namespace); <!-- CODEX REVISION -->

CREATE TABLE ingest_runs (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES ingest_sources(id) ON DELETE CASCADE,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    status TEXT NOT NULL,
    docs_created INTEGER DEFAULT 0,
    chunks_created INTEGER DEFAULT 0,
    error_message TEXT
);
CREATE INDEX idx_ingest_runs_source_id ON ingest_runs(source_id); <!-- CODEX REVISION -->

-- Rebuild kb_documents to support namespace-aware uniqueness
CREATE TABLE kb_documents_new (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL,
    file_hash TEXT NOT NULL,
    title TEXT,
    indexed_at TEXT,
    chunk_count INTEGER,
    ocr_quality TEXT,
    partial_index INTEGER DEFAULT 0,
    namespace TEXT NOT NULL DEFAULT 'default',
    source_type TEXT NOT NULL DEFAULT 'file',
    source_uri TEXT,
    source_id TEXT REFERENCES ingest_sources(id) ON DELETE SET NULL,
    mime_type TEXT,
    byte_size INTEGER,
    content_etag TEXT,
    last_modified TEXT
);
CREATE UNIQUE INDEX idx_kb_docs_unique ON kb_documents_new(namespace, file_path);
CREATE INDEX idx_kb_docs_namespace ON kb_documents_new(namespace);
CREATE INDEX idx_kb_docs_source_id ON kb_documents_new(source_id); <!-- CODEX REVISION -->

INSERT INTO kb_documents_new (id, file_path, file_hash, title, indexed_at, chunk_count, ocr_quality, partial_index, namespace, source_type)
SELECT id, file_path, file_hash, title, indexed_at, chunk_count, ocr_quality, partial_index, 'default', 'file'
FROM kb_documents;

DROP TABLE kb_documents;
ALTER TABLE kb_documents_new RENAME TO kb_documents;
CREATE INDEX idx_kb_docs_path ON kb_documents(file_path);

-- Add namespace to kb_chunks
ALTER TABLE kb_chunks ADD COLUMN namespace TEXT DEFAULT 'default';
CREATE INDEX idx_kb_chunks_namespace ON kb_chunks(namespace);
UPDATE kb_chunks SET namespace = 'default' WHERE namespace IS NULL;
```

**Vector store migration (LanceDB)**: add `namespace` (and `document_id`) fields to the vector table schema; rebuild the table and re-insert embeddings (fallback: re-embed all documents). <!-- CODEX REVISION -->

---

## 8. API Specifications

### New Tauri Commands

| Command | Parameters | Returns |
|---------|------------|---------|
| `ingest_url` | url, namespace, depth?, allow_private?, allowed_hosts? | IngestResult <!-- CODEX REVISION --> |
| `ingest_youtube` | url, namespace | IngestResult |
| `ingest_github` | repo_or_path, namespace | IngestResult <!-- CODEX REVISION --> |
| `process_source_file` | path | BatchResult |
| `cancel_ingest` | ingest_run_id | () <!-- CODEX REVISION --> |
| `list_namespaces` | - | Vec<NamespaceInfo> |
| `create_namespace` | name, description? | () <!-- CODEX REVISION --> |
| `rename_namespace` | old_name, new_name | () <!-- CODEX REVISION --> |
| `delete_namespace` | name | () (cascades docs/chunks/vectors) <!-- CODEX REVISION --> |
| `list_sources` | namespace? | Vec<IngestSource> <!-- CODEX REVISION --> |
| `refresh_source` | source_id | IngestResult <!-- CODEX REVISION --> |
| `delete_source` | source_id | () <!-- CODEX REVISION --> |
| `list_documents` | namespace, source_id? | Vec<KbDocument> <!-- CODEX REVISION --> |
| `list_document_chunks` | document_id | Vec<KbChunk> <!-- CODEX REVISION --> |
| `delete_document` | document_id | () <!-- CODEX REVISION --> |
| `clear_knowledge_data` | namespace? | () <!-- CODEX REVISION --> |

### Updated Commands

| Command | Changes |
|---------|---------|
| `search_kb` | Add optional `namespace` parameter |

---

## 9. UI/UX Specifications

### Navigation

```
┌─────────────────────────────────────────────────────────────┐
│  Draft  │  Sources  │  Follow-ups  │  Ingest  │  Settings  │
└─────────────────────────────────────────────────────────────┘
                                        ▲
                                        │ NEW TAB
```

### Source Type Icons

| Type | Icon |
|------|------|
| File | 📄 |
| URL | 🌐 |
| YouTube | 🎬 |
| GitHub | 🐙 |

### Namespace & Source Management UX <!-- CODEX REVISION -->

- Namespace selector with create/rename/delete and safe-guard confirmations. <!-- CODEX REVISION -->
- Ingest panel shows dependency status (yt-dlp) and offline/network errors. <!-- CODEX REVISION -->
- Knowledge Browser surfaces source metadata (origin, last indexed, status) and refresh/disable actions. <!-- CODEX REVISION -->
- Private URL allowlist toggle includes explicit warning about internal network access. <!-- CODEX REVISION -->
- Clear-all knowledge data action is surfaced with strong confirmation. <!-- CODEX REVISION -->
- Ingest UI includes a short privacy notice about local storage and sensitive content. <!-- CODEX REVISION -->
- Search UI indicates when vector (semantic) search is disabled and FTS-only mode is used. <!-- CODEX REVISION -->

---

## 10. Testing Strategy

### Unit Tests

| Module | Priority |
|--------|----------|
| `ingest/web.rs` | High |
| `ingest/youtube.rs` | High |
| `ingest/github.rs` | Medium |
| `sources/parser.rs` | Medium |
| `db.rs` (migration) | High |
| `kb/search.rs` | High |
| `ingest/pipeline.rs` | High <!-- CODEX REVISION --> |
| `ingest/http.rs` | Medium <!-- CODEX REVISION --> |
| `kb/indexer.rs` (namespace + txn paths) | Medium <!-- CODEX REVISION --> |

### Integration Tests

| Test | Verification |
|------|--------------|
| Web → Search | Query returns ingested chunks |
| YouTube → Search | Query returns transcript |
| Namespace isolation | Filtered search works |
| Migration | Existing data preserved |
| Offline ingest | Clear error with no background network calls <!-- CODEX REVISION --> |
| Delete cascade | Deleting namespace/source removes chunks + vectors <!-- CODEX REVISION --> |
| Private GitHub | Private repo ingests succeed with token; clear error without token <!-- CODEX REVISION --> |
| Vector disabled | FTS-only indexing works when vector store is disabled <!-- CODEX REVISION --> |

### Security & Compliance Tests <!-- CODEX REVISION -->

| Test | Verification |
|------|--------------|
| SSRF block | Private/loopback/link-local URLs rejected by default <!-- CODEX REVISION --> |
| Allowlist override | Private URL succeeds only when allowlist/toggle set <!-- CODEX REVISION --> |
| XSS rendering | Ingested HTML is not executed in UI <!-- CODEX REVISION --> |
| Logging hygiene | Logs contain no URL bodies or transcript content <!-- CODEX REVISION --> |
| Data deletion | Namespace/source/document delete removes DB rows and vectors <!-- CODEX REVISION --> |

---

## 11. Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| yt-dlp breaking changes | Medium | Medium | Version pin |
| GitHub rate limiting | High | High | Rate limiter, backoff |
| Large content OOM | High | Low | Size limits |
| Migration failure | High | Low | Auto backup |
| yt-dlp not installed | Medium | High | Dependency check + clear install guidance <!-- CODEX REVISION --> |
| XSS from ingested HTML | High | Medium | Sanitize/disable HTML rendering in UI <!-- CODEX REVISION --> |
| Huge repos/crawls | High | Medium | `max_files`/`max_pages`/`max_total_bytes` caps <!-- CODEX REVISION --> |
| Vector store unencrypted | High | Medium | Keep disabled by default, require explicit consent <!-- CODEX REVISION --> |
| Network restrictions/proxy | Medium | Medium | Respect system proxy, offline errors are explicit <!-- CODEX REVISION --> |
| SSRF / private network access | High | Medium | Block private IPs by default; allowlist only with explicit user opt-in <!-- CODEX REVISION --> |
| GDPR/US privacy non-compliance | High | Low | Local-only processing, explicit deletion/export, no telemetry <!-- CODEX REVISION --> |
| JS-rendered pages not captured | Medium | Medium | Document limitation; recommend file-based ingestion for dynamic sites <!-- CODEX REVISION --> |
| GitHub token leakage | High | Low | Secure storage only; never log tokens; redact in error paths <!-- CODEX REVISION --> |
| Partial ingest leaving orphan chunks | Medium | Medium | Wrap ingest per document in transaction; delete on failure <!-- CODEX REVISION --> |
| Embeddings unavailable | Medium | Low | Fall back to FTS-only indexing with warnings <!-- CODEX REVISION --> |

---

## 12. Security & Compliance Requirements <!-- CODEX REVISION -->

- **Data locality**: All processing remains local; no telemetry or remote storage. Network use is ingestion-only and user-initiated. <!-- CODEX REVISION -->
- **No telemetry**: Do not collect or transmit analytics; logs are local only. <!-- CODEX REVISION -->
- **GDPR alignment**: Purpose limitation (support workflows only), data minimization (ingest only what is needed), storage limitation (deletion controls), integrity/confidentiality (encryption, access control), and transparency via UI labels. <!-- CODEX REVISION -->
- **US privacy alignment**: Provide clear local storage notice and user controls to delete/export data (aligns with CPRA/CCPA, CPA, VCDPA, CTDPA, UCPA principles). <!-- CODEX REVISION -->
- **PII handling**: Treat ingested content as potentially sensitive; avoid logging content or URLs; store only required metadata. <!-- CODEX REVISION -->
- **Credential handling**: Any optional tokens (GitHub) use existing secure storage and are never logged. <!-- CODEX REVISION -->
- **User notice**: Ingest UI displays a brief notice that content is stored locally and may contain sensitive data. <!-- CODEX REVISION -->
- **Encryption**: SQLCipher for DB; encrypted backups; vector store remains opt-in with explicit consent due to lack of encryption support. <!-- CODEX REVISION -->
- **Network safety**: Block private/loopback/link-local IPs by default; allowlist/override requires explicit user action and warning. <!-- CODEX REVISION -->
- **Private IP ranges**: IPv4 RFC1918, loopback, link-local; IPv6 loopback, link-local, and ULA ranges. <!-- CODEX REVISION -->
- **Content safety**: Render Markdown with HTML disabled/sanitized to prevent XSS. <!-- CODEX REVISION -->
- **Data deletion**: Namespace/source/document deletion cascades to chunks and vectors; provide “clear all data” path if not already present. <!-- CODEX REVISION -->
- **Data portability**: Existing backup/export functions provide user-controlled data export for GDPR access/portability. <!-- CODEX REVISION -->
- **Retention**: Default is manual deletion; no automatic retention policy in this phase (documented). <!-- CODEX REVISION -->
- **Transport security**: TLS verification required; no insecure TLS overrides. <!-- CODEX REVISION -->

---

## 13. Rollback Plan <!-- CODEX REVISION -->

1. **Database**: Restore from automatic pre-migration backup
2. **Vector store**: Restore `vectors/` backup or disable vector search and reindex <!-- CODEX REVISION -->
3. **Code**: `git checkout <previous-tag>`
4. **Full rebuild**: `pnpm install && pnpm tauri build`

---

## 14. Dependencies <!-- CODEX REVISION -->

### Rust (Cargo.toml)

```toml
reqwest = { version = "0.12", features = ["stream"] } <!-- CODEX REVISION -->
scraper = "0.18"
html2md = "0.2"
url = "2"
serde_yaml = "0.9" <!-- CODEX REVISION -->
```

### External

| Dependency | Purpose | Installation |
|------------|---------|--------------|
| yt-dlp | YouTube transcripts | `brew install yt-dlp` |
| git (optional) | Local repo ingestion | Preinstalled on macOS or via Xcode CLT <!-- CODEX REVISION --> |

**Explicitly not used**: Docker (no container runtime required or permitted). <!-- CODEX REVISION -->

---

## 15. Success Metrics & Confidence Gates <!-- CODEX REVISION -->

| Metric | Target |
|--------|--------|
| Web ingestion success | >95% |
| YouTube ingestion success | >90% |
| Search latency (p95) | <200ms |
| Test coverage | >80% |
| Offline behavior | No background network calls; ingest returns clear offline error <!-- CODEX REVISION --> |

### Confidence Gates <!-- CODEX REVISION -->

- **Migration proof**: Run v4 migration on a seeded DB; verify `(namespace, file_path)` uniqueness, FTS triggers, and default namespace population. <!-- CODEX REVISION -->
- **Vector isolation proof**: Rebuild LanceDB table with `namespace` metadata; add test to ensure zero cross-namespace hits. <!-- CODEX REVISION -->
- **Offline/SSRF proof**: Confirm no background network calls; private IP URLs blocked by default; allowlist works when enabled. <!-- CODEX REVISION -->
- **Ingestion reliability**: Web/YouTube/GitHub integration tests cover size limits, rate limits, and missing transcripts. <!-- CODEX REVISION -->
- **Private GitHub proof**: Private repo ingest works with token and fails cleanly without one. <!-- CODEX REVISION -->
- **Security proof**: Markdown rendering is HTML-sanitized; logs contain no content/PII; vector store opt-in is enforced. <!-- CODEX REVISION -->
- **Compliance signoff**: Security/compliance checklist reviewed and approved by engineering lead. <!-- CODEX REVISION -->

---

## 16. Appendix <!-- CODEX REVISION -->

### Migration Runbook (DB + Vectors) <!-- CODEX REVISION -->

1. Create automatic backups: `assistsupport.db` and `vectors/` directory. <!-- CODEX REVISION -->
2. Run schema migration v4; verify `namespaces` contains `default`. <!-- CODEX REVISION -->
3. Validate counts: `kb_documents`/`kb_chunks` before and after; ensure no data loss. <!-- CODEX REVISION -->
4. Rebuild LanceDB table with `namespace` + `document_id` fields; reinsert embeddings. <!-- CODEX REVISION -->
5. Run smoke tests: namespace-filtered search, delete cascade, and offline ingest error. <!-- CODEX REVISION -->
6. If any validation fails, restore backups and abort release. <!-- CODEX REVISION -->

### File Structure After Implementation

```
src-tauri/src/
├── ingest/              # NEW
│   ├── mod.rs
│   ├── types.rs         # Ingest result/error types <!-- CODEX REVISION -->
│   ├── http.rs          # Shared HTTP client + limits <!-- CODEX REVISION -->
│   ├── web.rs
│   ├── youtube.rs
│   ├── github.rs
│   └── batch.rs
├── sources/             # NEW
│   ├── mod.rs
│   └── parser.rs
├── kb/                  # ENHANCED
│   └── search.rs        # + namespace filtering
└── db.rs                # Schema v4

src/components/
├── Ingest/              # NEW
│   ├── IngestPanel.tsx
│   ├── UrlIngest.tsx
│   ├── YouTubeIngest.tsx
│   ├── GitHubIngest.tsx
│   └── BatchIngest.tsx
└── Knowledge/           # NEW
    ├── KnowledgeBrowser.tsx
    └── ...

sources/                 # NEW
├── it-support.yaml
├── coding.yaml
└── finance.yaml
                         # Sample definitions only; not auto-loaded <!-- CODEX REVISION -->
```

### Total Estimated Effort: ~14 days <!-- CODEX REVISION -->

---

**END OF DOCUMENT**
