# Phase 3: Retrieval Expansion

## Deliverables

- Deterministic retrieval flows for `decision`, `preference`, `event`, and `outcome`.
- Extended context package behavior with explainability guarantees.
- Resolver tests for mixed-record queries.

## Non-Goals

- Performance tuning beyond baseline correctness.
- Security signing/encryption enhancements.

## Rollback Criteria

- Nondeterministic ordering across repeated runs.
- Explainability fields missing from selected/excluded outputs.

## Exit Checklist

- [x] New resolver rules documented and normative.
- [x] Determinism tests pass for mixed-record retrieval.
- [x] Context package contract updated and validated.
- [x] Existing policy-query behavior remains regression-free.
