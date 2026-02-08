# OutcomeMemory v1 Contract Freeze

## Scope
This document freezes OutcomeMemory standalone contracts before MemoryKernel integration.

## Frozen Contracts
- CLI command surface under `mk outcome ...`:
  - `log`, `manual`, `system`, `trust`, `replay`, `benchmark`, `projector`, `gate`, `events`
- Embedded host API surface:
  - `/Users/d/Projects/OutcomeMemory/crates/memory-kernel-outcome-cli/src/lib.rs`
  - Stable entrypoints: `run_cli`, `run_outcome_with_db`, `run_outcome`, `run_benchmark`
- JSON contract payload versions:
  - Gate preview: `gate_preview.v1`
  - Projector status: `projector_status.v1`
  - Projector check: `projector_check.v1`
  - Benchmark report: `benchmark_report.v1`
- SQLite contract:
  - Tables: `outcome_rulesets`, `outcome_events`, `memory_trust`, `outcome_projection_state`
  - Append-only triggers on `outcome_events`
  - Compatibility prerequisite: upstream `memory_records` with UNIQUE `(memory_id, version)`

## v1 Invariants Matrix
- Append-only: outcome events cannot be updated or deleted.
- Deterministic replay: full replay and incremental replay produce equivalent trust state.
- Version isolation: trust never smears across `(memory_id, version)` boundaries.
- Ruleset pinning: each event replays with its own `ruleset_version`.
- Projector health consistency: stale keys are detectable pre-replay and clear post-replay.

## Contract Tests
- CLI contracts:
  - `/Users/d/Projects/OutcomeMemory/crates/memory-kernel-outcome-cli/tests/cli_contracts_v1.rs`
- Projector smoke:
  - `/Users/d/Projects/OutcomeMemory/crates/memory-kernel-outcome-cli/tests/projector_smoke.rs`
- Shared integration contract pack:
  - `/Users/d/Projects/OutcomeMemory/crates/memory-kernel-outcome-cli/tests/integration_contracts.rs`
  - `/Users/d/Projects/OutcomeMemory/scripts/check_contract_pack_parity.sh`
- Store schema/invariant tests:
  - `/Users/d/Projects/OutcomeMemory/crates/memory-kernel-outcome-store-sqlite/src/lib.rs`
