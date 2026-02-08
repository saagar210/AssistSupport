# Service.v3 Consumer Rehearsal Plan (AssistSupport)

Updated: 2026-02-08
Owner: AssistSupport

## Scope
Planning-only rehearsal for service.v3 adoption. No runtime cutover in this phase.

## Rehearsal Window
- Duration: 14 calendar days (1 sprint)
- Goal: validate consumer readiness and rollback confidence before runtime transition.

## Preconditions
1. Producer publishes immutable service.v3 candidate tag/sha and manifest.
2. Producer OpenAPI/spec/docs are aligned for service.v3 envelope behavior.
3. AssistSupport branch prepared for atomic pin+matrix+manifest update.

## Rehearsal Checklist
1. Update pin + matrix + mirrored producer manifest in one PR.
2. Validate producer service.v3 candidate handoff payload in rehearsal mode.
   - `pnpm run check:memorykernel-handoff:service-v3-candidate`
   - no runtime cutover and no pin update in this step
3. Ensure non-2xx policy assertions remain true:
   - include `service_contract_version`
   - include `error.code` and `error.message`
   - exclude `api_contract_version`
   - `legacy_error` behavior aligned to service.v3 policy
4. Validate deterministic fallback for offline/timeout/malformed/version-mismatch/non-2xx.
5. Run full consumer verification suite.
6. Execute rollback rehearsal to prior approved baseline and verify no Draft-flow regression.

## Required Commands
```bash
pnpm run typecheck
pnpm run test
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
pnpm run check:memorykernel-handoff:service-v3-candidate
pnpm run test:memorykernel-contract
pnpm run test:ci
```

## Exit Criteria
1. All required commands pass.
2. Runtime diagnostics remain actionable for each failure class.
3. Rollback rehearsal passes.
4. Joint go/no-go decision recorded with MemoryKernel.
