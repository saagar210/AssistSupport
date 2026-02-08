# Joint Checkpoint Status: AssistSupport + MemoryKernel

Updated: 2026-02-08

## Baseline
- MemoryKernel release tag: `v0.3.1`
- MemoryKernel commit: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- Service/API/baseline: `service.v2` / `api.v1` / `integration/v1`

## Checkpoint Results
- Checkpoint A (manifest mirrored + governance checks): `GO`
- Checkpoint B (consumer contract suite green): `GO`
- Checkpoint C (steady-state service.v2 window): `GO` (14-day window approved)
- Checkpoint D (service.v3 RFC kickoff): `GO` for planning only

## Locked Decisions
1. `error_code_enum` validation is set equality (order-independent).
2. Non-2xx envelopes keep `api_contract_version` absent in service.v3 unless a future joint RFC changes this.
3. Producer-manifest hash validation in AssistSupport CI is deferred from A/B and must be in place by end of Phase 3.
4. Pin + matrix + mirrored producer manifest must be updated atomically in one PR.

## Service.v3 Cutover Gate (Consumer Criteria)
1. Producer publishes immutable service.v3 tag/sha with updated manifest/OpenAPI/spec.
2. AssistSupport updates pin+matrix+mirrored manifest in one PR.
3. `pnpm run check:memorykernel-pin` passes for service.v3 expectations.
4. `pnpm run test:memorykernel-contract` passes for service.v3 envelope semantics.
5. Deterministic fallback remains green for offline/timeout/malformed/version mismatch/non-2xx.
6. `pnpm run test:ci` passes.
7. Rollback rehearsal to prior approved baseline succeeds with no Draft-flow regression.

## Joint Risks to Track
- Dual-compat drift risk during v2/v3 overlap.
- Manifest integrity risk until hash-validation gate is activated in Phase 3.
- Release-cadence mismatch risk reducing overlap rehearsal time.
- Error taxonomy growth risk if lead-time notice policy slips.

## Next Execution Window
- Phase 1: Governed steady-state service.v2 operation and weekly checkpoints.
- Phase 2: Consumer runtime hardening and diagnostics maturity.
- Phase 3: Cross-repo automation (including manifest-hash validation gate in AssistSupport CI).
- Phase 4: service.v3 migration readiness and pre-cutover rehearsal.
