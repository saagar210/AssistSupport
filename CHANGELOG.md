# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

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
