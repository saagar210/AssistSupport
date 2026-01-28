# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.6.0-pilot] - 2026-01-28

### Added
- **Feedback Logger Module**: `src-tauri/src/feedback/mod.rs` — SQLite-backed query logging, feedback submission (accuracy/clarity/helpfulness 1-5), pilot stats aggregation with per-category breakdown
- **DB Schema v11**: `pilot_query_logs` and `pilot_feedback` tables with foreign keys, CHECK constraints, and indexes
- **5 Tauri Commands**: `log_pilot_query`, `submit_pilot_feedback`, `get_pilot_stats`, `get_pilot_query_logs`, `export_pilot_data`
- **CSV Export**: `feedback/export.rs` — export query logs and feedback to CSV file
- **Pilot Tab**: New app tab with query tester (search + auto-log + rate) and real-time dashboard (stats grid, per-category table, query log viewer)
- **FeedbackForm Component**: Star-rating UI for accuracy, clarity, helpfulness with optional comment
- **PilotDashboard Component**: Live stats (total queries, feedback count, accuracy %, clarity avg, helpfulness avg), category breakdown table, expandable query log
- **PilotQueryTester Component**: Search KB, view results, rate response — all in one flow
- **8 Backend Tests**: Query logging, feedback submission, validation, rating clamping, category detection, pilot stats, CSV export
- **Pilot Documentation**: Deploy checklist, feedback template, aggregation sheet, daily log, analysis template, go/no-go decision, v0.6.1 plan, Week 4 brief, pilot results template

## [0.6.0] - 2026-01-28

### Added
- **Disk Ingestion Pipeline**: New `DiskIngester` module wraps `KbIndexer` with `IngestSource`/`IngestRun` tracking so disk-indexed articles appear in the source management UI
- **`ingest_kb_from_disk` Command**: New Tauri command for ingesting a folder with path validation, namespace support, and full source tracking
- **Incremental Re-ingestion**: File hash (SHA-256) comparison skips unchanged files automatically on re-ingest
- **6 Integration Tests**: End-to-end tests for disk ingestion, search ranking, policy boost, incremental re-index, and full pipeline verification
- **Pilot Documentation**: 20 structured test queries (`TEAM_PILOT_VALIDATION.md`), quick start guide (`PILOT_QUICK_START.md`), and week 2 summary (`WEEK2_SUMMARY.md`)

### Technical Details
- `DiskIngester` creates `ingest_sources` (source_type="file") and `ingest_runs` per file
- Documents and chunks get `namespace_id`, `source_type`, and `source_id` fields set
- Follows the same source/run tracking pattern as `WebIngester`
- Path validation ensures folder is within home directory (reuses `validate_within_home`)

## [0.5.3] - 2026-01-28

### Added
- **Policy-First Search Ranking**: Search results from POLICIES/ documents are automatically boosted when the query is policy-related (permission checks, restriction queries)
- **Policy Query Detection**: New `is_policy_query()` and `policy_query_confidence()` functions detect permission/restriction questions with confidence scoring
- **Policy Enforcement in System Prompt**: Updated IT support system prompt (v5.1.0) with strict policy enforcement rules — forbidden items are denied with alternatives, no exceptions
- **Knowledge Base Structure**: Created `knowledge_base/` directory with POLICIES/ (12 files), PROCEDURES/ (8 files), and REFERENCE/ (6 files) — 26 articles total
- **38 New Tests**: Policy detection (10), confidence scoring (4), result classification (4), boost application (8), search integration (4), prompt enforcement (8)

### Changed
- `SearchOptions` now carries `query_text` for policy-aware post-processing
- `HybridSearch::post_process_results()` applies policy boost before score normalization
- System prompt template version bumped from 5.0.0 to 5.1.0
- Template name updated to `it_support_v3_policy`
- `search_kb_with_options` command now passes query text into search options and applies post-processing

### Technical Details
- Policy confidence scoring: keywords (0.5) + restricted items (0.5) + question patterns (0.2), capped at 1.0
- Policy boost: `POLICY_BOOST (0.5) * confidence` added to scores of POLICIES/ results
- Threshold for policy detection: confidence >= 0.4
- Policy boost applied before normalization to preserve ranking impact

## [0.5.0] - 2026-01-28

