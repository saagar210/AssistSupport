# Service.v3 Cutover Day Checklist (Producer)

Updated: 2026-02-08  
Scope: Governance-only checklist; does not authorize autonomous runtime cutover.

## Preconditions (must be true before cutover window starts)
1. Producer release candidate artifacts are complete and immutable.
2. AssistSupport confirms candidate validation is green.
3. Joint go/no-go meeting is scheduled with named incident channel.
4. Rollback target (`service.v2` baseline) is confirmed and documented.

## Producer Cutover Checklist
1. Verify all producer gates are green on target commit:
   - format, clippy, tests
   - service alignment
   - parity + compatibility artifacts
   - smoke + compliance suite
2. Regenerate and publish producer handoff payload:
   - stable mode release handoff
   - service.v3 candidate/release handoff (as applicable)
3. Confirm non-2xx envelope policy and error code enum are unchanged from approved RFC for the target release.
4. Publish release impact statement (consumer-facing).
5. Capture all evidence artifacts in release notes.

## Consumer Readiness Confirmation (required input from AssistSupport)
1. `pnpm run check:memorykernel-handoff:service-v3-candidate` passes.
2. `pnpm run check:memorykernel-pin` passes.
3. `pnpm run test:memorykernel-contract` passes.
4. `pnpm run test:ci` passes.
5. Deterministic fallback remains non-blocking.

## Go/No-Go Decision Record
Record:
1. Decision timestamp (UTC)
2. Producer approver
3. Consumer approver
4. Evidence links
5. Risks accepted
6. Rollback target tag/sha

## Rollback Triggers (immediate)
1. Envelope shape mismatch against approved service.v3 contract.
2. Consumer regression in deterministic fallback path.
3. Critical incident with unresolved mitigation inside cutover window.

## Rollback Evidence Requirements
1. Incident summary with trigger classification.
2. Exact rollback command/evidence and resulting deployed baseline.
3. Post-rollback verification command outputs.
4. Consumer confirmation after re-pin.
