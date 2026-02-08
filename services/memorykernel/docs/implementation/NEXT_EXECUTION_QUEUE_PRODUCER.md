# Next Execution Queue (Producer)

Updated: 2026-02-08
Owner: MemoryKernel

## Section 1: Producer Observability Deepening
- [x] Add service-level timeout/failure metrics artifact in docs and CLI diagnostics output.
- [x] Define explicit SLO thresholds for timeout/error classes and publish alert trigger guidance.
- [x] Add regression tests for degraded dependency behavior per endpoint.

Exit Criteria:
- Metrics + SLO docs are published.
- Regression tests cover timeout, malformed payload, and schema-unavailable classes.

Evidence:
- `/Users/d/Projects/MemoryKernel/docs/spec/service-slo-policy.md`
- `/Users/d/Projects/MemoryKernel/contracts/integration/v1/service-slo-policy.json`
- `/Users/d/Projects/MemoryKernel/crates/memory-kernel-service/src/main.rs`

## Section 2: Contract and Handoff Automation Hardening
- [x] Add negative-fixture checks for handoff payload and manifest drift.
- [x] Add explicit producer evidence generation command for release packets.
- [x] Ensure CI fails when handoff payload policy section drifts from service contract docs.

Exit Criteria:
- Negative tests pass and fail as expected.
- Release packet generation is deterministic.

Evidence:
- `/Users/d/Projects/MemoryKernel/scripts/test_producer_governance_negative.sh`
- `/Users/d/Projects/MemoryKernel/scripts/generate_release_evidence_bundle.sh`
- `/Users/d/Projects/MemoryKernel/scripts/verify_release_evidence_bundle.sh`

## Section 3: Monorepo Consolidation Readiness (Producer Scope)
- [x] Publish producer module-boundary mapping for consolidated layout.
- [x] Publish migration-safe ownership matrix for MemoryKernel crates/components.
- [x] Produce rollback checklist specific to producer modules post-consolidation.

Exit Criteria:
- Module boundary map and ownership matrix are approved and executable.
- Rollback checklist is validated in dry run.

Evidence:
- `/Users/d/Projects/MemoryKernel/docs/implementation/PRODUCER_MODULE_BOUNDARY_MAP.md`
- `/Users/d/Projects/MemoryKernel/docs/implementation/PRODUCER_OWNERSHIP_MATRIX.md`
- `/Users/d/Projects/MemoryKernel/docs/implementation/PRODUCER_CONSOLIDATION_ROLLBACK_CHECKLIST.md`

## Mandatory Verification Set
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_contract_parity.sh --canonical-root /Users/d/Projects/MemoryKernel
./scripts/verify_producer_contract_manifest.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_producer_handoff_payload.sh --memorykernel-root /Users/d/Projects/MemoryKernel
```
