# Security Architecture

This document summarizes the security model used in AssistSupport and serves as
an implementation-level companion to `SECURITY.md`.

## Core Design Goals

- Keep customer support data local to the operator workstation by default.
- Protect sensitive state at rest with encryption and strict file permissions.
- Fail closed for network-facing paths and external ingestion.
- Maintain auditable security checks in CI and local verification workflows.

## Data Protection

- Local database encryption uses SQLCipher (`AES-256`) in the Rust backend.
- Token material is encrypted before persistence.
- New workspace setup uses macOS Keychain-backed key storage by default.
- Existing passphrase-protected workspaces are still supported through the dedicated unlock flow.
- Optional vector-search embeddings stay local, but they are not currently encrypted at rest when vector search is enabled.
- Sensitive key material is zeroized after use where practical.

## Network and Input Hardening

- Search API requires bearer auth by default and enforces request limits.
- Runtime validation blocks unsafe production configuration.
- `GET /health` is liveness-only; `GET /ready` is the dependency-backed readiness signal for operators and CI.
- SSRF and path-validation tests exist in `src-tauri/tests/`.
- Request payloads are size-limited in the search API to reduce abuse risk.

## Recovery and Diagnostics

- Startup can enter recovery mode before normal DB initialization when corruption or migration conflicts are detected.
- Recovery diagnostics run both `PRAGMA integrity_check` and `PRAGMA foreign_key_check`.
- Backup import now validates archive size bounds before full restore and can restore a fresh encrypted database from recovery mode.

## Verification and Gates

- Git hygiene checks: branch safety, atomic commits, secret scanning.
- UI gates: static checks + Playwright visual/a11y suites.
- Coverage gates: frontend coverage artifacts with diff coverage checks.
- Security checks: `pnpm audit` and `cargo audit`.

## Dependency Intake Policy

- Dependabot PRs are monitored and reviewed as intake signals.
- Merge-ready dependency updates are re-created on codex-compliant branches
  (`codex/<type>/<slug>`) to satisfy repository branch governance checks.
- Superseded Dependabot PRs are closed with links to replacement PRs for traceability.

## Reporting

Use the process in `SECURITY.md` to report vulnerabilities privately.
