# Monorepo Migration Gate Checklist

Updated: 2026-02-08
Owner: AssistSupport + MemoryKernel Joint

This checklist is pass/fail only. Any unchecked line is an automatic NO-GO.

## Gate A: Source-of-Truth Alignment

- [ ] Pin manifest present and valid JSON.
- [ ] Producer-manifest mirror present and hash-aligned.
- [ ] Compatibility matrix reflects exact tag/commit/service/api versions.
- [ ] Active runtime status doc reflects same baseline.

Validation:

```bash
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
```

## Gate B: Boundary and Runtime Safety

- [ ] No direct MemoryKernel calls outside adapter boundary.
- [ ] Enrichment remains optional/non-blocking.
- [ ] Deterministic fallback behavior remains active.
- [ ] Cutover policy guard still blocks unauthorized runtime switch.

Validation:

```bash
pnpm run check:memorykernel-boundary
pnpm run check:memorykernel-cutover-policy
pnpm run test:memorykernel-contract
```

## Gate C: Producer Handoff Integrity

- [ ] Stable handoff payload validates against pinned baseline.
- [ ] Candidate payload validates with expected candidate policy.
- [ ] Error-code enum equality remains green (set-equality consumer policy).

Validation:

```bash
pnpm run check:memorykernel-handoff
pnpm run check:memorykernel-handoff:service-v3-candidate
```

## Gate D: Negative Drift Proof

- [ ] Pin/matrix/manifest drift is detected and fails.
- [ ] Governance doc/runtime-state drift is detected and fails.
- [ ] Producer manifest/handoff drift is detected and fails.

Validation:

```bash
pnpm run test:memorykernel-governance-negative
cd services/memorykernel && ./scripts/test_producer_governance_negative.sh --memorykernel-root "$(pwd)"
```

## Gate E: Full Program Verification

- [ ] Frontend typecheck and tests green.
- [ ] Consumer CI suite green.
- [ ] Embedded MemoryKernel suite green.
- [ ] Compliance suite green (skip-baseline mode allowed for local fast validation).

Validation:

```bash
pnpm run typecheck
pnpm run test
pnpm run test:ci
cd services/memorykernel
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
./scripts/run_trilogy_compliance_suite.sh --memorykernel-root "$(pwd)" --skip-baseline
```

## Gate F: Rollback Readiness

- [ ] Last-known-good baseline recorded.
- [ ] Rollback commands validated in dry-run evidence.
- [ ] Incident communication template updated and approved.

Evidence references:

- `docs/implementation/MEMORYKERNEL_ROLLBACK_DRILL_2026-02-08.md`
- `docs/implementation/RUNTIME_CUTOVER_GOVERNANCE_PACKET.md`
- `docs/implementation/MONOREPO_OPERATOR_HANDOFF_RUNBOOK.md`
