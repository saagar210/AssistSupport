# Service.v3 RFC Draft (Producer Planning)

## Status
- Draft
- Scope: planning/governance only (no runtime cutover in this draft)

## Objective
Define a safe, testable transition from `service.v2` to `service.v3` where
`legacy_error` can be removed from non-2xx envelopes without breaking AssistSupport.

## Current baseline
- Producer release baseline: `v0.3.2`
- Producer commit baseline: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- Active contracts:
  - `service.v2`
  - `api.v1`
  - `integration/v1`
- Rehearsal planning commit seen by consumer:
  - `483b01a7e39c7043b71d81707ff59a65f7f230b1`

## Non-negotiables
1. No `legacy_error` removal while `service.v2` is active.
2. Consumer (`AssistSupport`) must complete repin + green CI before any `service.v3` cutover.
3. Deterministic fallback and optional/non-blocking enrichment behavior must remain intact.
4. No unannounced additive `service.v2` error-code changes.

## Proposed `service.v3` direction
- Non-2xx envelope remains machine-readable with:
  - `service_contract_version`
  - `error.code`
  - `error.message`
  - optional `error.details`
- `legacy_error` removed in `service.v3`.
- `api_contract_version` remains excluded from non-2xx (locked policy for service.v3 unless future joint RFC changes this).

## Rehearsal handoff artifact
Producer publishes a rehearsal candidate payload at:
- `docs/implementation/SERVICE_V3_REHEARSAL_HANDOFF_CANDIDATE.json`
- `docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json` (generated in `service-v3-candidate` mode)

This artifact is intended for immediate consumer CI validation and must include:
1. Active runtime baseline (`v0.3.2` and associated immutable commit).
2. Rehearsal target (`service.v3`) with explicit no-runtime-cutover mode.
3. Non-2xx envelope policy for both stable (`service.v2`) and rehearsal candidate (`service.v3`).
4. Error-code validation mode and current enum.
5. Required producer and consumer verification command sets.
6. Rollback triggers and rollback action.

Compatibility expectation (candidate payload vs pinned runtime baseline):
1. Candidate payload sets `expected_service_contract_version = service.v3`.
2. The same payload must retain active runtime baseline references (`release_tag`, `commit_sha`, and `active_runtime_baseline`) pointing at current pinned `service.v2` release until cutover.
3. Consumer validation treats this as rehearsal metadata only, not runtime cutover authorization.

## Migration stages

## Stage 0: Preparation (current)
### Producer actions
- Keep `service.v2` stable for sprint window.
- Keep manifest + alignment gates green.

### Consumer actions
- Keep `error.code` as primary path.
- Keep `legacy_error` compatibility parsing.

### Exit criteria
- Joint checkpoint C sign-off complete.

## Stage 1: RFC finalization
### Producer actions
- Publish finalized `service.v3` RFC with:
  - exact schema diffs
  - migration window
  - rollback path
  - handoff payload template

### Consumer actions
- Review and approve migration criteria.

### Exit criteria
- Joint approval recorded.

## Stage 2: Pre-release artifacts (no cutover)
### Producer actions
- Prepare `service.v3` artifacts:
  - OpenAPI update
  - docs/spec updates
  - producer manifest update
  - changelog migration notes
- Keep `service.v2` release path available.

### Consumer actions
- Add/enable `service.v3` contract tests behind controlled pin updates.

### Exit criteria
- Both repos green on planned `service.v3` compatibility checks in pre-release branch contexts.
- AssistSupport confirms rehearsal payload ingestion and evidence production.

## Stage 3: Controlled cutover
### Producer actions
- Publish immutable `service.v3` release tag.
- Deliver full handoff payload and evidence.

### Consumer actions
- Repin and validate full CI on published `service.v3`.

### Exit criteria (hard gate)
- Consumer repin merged.
- Consumer CI green.
- Joint go decision recorded.
- Producer handoff package references explicit cutover gates and evidence checklist.

