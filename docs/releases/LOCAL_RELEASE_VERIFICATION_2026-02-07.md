# Local Release Verification Report (2026-02-07)

- Commit baseline: `e6bfbf3`
- Verification timestamp (UTC): `2026-02-07T15:57:24Z`
- Scope: local release-candidate validation for frontend, backend, Search API, security audit, and packaging.

## Command Results

| Command | Result | Notes |
|---|---|---|
| `pnpm run typecheck` | PASS | TypeScript no-emit check passed. |
| `pnpm test` | PASS | 77/77 tests passed. |
| `pnpm run build` | PASS | Frontend production bundle generated in `dist/`. |
| `pnpm run test:e2e:smoke` | PASS | 4/4 smoke tests passed in Chromium. |
| `cd src-tauri && cargo fmt --check` | PASS | Formatting clean after clippy fixes. |
| `cd src-tauri && cargo clippy -- -D warnings` | PASS | Zero warnings after targeted remediations. |
| `cd src-tauri && cargo test` | PASS | All backend/unit/integration tests passed. |
| `pnpm audit --audit-level high` | PASS | No high-severity npm vulnerabilities. |
| `cd src-tauri && cargo audit` | PASS (with warnings) | 0 vulnerabilities; 22 allowed warnings (unmaintained/unsound) tracked in baseline. |
| `cd search-api && source venv/bin/activate && pytest -q` | PASS | 24/24 Search API tests passed. |
| `cd search-api && source venv/bin/activate && ENVIRONMENT=production ASSISTSUPPORT_API_KEY=local-test-key ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI=redis://127.0.0.1:6379/0 ASSISTSUPPORT_API_PORT=3390 python validate_runtime.py --check-backends --json` | PASS | Runtime config valid with backend connectivity checks. |
| `cd search-api && source venv/bin/activate && ENVIRONMENT=production ASSISTSUPPORT_API_KEY=local-test-key ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI=redis://127.0.0.1:6379/0 ASSISTSUPPORT_API_PORT=3390 python smoke_search_api.py` | PASS | `/health` returned 200; unauthenticated `/search` returned 401. |
| `pnpm tauri build` | PASS | Built app + DMG artifacts successfully. |

## Build Artifacts

- macOS app bundle: `/Users/d/Projects/AssistSupport/src-tauri/target/release/bundle/macos/AssistSupport.app`
- DMG: `/Users/d/Projects/AssistSupport/src-tauri/target/release/bundle/dmg/AssistSupport_1.0.0_aarch64.dmg`
- Release binary: `/Users/d/Projects/AssistSupport/src-tauri/target/release/assistsupport`

## Residual Risk Disposition

- `cargo audit` warnings remain transitive and non-vulnerability advisories in current dependency graph.
- The warning set is captured in `docs/security/DEPENDENCY_ADVISORY_BASELINE.md`.
- Weekly dependency watch workflow and triage policy are in place to detect and route changes.
