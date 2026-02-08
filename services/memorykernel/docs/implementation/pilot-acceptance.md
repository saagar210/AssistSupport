# Pilot Integration Acceptance Criteria

## Candidate Integrations

- OutcomeMemory host integration via `mk outcome ...`
- MultiAgentCenter orchestration integration via MemoryKernel API-backed context sourcing

## Entry Criteria

- Phase 2 checklist complete
- Phase 3 checklist complete
- CI green for determinism, contract, and security tests

## Acceptance Criteria

1. Policy query parity:
   - `query ask` answers match expected allow/deny/inconclusive outcomes for pilot fixtures.
2. Recall retrieval parity:
   - `query recall` returns deterministic ordering for mixed-record fixtures.
3. Trust controls:
   - signed snapshots verify in pilot environment.
   - encrypted snapshots require and validate decrypt keys.
4. Operational readiness:
   - migration and recovery runbooks executed successfully in pilot.

## Exit Artifacts

- Pilot test report with pass/fail results
- Current report: `docs/implementation/pilot-report-2026-02-07.md`
- List of blocking defects (if any)
- Adoption decision recorded in `docs/implementation/adoption-decisions.md`
