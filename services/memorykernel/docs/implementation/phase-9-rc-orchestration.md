# Phase 9: RC Orchestration and Version Lock

## Deliverables

- Trilogy RC sequencing plan fixed to integration order.
- Compatibility matrix updated with locked versions and evidence references.
- RC rollback order documented and verified by dry-run criteria.
- RC lock capture procedure centralized in `trilogy-closeout-playbook.md`.

## Non-Goals

- Introducing new cross-project behaviors after RC freeze.
- Re-scoping contract semantics in `integration/v1`.
- Broad refactors unrelated to release blockers.

## Rollback Criteria

- RC ordering is bypassed and creates incompatible dependency states.
- Version lock metadata is incomplete or inconsistent across repos.
- Required release-gate checks fail after any RC update.

## Exit Checklist

- [x] RC promotion order is documented: MemoryKernel -> OutcomeMemory -> MultiAgentCenter.
- [x] Rollback order is documented: MultiAgentCenter -> OutcomeMemory -> MemoryKernel.
- [x] RC lock metadata format is defined (SemVer + commit SHA + gate evidence reference).
- [x] Compatibility matrix contains final locked RC versions/SHAs for all three repos.
- [x] Full trilogy release gate passes after each RC lock update.