### Changed
- **UI Redesign**: ChatGPT-inspired visual overhaul — green accent (#10a37f), gradient buttons, truly dark backgrounds (#0d0d0d)
- **Typography**: Switched to system font stack (replaces Google Fonts) for instant rendering
- **Sidebar**: Polished 260px sidebar with green glow brand icon, rounded nav items
- **Animations**: Smooth slide-in/fade transitions with `prefers-reduced-motion` support
- **Components**: Lift-on-hover buttons with glow shadows, thinner hide-on-idle scrollbars, larger card border radii
- **Accessibility**: WCAG AA contrast ratios maintained throughout redesign

## [0.4.1] - 2026-01-28

### Added
- **Fast Startup**: Background model auto-loading with startup metrics tracking (2-3 second launches)
- **Session Tokens**: 24h auto-unlock tokens — no password friction on every launch, with Lock App button in Settings
- **Model Persistence**: Save/restore last-used models across app restarts
- **CLI Search**: Real KB indexing and hybrid search in CLI (replaces stubs)
- 6 new Tauri commands: `get_model_state`, `get_startup_metrics`, `create_session_token`, `validate_session_token`, `clear_session_token`, `lock_app`

### Fixed
- Draft panel cutoff — added `min-height: 0` to flex/grid children

## [0.4.0] - 2026-01-27

### Added
- **Analytics Dashboard**: Response quality tracking with ratings, trends, and article-level drill-down
- **Response Ratings**: 1-5 star rating system with analytics instrumentation
- **Response Alternatives**: Side-by-side comparison of multiple generated responses (AlternativePanel)
- **Template Recycling**: Save top-rated responses as reusable templates (SaveAsTemplateModal, SavedResponsesSuggestion)
- **Jira Transitions**: Post responses and transition tickets to new status in one workflow (JiraPostPanel)
- **KB Staleness Monitoring**: Staleness indicators in KbHealthPanel and KnowledgeBrowser with article review tracking
- **Article Analytics**: Per-article drill-down showing citation frequency and search hits (ArticleDetailPanel)
- **Conversation Input**: Threaded conversation-style input with context threading
- **Batch Processing**: Generate responses for multiple similar tickets (BatchPanel)
- **Auto-Suggest**: Automatic suggestions based on ticket content
- **Draft Versioning**: Version history with diff viewer for drafts
- **KB Management**: Chunk editing and health monitoring from the UI
- **Source Transparency**: KB article citations shown alongside generated responses
- DB schema v8 and v9 migrations (response_ratings, analytics_events, response_alternatives, saved_response_templates, jira_status_transitions tables)
- 27+ new Tauri commands for ratings, analytics, batch processing, KB health, alternatives, templates, and Jira transitions

### Fixed
- WCAG AA contrast issues in dark mode
- FTS5 query preprocessing for improved search relevance
- Prompt perspective framing for better response quality

## [0.3.2] - 2026-01-26

### Changed
- **Repositioning**: Refocused messaging for IT support engineers (from generic "customer support")
- **README**: Added competitive comparison (vs ChatGPT), compliance badges, "Why IT Support Engineers Use This" section
- **Compliance Report**: Added quick-link header with per-standard navigation

### Added
- `docs/IT_SUPPORT_GUIDE.md` - Complete guide with 5 workflow examples, 3 team deployment options, Jira integration guide, compliance section
- `docs/QUICKSTART_IT_SUPPORT.md` - 5-minute setup guide for IT support engineers
- `docs/ROADMAP.md` - Q1-Q3 2026 feature priorities (ServiceNow, analytics, Windows installer)
- Team setup section in `docs/INSTALLATION.md` with shared KB configuration
- `planning/` directory with testing strategy and results

### Fixed
- Frontend test setup: Added localStorage mock to `src/test/setup.ts` (resolves 18 SettingsTab test failures in jsdom)

## [0.3.1] - 2026-01-25

### Fixed

- **H1**: `.gitignore` now excludes sensitive file patterns (`.key`, `.pem`, `.env`, `tokens.json`, etc.)
- **H2**: Diagnostics filesystem health check uses correct data directory path (`dirs::data_dir()`)
- **H3**: Directory helper functions (`get_app_data_dir`, `get_logs_dir`, `get_cache_dir`) use `expect()` instead of insecure `"."` fallback
- **H4**: Database file permissions set to 0600 on creation
- **H5**: Database backup file permissions set to 0600 after copy

### Changed

- **M1**: LLM generation now validates prompt length against context window before decoding
- **M2**: Jira `get_ticket()` and `add_comment()` validate ticket key format before URL interpolation
- **M3**: `validate_url_for_ssrf` marked `#[deprecated]` in favor of `validate_url_for_ssrf_with_pinning`
- **M4**: SQLCipher key pragma and hex key strings are zeroized after use; `to_hex()` doc warns callers
- **M5**: Security docs updated with 5 missing audit event types and custom event row
- **M6**: Added this CHANGELOG

### Improved

- **L1**: Keychain secret bytes zeroized after copying to key buffer
- **L2**: Legacy key file hex string and decoded bytes zeroized after use
- **L3**: AWS IMDSv2 IPv6 metadata endpoint (`fd00:ec2::254`) blocked in SSRF protection
- **L4/L12**: Vector store namespace filters use `sanitize_id()` instead of `sanitize_filter_value()`
- **L5**: Audit `log_audit_best_effort` uses bounded channel + single writer thread instead of unbounded `thread::spawn`
- **L6**: Audit log rotation errors logged to stderr instead of silently ignored
- **L7**: Job type parse failures return `Custom("unknown")` instead of panicking
- **L8**: File parsers (markdown, plaintext, code) use `from_utf8_lossy` for non-UTF-8 tolerance
- **L9**: Memory monitoring uses macOS `task_info` mach API instead of hardcoded 0
- **L10**: Removed dead commented-out invoke call in OnboardingWizard
- **L11**: Covered by H3 (`expect()` replaces insecure fallback)

## [0.3.0] - 2026-01-25

### Added

- LLM-driven first-response drafting with streaming generation
- Troubleshooting checklists and decision trees
- Audit UI for viewing security events
- Model integrity enforcement with SHA256 verification and allowlist
- Security hardening: SQLCipher encryption, dual key storage (Keychain/Passphrase)
- SSRF protection with DNS pinning
- Path traversal protection with home directory restriction
- Comprehensive test suite (366 backend tests, 72 frontend tests)
- Health diagnostics and self-repair commands
- Web, YouTube, and GitHub content ingestion
- Hybrid search (FTS5 + vector)
- Multi-format document indexing (Markdown, PDF, DOCX, XLSX, code, images)
