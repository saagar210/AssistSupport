# Memory Kernel v1 Normative Requirements

This document is normative. RFC 2119 keywords (`MUST`, `MUST NOT`, `SHOULD`, `MAY`) are binding.

## Core Requirements

- `MKR-001` The system MUST support exactly these base record types in v1: `constraint`, `decision`, `preference`, `event`, `outcome`.
- `MKR-002` Memory records MUST be append-only. Existing rows MUST NOT be updated in place for semantic changes.
- `MKR-003` Record evolution MUST be modeled through explicit lineage links: `supersedes` and `contradicts`.
- `MKR-004` Every memory write MUST include `source_uri`. `source_hash` MAY be omitted.
- `MKR-005` If `source_hash` is present, it MUST use `sha256:<hex>` format.
- `MKR-006` `truth_status` and `confidence` MUST be separate fields and MUST NOT be conflated.
- `MKR-007` The authority lattice MUST be `authoritative > derived > note`.
- `MKR-008` Retrieval MUST produce Context Packages rather than raw table output.
- `MKR-009` Context Package MUST include selected items, excluded items, deterministic ordering metadata, and per-item explanations.
- `MKR-010` Every write MUST include explicit `writer` and `justification` values.

## Foundation Query Requirements

- `MKR-011` The system MUST support a policy query: `Am I allowed to use a USB drive?` through structured normalization to `{actor, action, resource}`.
- `MKR-012` Query evaluation MUST be deterministic for identical input records and identical snapshot inputs.
- `MKR-013` Deterministic ordering MUST use the fixed precedence tuple defined in `docs/spec/resolver.md`.
- `MKR-014` Final tie-break chain MUST be immutable `memory_id` ascending, then immutable `memory_version_id` ascending.
- `MKR-015` `retracted` records MUST be excluded from selected results by default and MUST appear in exclusions with reason.
- `MKR-016` Superseded records MUST be excluded from selected results by default and MUST appear in exclusions with reason.
- `MKR-017` If top-precedence effective constraints conflict between `allow` and `deny`, the answer MUST be `inconclusive`.

## Time, Identity, and Output Requirements

- `MKR-018` All timestamps MUST be RFC3339 with UTC offset `Z`.
- `MKR-019` Public identifiers MUST use ULID values for both `memory_id` and `memory_version_id`.
- `MKR-020` Context Package serialization in JSON mode MUST be stable and deterministic.
- `MKR-021` Query output MUST include applied ruleset version and tie-breaker list.
- `MKR-028` Every persisted memory version MUST have a unique `memory_version_id` and that ID MUST be included in retrieval output items.
- `MKR-029` Lineage links (`supersedes`, `contradicts`) MUST reference `memory_version_id` values, not `memory_id` values.

## CLI and Persistence Requirements

- `MKR-022` CLI write commands MUST reject requests missing required accountability/provenance fields.
- `MKR-023` Schema constraints MUST enforce enum validity and referential integrity.
- `MKR-024` SQLite runtime MUST enable `WAL`, foreign keys, and a bounded busy timeout.
- `MKR-025` Migrations MUST be forward-only and recorded in schema version state.
- `MKR-030` Persistence MUST key `memory_records` by `memory_version_id` and enforce `UNIQUE(memory_id, version)`.
- `MKR-031` If a database is missing required version-id schema columns, startup MUST fail explicitly with actionable migration guidance.
- `MKR-032` CLI MUST expose schema inspection and dry-run migration planning without forcing mutation.
- `MKR-033` CLI MUST support deterministic snapshot export and import using NDJSON files plus a manifest.
- `MKR-034` CLI MUST support SQLite backup/restore operations and an explicit integrity-check report.

## Prohibited Ambiguity

- `MKR-026` Unspecified behavior MUST result in explicit error responses, not implicit fallback.
- `MKR-027` Normative documentation MUST avoid ambiguous terms such as "usually" and "etc." in requirements.

