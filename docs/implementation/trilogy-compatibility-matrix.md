# Trilogy Compatibility Matrix

Last updated: 2026-02-07

## Contract Baseline

- Integration contract root: `contracts/integration/v1/`
- Required schema set:
  - `context-package-envelope.schema.json`
  - `trust-gate-attachment.schema.json`
  - `proposed-memory-write.schema.json`
  - `error-envelope.schema.json`
- Contract change policy: any required-field or semantic breaking change requires `v2`.

## Compatibility Table

| Project | Required Contract Baseline | Required Integration Surface | Required Determinism/Guardrails |
|---|---|---|---|
| MemoryKernel | `contracts/integration/v1/*` canonical | `mk query ask`, `mk query recall`, `mk context show`, `mk outcome ...` host path | deterministic ordering metadata and explainable exclusions; Outcome benchmark threshold triplet in CI |
| OutcomeMemory | must byte-match MemoryKernel `contracts/integration/v1/*` | stable embed API only: `run_cli`, `run_outcome_with_db`, `run_outcome`, `run_benchmark` | benchmark threshold semantics with non-zero exit on any threshold violation |
| MultiAgentCenter | must byte-match MemoryKernel `contracts/integration/v1/*` | `ApiMemoryKernelContextSource` for `--memory-db`; trust identity requires `memory_id + version + memory_version_id` for `memory_ref` | deterministic `policy|recall` context-query behavior and replayable trace guarantees |

## Verification Commands

Run from MemoryKernel root:

```bash
./scripts/run_trilogy_phase_8_11_closeout.sh --soak-iterations 1
./scripts/verify_contract_parity.sh
./scripts/verify_trilogy_compatibility_artifacts.sh
./scripts/run_trilogy_smoke.sh
./scripts/run_trilogy_soak.sh --iterations 3
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
```

## Upgrade and Rollback Order

1. Update contract pack in MemoryKernel and complete schema/fixture tests.
2. Mirror contract pack in OutcomeMemory and MultiAgentCenter; pass parity checks.
3. Run trilogy smoke and full quality gates in each workspace.
4. Roll forward in integration order: MemoryKernel -> OutcomeMemory -> MultiAgentCenter.
5. Roll back in reverse order for compatibility incidents: MultiAgentCenter -> OutcomeMemory -> MemoryKernel.

## Current Baseline Lock (Pre-RC Finalization)

| Project | Workspace Version | Source |
|---|---|---|
| MemoryKernel | `0.1.0` | local workspace crates |
| OutcomeMemory | `0.1.0` | `trilogy-compatibility.v1.json` |
| MultiAgentCenter | `0.1.0` | `trilogy-compatibility.v1.json` |

Final RC version/sha locks are tracked in Phase 9 and must be recorded before final promotion.
