# Monorepo Structure Specification

## Root Repository
- Canonical root: `/Users/d/Projects/AssistSupport`
- Migration branch: `codex/monorepo-migration`

## Top-Level Architecture
1. AssistSupport remains at repository root during migration execution.
2. MemoryKernel is imported under `services/memorykernel`.
3. Shared governance and execution artifacts remain under `docs/implementation`.
4. Compliance controls and evidence index remain under `docs/compliance`.

## Required Directories
- `services/memorykernel/` (imported producer codebase)
- `docs/implementation/evidence/` (gate artifacts)
- `docs/compliance/` (control matrix and evidence index)
- `scripts/` (consumer governance validators)

## Boundary and Runtime Invariants
1. AssistSupport integration logic may call MemoryKernel only through the adapter boundary.
2. Enrichment is optional/non-blocking for all draft flows.
3. Deterministic fallback remains mandatory under offline/timeout/malformed/version mismatch/non-2xx paths.
4. Runtime cutover remains policy-gated and defaults to NO-GO unless bilateral decision records explicitly approve.

## Legacy Repo Relationship
- Source MemoryKernel repository remains authoritative for historical standalone development records.
- After operational switchover, legacy repo mode must be set to mirror/read-only or archive per governance policy.
