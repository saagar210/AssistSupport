# Phase 1: Contract Freeze

## Deliverables

- Versioned JSON Schemas for CLI success outputs.
- Golden fixtures for key command outputs.
- Contract compatibility tests in CI.
- Top-level `contract_version` in success outputs.

## Non-Goals

- Service transport/API implementation.
- Non-foundation retrieval expansion.

## Rollback Criteria

- Schema validation regressions in CI.
- Breaking output changes without a contract version bump.

## Exit Checklist

- [x] Schemas exist under `contracts/v1/`.
- [x] Golden fixtures are committed and validated.
- [x] Contract tests pass in CI.
- [x] Changelog updated for contract-affecting changes.
