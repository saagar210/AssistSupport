# Producer Module Boundary Map

Updated: 2026-02-08
Owner: MemoryKernel

## Objective

Define module boundaries that must remain stable during monorepo operation and future migration phases.

## Boundary Domains

1. Service boundary
   - `crates/memory-kernel-service/`
   - Owns HTTP transport, request/response envelopes, operation timeout handling.
2. API boundary
   - `crates/memory-kernel-api/`
   - Owns stable consumer-facing API surface and storage orchestration.
3. Core domain boundary
   - `crates/memory-kernel-core/`
   - Owns domain model, determinism logic, and context package assembly.
4. Storage boundary
   - `crates/memory-kernel-store-sqlite/`
   - Owns migrations, schema status, persistence behavior.
5. Contract boundary
   - `contracts/integration/v1/`
   - Owns canonical producer-consumer machine-readable contract artifacts.
6. Governance boundary
   - `scripts/verify_*.sh` + `docs/implementation/*`
   - Owns release gates, handoff policies, and evidence requirements.

## Non-Negotiable Boundary Rules

- Service contract changes require: OpenAPI update + contract tests + manifest/handoff updates.
- Contract pack changes require parity checks for component mirrors.
- Governance scripts are treated as production controls; changes require negative-fixture validation.
