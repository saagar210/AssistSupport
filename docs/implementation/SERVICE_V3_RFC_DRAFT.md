# Service.v3 RFC Draft (Producer Planning)

## Status
- Draft
- Scope: planning/governance only (no runtime cutover in this draft)

## Objective
Define a safe, testable transition from `service.v2` to `service.v3` where
`legacy_error` can be removed from non-2xx envelopes without breaking AssistSupport.

## Current baseline
- Producer release baseline: `v0.3.1`
- Producer commit baseline: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- Active contracts:
  - `service.v2`
  - `api.v1`
  - `integration/v1`

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

## Stage 3: Controlled cutover
### Producer actions
- Publish immutable `service.v3` release tag.
- Deliver full handoff payload and evidence.

### Consumer actions
- Repin and validate full CI on published `service.v3`.

### Exit criteria (hard gate)
- Consumer repin merged.
- Consumer CI green.
- Consumer manifest-hash validation gate enabled (Phase 3 automation prerequisite).
- Joint go decision recorded.

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
5. AssistSupport manifest-hash validation is enabled in CI.
6. Joint cutover acknowledgment is explicitly recorded.

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
./scripts/verify_producer_contract_manifest.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_contract_parity.sh --canonical-root /Users/d/Projects/MemoryKernel
./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_smoke.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_compliance_suite.sh --memorykernel-root /Users/d/Projects/MemoryKernel --skip-baseline
```

## Resolved planning decisions
1. Overlap rehearsal duration is locked to 14 calendar days (1 sprint).
2. Consumer manifest-hash validation is active for local mirror integrity and can run authenticated remote validation when `MEMORYKERNEL_REPO_READ_TOKEN` is configured.