## Contract and Rollout Governance

- `MKR-035` Every successful CLI JSON output MUST include a top-level `contract_version`.
- `MKR-036` The repository MUST publish versioned JSON Schemas for CLI success outputs under `contracts/`.
- `MKR-037` Golden output fixtures for key CLI commands MUST be validated in CI using deterministic normalization.
- `MKR-038` Contract changes (schemas, CLI output shape, service OpenAPI) MUST include an explicit contract version bump and changelog entry.
- `MKR-039` Each implementation phase MUST have a written exit checklist including deliverables, non-goals, and rollback criteria.
- `MKR-040` Cross-project adoption MUST NOT begin until both Phase 2 and Phase 3 exit gates are completed.

## Phase 2 Integration Requirements

- `MKR-041` The workspace MUST expose a stable application API crate for local consumers.
- `MKR-042` The project MUST expose a local HTTP service for core foundation operations.
- `MKR-043` The service MUST publish a versioned OpenAPI artifact at `openapi/openapi.yaml`.

## Phase 3 Retrieval Expansion Requirements

- `MKR-044` The resolver MUST support deterministic recall retrieval for `decision`, `preference`, `event`, and `outcome` records.
- `MKR-045` Recall retrieval MUST emit exclusions with explicit reasons for at least `retracted`, `superseded`, and non-matching records.
- `MKR-046` Recall retrieval ordering MUST use a fixed precedence tuple and deterministic tie-breakers documented in `docs/spec/resolver.md`.
- `MKR-047` CLI, API, and service layers MUST expose a recall query operation that persists a Context Package artifact.

## Phase 4 Hardening Requirements

- `MKR-048` Deterministic retrieval guarantees MUST be validated with property-based tests over randomized record permutations.
- `MKR-049` Store read/write behavior MUST be validated under concurrent access without integrity violations.
- `MKR-050` Performance budgets for core retrieval paths MUST be documented and enforced in CI.

## Phase 5 Security and Trust Requirements

- `MKR-051` Snapshot exports MUST support manifest signatures, and import verification MUST fail on invalid signatures.
- `MKR-052` Threat model and abuse-case documentation MUST be maintained for retrieval, import/export, and local service surfaces.
- `MKR-053` Backup/export trust guarantees, including optional encryption controls, MUST be documented and test-covered.

## Phase 6 Release and Adoption Requirements

- `MKR-054` Release checks MUST be automated through a repeatable workflow aligned to semantic versioning.
- `MKR-055` Migration, rollback, and recovery runbooks MUST be versioned and validated against current commands.
- `MKR-056` Pilot integration acceptance criteria and adoption decisions MUST be recorded before broader rollout.

## Phase 7 Trilogy Convergence Requirements

- `MKR-057` MemoryKernel MUST be the canonical source for `contracts/integration/v1/*`, and cross-repo parity checks MUST fail on file-set or content drift.
- `MKR-058` Trilogy release readiness MUST include a versioned compatibility matrix and an executable cross-project smoke gate.
- `MKR-059` MemoryKernel CI and release workflows MUST validate sibling trilogy compatibility artifacts and fail when required integration assumptions are incompatible.

## Phase 8-11 Release Train Requirements

- `MKR-060` Hosted CI configuration dependencies required for trilogy parity checks MUST be explicitly tracked in release decisions with owning project identified.
- `MKR-061` Trilogy release-candidate promotion order MUST be fixed to `MemoryKernel -> OutcomeMemory -> MultiAgentCenter`, with reverse rollback order documented.
- `MKR-062` Trilogy soak verification MUST support repeated deterministic gate execution and require a published soak/release report artifact.
- `MKR-063` Final trilogy release approval MUST be documented with explicit preconditions, decisions, and post-release policy commitments.
- `MKR-064` A single executable closeout command MUST exist for Phases 8-11 that runs local trilogy gates and supports optional hosted-evidence validation.
