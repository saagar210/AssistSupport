# Test Catalog

## Determinism

- `TDET-001` Same inputs and same snapshot produce byte-identical Context Package JSON.
- `TDET-002` Input insertion order permutations do not change selected/excluded ordering.
- `TDET-003` Tie-break chain is `memory_id` ascending then `memory_version_id` ascending when all prior tuple dimensions are equal.
- `TDET-004` Recall query output remains byte-identical for permuted mixed-record input.
- `TDET-005` Property-based policy retrieval determinism holds across randomized input permutations.
- `TDET-006` Property-based recall retrieval determinism holds across randomized input permutations.

## Resolver

- `TRES-001` `retracted` candidates appear in exclusions with reason.
- `TRES-002` Superseded candidates appear in exclusions with reason.
- `TRES-003` Top-precedence allow+deny conflict returns `inconclusive`.
- `TRES-004` Authoritative deny outranks derived allow.
- `TRES-005` Recall retrieval emits explainable exclusions for retracted/superseded/non-overlap candidates.
- `TRES-006` Recall default scope includes non-constraint record types only.

## Write Validation

- `TWR-001` Missing writer rejected.
- `TWR-002` Missing justification rejected.
- `TWR-003` Missing source_uri rejected.
- `TWR-004` Invalid source_hash format rejected.
- `TWR-005` inferred/speculative without confidence rejected.

## Identity and Lineage

- `TID-001` Every inserted record has distinct `memory_version_id`.
- `TID-002` `(memory_id, version)` uniqueness enforced.
- `TID-003` Link and lineage targets use `memory_version_id` values.
- `TID-004` Context Package selected/excluded items always include `memory_version_id`.

## Store/Migrations

- `TDB-001` v1 migration succeeds on empty DB.
- `TDB-002` foreign key and check constraints active.
- `TDB-003` write/read round-trip preserves all fields.
- `TDB-004` startup fails explicitly when required version-id columns are missing.
- `TDB-005` schema status reports current and pending migration versions correctly.
- `TDB-006` export/import snapshot round-trip preserves memory records and context packages.
- `TDB-007` backup/restore round-trip preserves memory records.
- `TDB-008` integrity-check reports healthy database with no FK violations on clean state.
- `TDB-009` import rejects snapshot when manifest digest does not match NDJSON contents.
- `TCONC-001` concurrent writer/reader workloads preserve record integrity and pass integrity-check.

## CLI

- `TCLI-001` add constraint command writes a valid record.
- `TCLI-002` query ask emits and persists Context Package.
- `TCLI-003` context show returns persisted package by id.
- `TCLI-004` `memory link --from/--to` validates version IDs.
- `TCLI-005` `db export` and `db import` commands run successfully and report structured summaries.
- `TCLI-006` `db backup`, `db restore`, and `db integrity-check` commands run successfully and return structured output.
- `TCLI-007` `query recall` returns deterministic mixed-record Context Package output and persists package ids.

## Contract

- `TCON-001` Key CLI outputs validate against versioned JSON Schemas in `contracts/v1/schemas`.
- `TCON-002` Golden fixtures for key outputs stay stable after deterministic normalization.
- `TCON-003` Successful CLI outputs include top-level `contract_version` and match the active schema version.
- `TTRI-001` Trilogy integration contract parity checker succeeds and reports no drift when sibling repos are present.
- `TTRI-002` Trilogy compatibility artifacts from OutcomeMemory and MultiAgentCenter pass MemoryKernel consumer validation.
- `TTRI-003` Producer contract manifest validates service/api baselines, canonical error-code enum, and notice-policy guardrails.

## API and Service

- `TAPI-001` API crate can add a constraint, ask query, and load persisted context package.
- `TAPI-002` API crate can add summary records and execute recall retrieval with deterministic metadata.
- `TSVC-001` Service health endpoint returns success envelope with service contract version.
- `TSVC-002` Service add/query/context flow returns consistent persisted context package id.
- `TSVC-003` Service OpenAPI endpoint returns the versioned OpenAPI artifact for `service.v3`.
- `TSVC-004` Service summary-add and recall-query flow returns a persisted recall Context Package.
- `TSVC-005` Missing context package lookup returns machine-readable `context_package_not_found` with `404`.
- `TSVC-006` Validation failures return machine-readable `validation_error` with `400`.
- `TSVC-007` JSON parse failures return machine-readable `invalid_json` with `400`.
- `TSVC-008` Duplicate identity writes return machine-readable `write_conflict` with `409`.
- `TSVC-009` Non-2xx error envelope keeps `service.v3` shape (`service_contract_version` + `error`) and excludes `api_contract_version` + `legacy_error`.
- `TSVC-010` Service readiness endpoint returns `ready` when schema is current.
- `TSVC-011` Service readiness endpoint returns `schema_unavailable` when DB path is unreachable.
- `TSVC-012` Service blocking-dispatch helper returns successful values for fast operations.
- `TSVC-013` Service blocking-dispatch helper returns mapped timeout errors with deterministic details.
- `TSVC-014` Service telemetry counters increment deterministically for success, failure, and timeout paths.
- `TSVC-015` Summary-write validation failures return machine-readable `validation_error` with `400`.
- `TSVC-016` Memory-link JSON parse failures return machine-readable `invalid_json` with `400`.

## Performance

- `TPERF-001` Policy context-package generation meets the baseline CI budget.
- `TPERF-002` Recall context-package generation meets the baseline CI budget.

## Security

- `TSEC-001` Signed snapshot imports require verification and fail on tampered manifests.
- `TSEC-002` Encrypted snapshot imports require decrypt keys and succeed with valid keys.

## Documentation Quality

- `TDOC-001` Ambiguous terms (`usually`, `etc.`) are disallowed in spec docs except the explicit `MKR-027` exception line.
- `TDOC-002` Phase implementation checklist docs exist and include required sections.
- `TDOC-003` Versioning policy doc and changelog exist and declare the active contract version.
- `TDOC-004` Cross-project adoption gate doc exists and references Phase 2 and Phase 3 completion as required.
- `TDOC-005` Security threat-model and trust-control docs exist and describe import/export controls.
- `TDOC-006` Release workflow, migration/recovery runbooks, and pilot acceptance docs exist with actionable criteria.
- `TDOC-007` Phase 7 convergence docs and trilogy release-gate docs exist with required sections.
- `TDOC-008` Phase 8-11 release-train docs and trilogy release report artifacts exist with required sections.
- `TDOC-009` Phase 8-11 closeout playbook and executable closeout script exist and document hosted-evidence capture flow.
