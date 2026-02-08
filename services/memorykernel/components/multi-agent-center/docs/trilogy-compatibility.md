# Trilogy Compatibility Artifact

This artifact defines supported cross-project versions and required integration assumptions for
`MemoryKernel`, `OutcomeMemory`, and `MultiAgentCenter`.

## Supported Versions

| Component | Supported Range | Source |
|---|---|---|
| MemoryKernel | `>=0.1.0, <0.2.0` | `/Users/d/Projects/MemoryKernel/Cargo.toml` |
| OutcomeMemory | `>=0.1.0, <0.2.0` | `/Users/d/Projects/OutcomeMemory/Cargo.toml` |
| MultiAgentCenter | `0.1.x` | `/Users/d/Projects/MultiAgentCenter/Cargo.toml` |

## Required Contracts And Flags

- Integration pack `contracts/integration/v1/*` must remain parity-identical with MemoryKernel
  canonical `v1` pack; drift in `v1` is blocked by CI guard.
- Workflow `task.context_queries[].mode`:
  - `policy` (default)
  - `recall`
- Recall `record_types`:
  - missing or empty => defaults to MemoryKernel recall defaults
    (`decision`, `preference`, `event`, `outcome`)
  - invalid values => fail fast with validation error
- CLI `--memory-db`:
  - canonical path is MemoryKernel API-backed context sourcing
  - deterministic behavior validated across repeated runs for policy+recall modes
- CLI `--trust-db`:
  - optional OutcomeMemory trust gating source
  - trust persistence for `memory_ref` decisions must include:
    - `memory_id`
    - `version`
    - `memory_version_id`
- Replay (`replay --rerun-provider`):
  - runs from persisted workflow/context snapshots
  - trust `memory_ref` persistence identity remains strict for replay runs

## Operational Assumptions

- UTC RFC3339 timestamps for `as_of`.
- ULID identifiers for run/step/memory entities.
- SQLite trace schema with append-only protections for audit rows.