## Stage 4: Stabilization and cleanup
### Producer actions
- Monitor integration regressions for one sprint window.
- Keep rollback guidance explicit.

### Consumer actions
- Report runtime integration health and fallback behavior metrics.

### Exit criteria
- No unresolved P1/P2 integration issues for full stabilization window.

## Hard gate conditions before removing `legacy_error`
All must be true:
1. `service.v3` OpenAPI/spec/docs/manifests are published and consistent.
2. AssistSupport repin is merged to immutable `service.v3` tag/sha.
3. AssistSupport CI is green (`typecheck`, tests, contract tests, CI aggregate).
4. Producer verification suite is green (fmt/clippy/tests/alignment/parity/smoke/compliance).
5. Joint cutover acknowledgment is explicitly recorded.

## Producer handoff signal to start service.v3 consumer enforcement
MemoryKernel declares "rehearsal start ready" only when all are true:
1. `SERVICE_V3_REHEARSAL_HANDOFF_CANDIDATE.json` is published and committed.
2. `SERVICE_V3_CUTOVER_GATES.md` is committed with producer/consumer prerequisites and rollback triggers.
3. Full producer verification suite is green on the same commit.
4. AssistSupport confirms rehearsal-check acceptance criteria and command set.

## Consumer cutover checklist (must all pass)
1. Producer publishes immutable `service.v3` tag + commit with updated manifest/OpenAPI/spec.
2. AssistSupport updates pin + matrix + mirrored manifest atomically in one PR.
3. `pnpm run check:memorykernel-pin` passes with service.v3 expectations.
4. `pnpm run test:memorykernel-contract` passes with service.v3 non-2xx envelope assertions.
5. Deterministic fallback tests remain green for offline/timeout/malformed/version-mismatch/non-2xx.
6. `pnpm run test:ci` passes.
7. Rollback rehearsal verifies re-pin to last approved baseline with no Draft-flow regression.

## Rollback strategy
- If post-cutover issues emerge:
  1. Consumer repins to last stable `service.v2` tag.
  2. Producer publishes rollback advisory with exact tag/sha and impact summary.
  3. Re-open overlap window until blocking issue is resolved.

## Required release handoff fields (for service.v3)
- `release_tag`
- `commit_sha`
- `expected_service_contract_version`
- `expected_api_contract_version`
- `integration baseline`
- `consumer impact statement`
- verification evidence summary

## Gate-to-evidence mapping

| Gate | Producer evidence | Consumer evidence | Required command outputs |
|---|---|---|---|
| RFC finalization | Updated RFC + OpenAPI/spec notes | Consumer review notes and accepted criteria | N/A (document review gate) |
| Pre-release artifacts | Updated manifest + changelog + migration notes | Planned consumer repin PR checklist | Producer alignment/parity outputs |
| Cutover gate | Immutable release payload with v3 tag/sha | Atomic pin/matrix/manifest PR + rollback rehearsal notes | Consumer: `pnpm run check:memorykernel-pin`, `pnpm run test:memorykernel-contract`, `pnpm run test:ci` |
| Post-cutover stabilization | Producer incident monitoring logs | Consumer fallback health confirmation | Producer suite + consumer contract suite remain green |

## Verification checklist (producer)
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
./scripts/verify_producer_handoff_payload.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_producer_contract_manifest.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_contract_parity.sh --canonical-root /Users/d/Projects/MemoryKernel
./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_smoke.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_compliance_suite.sh --memorykernel-root /Users/d/Projects/MemoryKernel --skip-baseline
```

## Resolved planning decisions
1. Overlap rehearsal duration is locked to 14 calendar days (1 sprint).
2. Consumer manifest-hash validation is deferred for Checkpoint A/B and remains a Phase 3 automation hardening candidate.
3. Error-code enum validation mode is set equality (order-independent).
4. AssistSupport pin + matrix + mirrored manifest updates remain atomic in one PR.
