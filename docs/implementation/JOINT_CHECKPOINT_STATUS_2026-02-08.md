# Joint Checkpoint Status: AssistSupport + MemoryKernel

Updated: 2026-02-08 (cutover-decision checkpoint opened)

## Baseline
- MemoryKernel release tag: `v0.3.2`
- MemoryKernel commit: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- Service/API/baseline: `service.v2` / `api.v1` / `integration/v1`

## Checkpoint Results
- Checkpoint A (manifest mirrored + governance checks): `GO`
- Checkpoint B (consumer contract suite green): `GO`
- Checkpoint C (steady-state service.v2 window): `GO` (14-day window approved)
- Checkpoint D (service.v3 RFC kickoff): `GO` for planning only
- Rehearsal continuation decision: `GO`
- Runtime cutover decision: `NO-GO` (explicitly blocked pending joint gate completion)
- Bilateral sign-off status: `CLOSED` for rehearsal continuation + governance checkpoint

## Locked Decisions
1. `error_code_enum` validation is set equality (order-independent).
2. Non-2xx envelopes keep `api_contract_version` absent in service.v3 unless a future joint RFC changes this.
3. Producer-manifest hash validation is active in AssistSupport CI via pin + mirrored manifest SHA-256 integrity checks; authenticated remote validation runs when `MEMORYKERNEL_REPO_READ_TOKEN` is configured.
4. MemoryKernel governance bundle validation is active in AssistSupport CI (`pnpm run check:memorykernel-governance`).
5. Pin + matrix + mirrored producer manifest must be updated atomically in one PR.

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
- Release-cadence mismatch risk reducing overlap rehearsal time.
- Error taxonomy growth risk if lead-time notice policy slips.

## Next Execution Window
- Phase 1: Governed steady-state service.v2 operation (consumer closeout evidence captured).
- Phase 2: Consumer runtime hardening and diagnostics maturity (complete).
- Phase 3: Cross-repo automation (including manifest-hash validation gate, handoff payload validation, and consumer dry-run evidence).
- Phase 4: rehearsal readiness closed with candidate validation evidence.
- Phase 5: consumer cutover-prep controls enforced (`boundary` + `cutover-policy` checks active).
- Phase 6: cutover governance scaffold published (command checklist, rollback criteria, incident template).
- Consumer cutover-day dry-run execution evidence captured:
  - `/Users/d/Projects/AssistSupport/docs/implementation/SERVICE_V3_CUTOVER_DAY_DRY_RUN_EXECUTION_2026-02-08.md`
- Consumer sign-off checkpoint packet published:
  - `/Users/d/Projects/AssistSupport/docs/implementation/JOINT_SIGNOFF_CHECKPOINT_PACKET_2026-02-08.md`
- Producer sign-off checkpoint packet published:
  - `/Users/d/Projects/MemoryKernel/docs/implementation/JOINT_SIGNOFF_CHECKPOINT_PACKET_PRODUCER_2026-02-08.md`
- Closure evidence:
  - Producer sign-off commit: `208efc5b7006f4ac3f12dc0d9d57b8a0f3bbd85d`
- New cutover-decision checkpoint packet:
  - `/Users/d/Projects/AssistSupport/docs/implementation/SERVICE_V3_CUTOVER_DECISION_CHECKPOINT_2026-02-08.md`
- Next: keep runtime cutover blocked and use this checkpoint packet for any future cutover-decision session.
