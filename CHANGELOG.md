# Changelog

## Unreleased

### Added

- Contract-governed CLI output envelope with `contract_version`.
- Versioned CLI output schemas under `contracts/v1/`.
- Golden fixtures and compatibility tests for key CLI outputs.
- Deterministic mixed-record recall retrieval (`query recall`) across decision/preference/event/outcome.
- API and service support for recall query flows and summary-record ingestion.
- Phase 3 resolver rules, explainability tests, and traceability requirements (`MKR-044`..`MKR-047`).
- Property-based determinism tests, concurrency integrity tests, and resolver benchmark coverage (`MKR-048`..`MKR-050`).
- Signed/encrypted snapshot trust controls with explicit import verification flags and security regression tests (`MKR-051`..`MKR-053`).
- Release workflow automation plus migration/recovery runbooks and pilot adoption docs (`MKR-054`..`MKR-056`).
- Host-integrated OutcomeMemory command tree under `mk outcome ...` with compatibility coverage from MemoryKernel CLI integration tests.

### Contract

- Active CLI contract version: `cli.v1`.
