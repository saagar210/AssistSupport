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
- Added machine-readable producer baseline manifest at `contracts/integration/v1/producer-contract-manifest.json` with CI/release enforcement.

### Governance

- Added producer-side joint execution artifacts for AssistSupport parallel execution:
  - `docs/implementation/JOINT_EXECUTION_PLAYBOOK_PRODUCER.md`
  - `docs/implementation/SERVICE_V3_RFC_DRAFT.md`
  - `docs/implementation/PRODUCER_PHASE1_STABILITY_CLOSEOUT_2026-02-08.md`
  - `docs/implementation/PRODUCER_RELEASE_HANDOFF_PACKET_TEMPLATE.md`
  - `docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json`
- Added `scripts/generate_producer_handoff_payload.sh` to generate deterministic producer handoff payloads from the canonical manifest.
- Extended producer handoff payload generation with `--mode service-v3-candidate` for rehearsal-only consumer validation while keeping runtime baseline unchanged.
- Added Phase 4 rehearsal governance artifacts:
  - `docs/implementation/SERVICE_V3_CUTOVER_GATES.md`
  - `docs/implementation/SERVICE_V3_REHEARSAL_HANDOFF_CANDIDATE.json`
  - `docs/implementation/SERVICE_V3_REHEARSAL_VERIFICATION_EVIDENCE.md`
