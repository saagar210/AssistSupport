# Next Execution Queue (Producer)

Updated: 2026-02-08
Owner: MemoryKernel

## Section 1: Producer Observability Deepening
- [ ] Add service-level timeout/failure metrics artifact in docs and CLI diagnostics output.
- [ ] Define explicit SLO thresholds for timeout/error classes and publish alert trigger guidance.
- [ ] Add regression tests for degraded dependency behavior per endpoint.

Exit Criteria:
- Metrics + SLO docs are published.
- Regression tests cover timeout, malformed payload, and schema-unavailable classes.

## Section 2: Contract and Handoff Automation Hardening
- [ ] Add negative-fixture checks for handoff payload and manifest drift.
- [ ] Add explicit producer evidence generation command for release packets.
- [ ] Ensure CI fails when handoff payload policy section drifts from service contract docs.

Exit Criteria:
- Negative tests pass and fail as expected.
- Release packet generation is deterministic.

## Section 3: Monorepo Consolidation Readiness (Producer Scope)
- [ ] Publish producer module-boundary mapping for consolidated layout.
- [ ] Publish migration-safe ownership matrix for MemoryKernel crates/components.
- [ ] Produce rollback checklist specific to producer modules post-consolidation.

Exit Criteria:
- Module boundary map and ownership matrix are approved and executable.
- Rollback checklist is validated in dry run.

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
