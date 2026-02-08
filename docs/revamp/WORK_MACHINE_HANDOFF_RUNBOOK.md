# AssistSupport Work-Machine Handoff Runbook

Status: Validated  
Date: 2026-02-08

## Goal
Provide a deterministic onboarding path for the work machine with no architecture-level rework required.

## Runtime Baseline
1. `release_tag`: `v0.4.0`
2. `commit_sha`: `7e4806a34b98e6c06ee33fa9f11499a975e7b922`
3. `service_contract_version`: `service.v3`
4. `api_contract_version`: `api.v1`
5. `integration_baseline`: `integration/v1`

## Preconditions
1. Git access to `[saagar210/AssistSupport](https://github.com/saagar210/AssistSupport)`.
2. Node, pnpm, Rust/Cargo, and Playwright prerequisites installed.
3. Required local environment variables configured.

## Bootstrap Steps
```bash
git clone https://github.com/saagar210/AssistSupport.git
cd AssistSupport
pnpm install
cd src-tauri && cargo fetch && cd ..
pnpm run typecheck
pnpm run test
pnpm run test:ci
```

## Governance Validation Steps
```bash
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
pnpm run check:memorykernel-handoff
pnpm run check:memorykernel-handoff:service-v3-candidate
pnpm run check:llm-golden-set
pnpm run check:rollback-readiness
pnpm run check:phase6-ops-hardening
pnpm run test:security-regression
pnpm run test:memorykernel-contract
pnpm run test:memorykernel-phase3-dry-run
pnpm run test:memorykernel-cutover-dry-run
```

## Operational Validation
1. Launch app and confirm Draft + queue-first flows render.
2. Confirm Settings loads and diagnostics panels are responsive.
3. Confirm deterministic fallback remains active when enrichment is unavailable.

## Rollback Readiness
1. Confirm rollback anchor exists: `revamp-baseline-2026-02-08`.
2. Confirm rollback check passes: `pnpm run check:rollback-readiness`.
3. Confirm rollback evidence artifact exists: `artifacts/rollback-readiness-evidence.json`.

## Signoff
1. Platform Owner: Approved
2. Operator Lead: Approved
3. Security Owner: Approved
