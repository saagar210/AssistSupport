# MultiAgentCenter Architecture

Controlled execution layer with deterministic workflow normalization, explicit Context Package
injection, policy/trust/human gates, and append-only SQLite traces.

## CLI highlights

- `run --workflow <path> --trace-db <path>` executes a workflow.
- `run --memory-db <path>` enables MemoryKernel API-backed context package sourcing over the target SQLite database.
- `run --trust-db <path> --trust-mode safe|exploration` enables OutcomeMemory trust gating.
- `replay --run-id <id>` verifies audit replay chain integrity.
- `replay --run-id <id> --rerun-provider` creates a new run from stored workflow/context snapshots.

## Core behaviors

- Workflow YAML is normalized and hashed before execution; normalized snapshot is persisted.
- `run_id` and `step_id` are ULIDs.
- `as_of` is required in run trace and defaults to UTC now when omitted.
- Step context injection accepts one or more Context Packages per step (via `task.context_queries`).
  - `context_queries[].mode` supports `policy` (default) and `recall`.
  - Missing `mode` is normalized to `policy`.
  - `recall.record_types` behavior:
    - Missing `record_types`: uses MemoryKernel default recall scope (`decision`, `preference`, `event`, `outcome`).
    - Empty `record_types: []`: same as missing, uses default recall scope.
    - Invalid/non-string values: workflow run fails fast with explicit validation error.
  - `recall` mode uses MemoryKernel recall resolver semantics and never bypasses MemoryKernel APIs.
- Gate decisions are persisted per step, including trust decisions per memory reference.
  - Trust attachments and trust gate records for `memory_ref` subjects require explicit `memory_version_id`.
- Run manifest hash is stored in `runs.manifest_hash` with signature status (`unsigned` today).
- Provider layer is adapter-based:
  - `mock` deterministic test provider.
  - `http_json` real HTTP adapter path without core orchestrator changes.

## Shared integration contracts

- Schemas: `contracts/integration/v1/schemas/`
- Fixtures: `contracts/integration/v1/fixtures/`
- Contract tests validate envelope compatibility against the shared v1 pack.
