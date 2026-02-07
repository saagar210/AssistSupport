# Release Candidate Readiness Checklist

## Preconditions

- [ ] `master` branch is green in CI.
- [ ] Dependency alerts reviewed and triaged.
- [ ] Security advisories baseline reviewed (`docs/security/DEPENDENCY_ADVISORY_BASELINE.md`).

## Validation Matrix

- [ ] `pnpm run typecheck`
- [ ] `pnpm test`
- [ ] `pnpm run build`
- [ ] `pnpm run test:e2e:smoke`
- [ ] `cd src-tauri && cargo fmt --check`
- [ ] `cd src-tauri && cargo clippy -- -D warnings`
- [ ] `cd src-tauri && cargo test`
- [ ] `pnpm audit --audit-level high`
- [ ] `cd src-tauri && cargo audit`
- [ ] `cd search-api && pytest -q`
- [ ] `cd search-api && ENVIRONMENT=production ASSISTSUPPORT_API_KEY=test-key ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI=redis://127.0.0.1:6379/0 python validate_runtime.py --check-backends`
- [ ] `cd search-api && ENVIRONMENT=production ASSISTSUPPORT_API_KEY=test-key ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI=redis://127.0.0.1:6379/0 python smoke_search_api.py`

## Artifact Validation

- [ ] `pnpm tauri build` completes.
- [ ] App launches from built artifact.
- [ ] Draft generation, KB search, and settings flows pass smoke checks.

## Operational Readiness

- [ ] Backup/import drill result attached.
- [ ] Rollback drill result attached.
- [ ] Handoff checklist completed (`docs/releases/RELEASE_HANDOFF_CHECKLIST.md`).
