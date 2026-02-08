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
- Active service contract version: `service.v2`.
- Service error responses now include machine-readable `error.code` + `error.message` with explicit status mapping.
- `service.v2` non-2xx envelope policy is locked: includes `service_contract_version` + `error` + `legacy_error`, excludes `api_contract_version`; `legacy_error` removal deferred to `service.v3`.
