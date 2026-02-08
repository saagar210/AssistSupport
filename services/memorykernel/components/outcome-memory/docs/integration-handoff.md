# OutcomeMemory Integration Handoff

## Purpose
Defines the integration prerequisites and acceptance gates for adopting OutcomeMemory into sibling projects (e.g., MemoryKernel) once those projects are ready.

## Source-of-Truth Documents Pre-Integration
Until host projects are ready, treat these as the authority:
- `/Users/d/Projects/OutcomeMemory/docs/v1-contract-freeze.md`
- `/Users/d/Projects/OutcomeMemory/docs/performance.md`
- `/Users/d/Projects/OutcomeMemory/docs/integration-handoff.md`

## Migration Prerequisites
- Upstream schema must provide `memory_records` table with:
  - columns: `memory_id`, `version`
  - uniqueness: `UNIQUE(memory_id, version)`
- Outcome migration (`OUTCOME_MIGRATION_VERSION=2`) must run under the same SQLite DB file.
- Existing records must remain untouched (OutcomeMemory is additive).

## Command Mapping
Standalone OutcomeMemory commands map to integrated `mk` command tree:
- `mk outcome log ...`
- `mk outcome manual ...`
- `mk outcome system ...`
- `mk outcome trust show ...`
- `mk outcome replay ...`
- `mk outcome projector status|check|stale-keys ...`
- `mk outcome gate preview ...`
- `mk outcome benchmark run ...`

## Stable Embedded API
Host embedding must call the stable entrypoints in:
- `/Users/d/Projects/OutcomeMemory/crates/memory-kernel-outcome-cli/src/lib.rs`
- `run_cli`
- `run_outcome_with_db`
- `run_outcome`
- `run_benchmark`

No host project should call internal/private helper functions from this crate directly.

## Shared Integration Contract Pack

- Schemas: `contracts/integration/v1/schemas/`
- Fixtures: `contracts/integration/v1/fixtures/`
- Outcome integration tests validate trust/error artifacts against this pack:
  - `/Users/d/Projects/OutcomeMemory/crates/memory-kernel-outcome-cli/tests/integration_contracts.rs`
- CI parity guard compares this pack against MemoryKernel canonical pack:
  - `/Users/d/Projects/OutcomeMemory/scripts/check_contract_pack_parity.sh`

Hosted CI canonical resolution behavior:
- Preferred: set repository variable `MEMORYKERNEL_CANONICAL_REPO` so CI checks out canonical contracts into `_memorykernel/contracts/integration/v1`.
- Fallback: if that variable is not set, parity check uses sibling path `../MemoryKernel/contracts/integration/v1`.
- If neither source exists, parity check fails fast to prevent silent contract drift.

## Trilogy Compatibility Artifact
- Compatibility map:
  - `/Users/d/Projects/OutcomeMemory/trilogy-compatibility.v1.json`
- Captures supported semver ranges and required integration capabilities for:
  - OutcomeMemory
  - MemoryKernel
  - MultiAgentCenter

## Test Requirements Before Integration
- CLI smoke tests pass:
  - projector unhealthy before replay and healthy after replay.
- Contract tests pass:
  - help surface, JSON contract versions, stable error shape.
- Store invariants pass:
  - append-only, replay determinism, ruleset upgrade replay, version isolation.
- Performance guardrails pass:
  - append/replay budgets within configured limits.

## Rollback Strategy
- Outcome tables are append-only and additive; rollback is operationally done by:
  1. Stop writing new outcome events.
  2. Disable read-path gating integration in host project.
  3. Keep historical tables intact for auditability.
- No destructive data reset is required for rollback.

## Ready-to-Integrate Gate
A branch is ready to integrate when:
- `cargo test` passes in OutcomeMemory workspace.
- CI smoke workflow passes.
- CI perf workflow passes on both Linux and macOS runners.
- Linux baseline has been replaced from a real CI artifact (not bootstrap seed).
- Contract freeze doc remains valid (`v1-contract-freeze.md`).

## Adapter Start Criteria (MemoryKernel Still In Build Phase)
Start adapter wiring only when all are true:
- MemoryKernel DB contract is stable for `memory_records(memory_id, version)` uniqueness.
- MemoryKernel command surface can host `mk outcome ...` subcommands without breaking existing contracts.
- Integration branch agrees to consume OutcomeMemory `v1` JSON contracts unchanged.
- Cross-project migration order is defined: MemoryKernel migration first, then OutcomeMemory additive migrations.
