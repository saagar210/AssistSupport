# Phase 7: Trilogy Convergence and Contract Governance

## Deliverables

- Canonical integration-contract ownership in MemoryKernel for `contracts/integration/v1/*`.
- Automated contract parity checks against OutcomeMemory and MultiAgentCenter.
- Trilogy compatibility matrix with required contract and command assumptions.
- Cross-project deterministic smoke command sequence for local release readiness.

## Non-Goals

- Introducing `integration/v2` contracts.
- Refactoring OutcomeMemory or MultiAgentCenter internal architecture from MemoryKernel.
- Expanding public API surface without explicit version-governance updates.

## Rollback Criteria

- Contract file-set or content drift is detected across trilogy repositories.
- Host integration no longer uses documented stable Outcome embed surface.
- Deterministic policy/recall expectations fail under repeated identical inputs.

## Exit Checklist

- [x] MemoryKernel contract pack is documented as canonical for trilogy integration v1.
- [x] CI and release workflows execute parity checks (strict with sibling repos present, explicit allow-missing fallback for isolated CI clones).
- [x] Compatibility matrix is published and versioned.
- [x] Trilogy smoke script exists for deterministic policy/recall + Outcome host checks.
- [x] Sibling repos complete parity automation and publish matching compatibility artifacts.
- [x] MemoryKernel validates sibling compatibility artifacts in CI/release gates.
- [x] Trilogy release gate runbook is executed end-to-end as a final coordinated pre-release check (see `docs/implementation/trilogy-release-report-2026-02-07.md`).
