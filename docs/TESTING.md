# Testing Guide

This repository has three active test surfaces:
- Frontend unit tests (Vitest)
- Backend unit/integration tests (Rust/Cargo)
- Search API tests (pytest)

Run commands from the repository root unless noted otherwise.

## Frontend (TypeScript/React)

```bash
pnpm run typecheck
pnpm test
pnpm test:coverage
```

Run a single frontend test file:

```bash
pnpm vitest run src/components/Settings/SettingsTab.test.tsx
```

## E2E (Playwright)

```bash
pnpm run test:e2e:smoke
pnpm run test:e2e:ops
```

## Backend (Rust / Tauri)

```bash
cd src-tauri
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Search API (Python)

Install test dependencies and run tests:

```bash
cd search-api
python3 -m venv venv
source venv/bin/activate
python3 -m pip install -r requirements-test.txt
pytest -q

# Production config + auth smoke checks
ENVIRONMENT=production ASSISTSUPPORT_API_KEY=test-key ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI=redis://127.0.0.1:6379/0 python3 validate_runtime.py --check-backends
ENVIRONMENT=production ASSISTSUPPORT_API_KEY=test-key ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI=redis://127.0.0.1:6379/0 python3 smoke_search_api.py
```

## Security Checks

Dependency audits used in CI:

```bash
pnpm audit --audit-level high
cd src-tauri && cargo audit
```

## CI Parity

The canonical CI workflow is in `.github/workflows/ci.yml`. When validating changes locally, prefer the same commands and order used there.
