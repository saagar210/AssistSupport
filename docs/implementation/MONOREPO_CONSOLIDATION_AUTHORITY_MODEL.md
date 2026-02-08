# Monorepo Consolidation Authority Model

Updated: 2026-02-08
Owner: AssistSupport Program Lead

## Objective

Define a zero-ambiguity source-of-truth model for consolidated operation so cross-repo drift is blocked by policy and CI.

## Authority Hierarchy

1. Runtime authority (consumer): `/Users/d/Projects/AssistSupport/config/memorykernel-integration-pin.json`
2. Producer contract authority (canonical): `/Users/d/Projects/AssistSupport/services/memorykernel/contracts/integration/v1/producer-contract-manifest.json`
3. Consumer producer-manifest mirror: `/Users/d/Projects/AssistSupport/config/memorykernel-producer-manifest.json`
4. Human-readable compatibility policy: `/Users/d/Projects/AssistSupport/docs/MEMORYKERNEL_COMPATIBILITY_MATRIX.md`
5. Runtime decision posture and gates:
   - `/Users/d/Projects/AssistSupport/docs/implementation/ACTIVE_RUNTIME_STATUS_2026-02-08.md`
   - `/Users/d/Projects/AssistSupport/docs/implementation/SERVICE_V3_CUTOVER_DECISION_CHECKPOINT_2026-02-08.md`

## Ownership Boundaries

- AssistSupport ownership:
  - Consumer runtime adapter (`src-tauri/src/commands/memory_kernel.rs`)
  - Consumer governance scripts (`scripts/validate_memorykernel_*.mjs`)
  - Consumer policy and evidence docs under `docs/implementation/`
- MemoryKernel ownership:
  - Service/runtime contract and OpenAPI under `services/memorykernel/`
  - Producer handoff payload + manifest generation/validation scripts
  - Producer compliance and trilogy verification suite
- Joint ownership:
  - Cutover decision records
  - Rollback protocol and dry-run evidence
  - Compatibility matrix and baseline promotion policy

## Atomic Update Rule

When a producer baseline changes, the following files MUST be updated together in one PR:

1. `config/memorykernel-integration-pin.json`
2. `config/memorykernel-producer-manifest.json`
3. `docs/MEMORYKERNEL_COMPATIBILITY_MATRIX.md`

CI must fail if only a subset changes.

## Drift Detection Gates

- `pnpm run check:memorykernel-pin`
- `pnpm run check:memorykernel-governance`
- `pnpm run check:memorykernel-handoff`
- `pnpm run check:memorykernel-handoff:service-v3-candidate`
- `pnpm run test:memorykernel-governance-negative`

## Decision Escalation

If any gate fails:

1. Freeze baseline promotion (no pin changes).
2. Record failure in checkpoint docs.
3. Open rollback/recovery checklist if failure impacts runtime.
4. Resume only after all mandatory checks are green.
