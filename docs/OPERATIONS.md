# AssistSupport Operations Runbook

This runbook covers day-to-day development operations, incident triage, security maintenance, and release execution.

## 1) Local Development and Validation

### Prerequisites
- Node.js 20+
- pnpm 10+
- Rust stable toolchain
- macOS build tooling for Tauri

### Startup
```bash
pnpm install --frozen-lockfile
pnpm dev
```

### Core verification commands
```bash
# Frontend
pnpm run typecheck
pnpm test
pnpm run test:coverage
pnpm run test:e2e:smoke

# Backend
cd src-tauri
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo llvm-cov --workspace --lcov --output-path ../coverage/backend/lcov.info
```

### CI parity check (recommended before pushing)
```bash
pnpm run typecheck && pnpm test && (cd src-tauri && cargo test)
```

## 2) CI Failure Triage

### Fast path triage sequence
1. Identify failing job in GitHub Actions (`lint-frontend`, `test-frontend`, `test-e2e-smoke`, `coverage-*`, `security-audit`, `test-search-api`, `check-backend`, `build`).
2. Reproduce locally with the same command.
3. Fix the smallest failing surface first (lint/type errors before runtime or integration failures).
4. Re-run only impacted checks, then run full local verification before pushing.

### Common failure patterns
- `test-frontend`: Usually mock drift or component behavior changes.
- `test-e2e-smoke`: Check `VITE_E2E_MOCK_TAURI=1` behavior and selectors.
- `coverage-*`: Confirm coverage providers and lcov output paths exist.
- `security-audit`: Update vulnerable dependencies, then rerun audit.
- `test-search-api`: Verify redis is reachable and run `python validate_runtime.py --check-backends` plus `python smoke_search_api.py`.
- `check-backend`: Ensure formatting/clippy pass with zero warnings.

## 2.1) MemoryKernel Runtime Lifecycle

AssistSupport treats MemoryKernel as optional enrichment. Core drafting must remain available under all MemoryKernel failure modes.

### Runtime policy
- Timeout budget:
  - Default `2500ms` per request (`ASSISTSUPPORT_MEMORY_KERNEL_TIMEOUT_MS` override).
- Preflight at startup:
  - `GET /v1/health` contract/version check.
  - `POST /v1/db/schema-version` readiness check.
- Retry/backoff:
  - No synchronous retry loop in startup flow.
  - Health/preflight status naturally re-evaluates on periodic status refresh.
- State transitions:
  - `ready`: enrichment enabled.
  - `disabled`, `offline`, `schema-unavailable`, `version-mismatch`, `malformed-payload`, `degraded`: enrichment disabled, fallback active.

### User-facing diagnostics
- Header system status includes MemoryKernel readiness detail.
- Draft generation continues in deterministic fallback mode when enrichment is unavailable.
- Preflight failures should be treated as actionable diagnostics, not fatal app errors.

### MemoryKernel pin upgrade runbook
1. Update `config/memorykernel-integration-pin.json` to the new approved tag and commit SHA.
2. Update `docs/MEMORYKERNEL_COMPATIBILITY_MATRIX.md` with the same tag/SHA pair in baseline and approved row.
3. Mirror producer manifest into `config/memorykernel-producer-manifest.json`.
4. Run governance synchronization check:
   ```bash
   pnpm run check:memorykernel-pin
   ```
   Optional strict mode (environments with direct producer repo access):
   ```bash
   ASSISTSUPPORT_VALIDATE_REMOTE_MANIFEST=1 MEMORYKERNEL_GITHUB_TOKEN=<token> pnpm run check:memorykernel-pin
   ```
5. Run MemoryKernel contract suite:
   ```bash
   pnpm run test:memorykernel-contract
   ```
6. Confirm generated evidence artifact exists locally:
   - `artifacts/memorykernel-contract-evidence.json`
7. Run full CI parity suite before push:
   ```bash
   pnpm run typecheck
   pnpm run test
   pnpm run test:ci
   ```
8. Rollback policy:
   - If contract gate fails on pin bump, revert pin+matrix+producer manifest mirror to last approved pair and re-run tests.

## 3) Backup/Restore Validation

Use this flow before releases and after backup/import logic changes.

1. Launch app with a test profile.
2. Create representative data:
   - saved drafts
   - templates
   - custom variables
   - custom decision trees
3. Export backup (both unencrypted `.zip` and encrypted `.enc`).
4. Preview import for both backups.
5. Import into a clean profile.
6. Verify counts and sample records match expectations.

### Optional CLI-assisted checks
- Confirm file exists and is readable.
- Confirm encrypted backups fail to preview without password.
- Confirm imported records are accessible in UI tabs.

## 4) Key Rotation and Security Checks

### Token/key hygiene
- Clear and reset Jira/HuggingFace/GitHub tokens after credential events.
- Verify audit entries are written for token set/clear actions.

### Session controls
- Validate session token creation and invalidation paths.
- Trigger lock flow (`lock_app`) and confirm unlock behavior.

### Security checks
```bash
pnpm audit --audit-level high
cd src-tauri && cargo audit
```

### Search API runtime checks
```bash
cd search-api
python3 -m venv venv
source venv/bin/activate
pip install -r requirements-test.txt
ENVIRONMENT=production ASSISTSUPPORT_API_KEY=test-key ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI=redis://127.0.0.1:6379/0 python validate_runtime.py --check-backends
ENVIRONMENT=production ASSISTSUPPORT_API_KEY=test-key ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI=redis://127.0.0.1:6379/0 python smoke_search_api.py
```

### Dependency watch cadence
- Weekly automated run: `.github/workflows/dependency-watch.yml` (Monday 14:00 UTC).
- Captures:
  - `pnpm audit --audit-level high`
  - `cargo audit`
  - `pnpm outdated`
  - `cargo update --dry-run`
- Auto-creates/updates a GitHub issue (`Dependency Watch Alerts`) when vulnerabilities or Rust advisory warnings are detected.
- Review the `dependency-watch-reports` artifact from each run and update dependencies when drift/advisories appear.

### File and path safety checks
- Keep KB paths inside user home.
- Avoid sensitive directories (`.ssh`, keychains, etc.).
- Verify path validation and SSRF tests remain green.

## 5) Release Checklist

1. Update changelog/version metadata.
2. Run full local verification (frontend + backend + e2e smoke).
3. Ensure all CI jobs pass on the release commit.
4. Build signed app artifact:
   ```bash
   pnpm tauri build
   ```
5. Validate startup, model load, KB search, draft generation, settings, and export/import in release build.
6. Tag release and publish notes.

## 6) Rollback Procedure

If release issues are detected:
1. Halt distribution of affected artifact.
2. Repoint users to previous stable release.
3. Revert problematic commit(s) on `master`.
4. Run full CI and smoke validation on rollback commit.
5. Publish rollback release notes with impact + remediation timeline.

## 7) Ownership Notes

- Frontend quality: Typecheck + unit tests + Playwright smoke suite
- Rust command contracts/security: Rust tests + security audit jobs
- CI governance: Coverage gates and artifact retention in `.github/workflows/ci.yml`
