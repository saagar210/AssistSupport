# Baseline Metrics (Pre-Revamp)

Status: Captured  
Date: 2026-02-08  
Branch: `master`  
Baseline Tag: `revamp-baseline-2026-02-08`  
Baseline Commit: `bc77ae5286e6a9fdabe6b6e64322202f3cb9f989`

## 1) Runtime and Integration Baseline
1. MemoryKernel release tag: `v0.4.0`
2. MemoryKernel commit SHA: `7e4806a34b98e6c06ee33fa9f11499a975e7b922`
3. Service/API contract expectation: `service.v3` / `api.v1`
4. Integration baseline: `integration/v1`
5. Default MemoryKernel timeout: `2500ms`

Source of truth:
- `config/memorykernel-integration-pin.json`
- `config/memorykernel-producer-manifest.json`

## 2) Verification Snapshot
Captured outputs:
1. `docs/revamp/evidence/BASELINE_COMMAND_OUTPUT_2026-02-08.md`
2. `docs/revamp/evidence/BASELINE_CI_OUTPUT_2026-02-08.md`

Command summary:
1. `pnpm run typecheck` -> PASS
2. `pnpm run test` -> PASS (82 tests)
3. `pnpm run check:memorykernel-governance` -> PASS
4. `pnpm run check:memorykernel-handoff:service-v3-candidate` -> PASS
5. `pnpm run test:ci` -> PASS (frontend + rust suite)

## 3) Product Baseline (Observed)
1. Existing information architecture is tab-centric and Draft-first.
2. MemoryKernel enrichment path is integrated and guarded by governance checks.
3. Startup preflight behavior is non-blocking (recent hardening complete).
4. Audit logs now support search/filter/pagination.
5. Keyboard tab switching includes Ops and Settings paths.

## 4) Baseline Performance Markers (from test logs)
1. Frontend test suite duration: ~1.3s to 1.4s.
2. Full `test:ci` suite includes Rust test corpus and completes successfully in current local environment.
3. No failing governance checks at baseline capture time.

## 5) Revamp Guardrails Derived from Baseline
1. Do not regress deterministic fallback behavior.
2. Do not regress MemoryKernel governance checks.
3. Maintain full local test pass status at every phase gate.
4. Keep baseline tag intact for fast rollback.
