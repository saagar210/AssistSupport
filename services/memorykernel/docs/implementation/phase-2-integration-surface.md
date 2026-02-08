# Phase 2: Integration Surface

## Deliverables

- Stable application API crate for consumer projects.
- Local service endpoint surface with versioned OpenAPI.
- Integration tests for service/API contract stability.

## Non-Goals

- Full retrieval strategy redesign for all record types.
- Production deployment orchestration.

## Rollback Criteria

- Contract instability across minor changes.
- Missing API parity with CLI foundation operations.

## Exit Checklist

- [x] API crate published in workspace with stable contract.
- [x] Service contract versioned and tested.
- [x] OpenAPI artifact committed and validated.
- [x] Consumer integration smoke tests pass.
