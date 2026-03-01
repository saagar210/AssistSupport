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
- Key storage supports macOS Keychain and passphrase-derived key wrapping.
- Sensitive key material is zeroized after use where practical.

## Network and Input Hardening

- Search API requires bearer auth by default and enforces request limits.
- Runtime validation blocks unsafe production configuration.
- SSRF and path-validation tests exist in `src-tauri/tests/`.
- Request payloads are size-limited in the search API to reduce abuse risk.

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
