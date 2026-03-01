# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Fixes

- Reduced Rust advisory surface by pruning X11-specific Tauri dependency paths (`gdkx11*` advisories no longer active).
- Replaced anonymous Rust audit waivers with issue-backed ownership and mitigation tracking in `scripts/security/run-cargo-audit.sh`.
- Added explicit Rust advisory mapping artifact with dependency-chain evidence and mitigation routing (`docs/reports/phase3-rust-advisory-map.md`).
- Reduced frontend main bundle chunk from ~529 kB to ~334 kB using deterministic Vite vendor chunking.
- Unblocked Rust dependency resolution by removing stale `tantivy` path patch in `src-tauri/Cargo.toml`.
- Removed deprecated llama token decoding call by migrating from `token_to_str` to `token_to_piece`.
- Reconciled CI with repository reality by removing broken MemoryKernel/monorepo lane references and wiring executable jobs only.
- Repaired quality gate coverage comparison to use repo default remote branch and actual coverage artifact paths.
- Converted Playwright runner behavior from skip-success to explicit failure when regression suites are missing.
- Added least-privilege workflow token permissions (`permissions: contents: read`) to CI and quality gate workflows.
- Kept quality verification active on `push` while selectively skipping only `pnpm git:guard:all` to avoid invalid post-merge branch guard failures.

### Testing

- Added frontend unit test lane with Vitest and coverage output (`coverage/frontend/lcov.info`).
- Added jsdom-based accessibility label unit coverage for updated draft panel controls.
- Added real UI regression suites in `tests/ui` covering smoke, visual snapshot, and accessibility checks.
- Added workflow command drift checker with file/path existence validation to prevent CI/script mismatch regressions.
- Added deterministic Rust security audit gate (`pnpm test:security:audit:rust`) with explicit temporary waiver IDs.

### Docs

- Added Phase 3 closeout readiness report (`docs/reports/phase3-closeout-readiness.md`).
- Added `docs/SECURITY.md` and aligned README testing and verification commands with executable scripts.
- Added consolidated stabilization readiness evidence report (`docs/reports/week1-2-stabilization-readiness.md`).
- Added Week 3 carry-forward execution checklist (`docs/reports/week3-carry-forward-plan.md`).
