# Producer Consolidation Rollback Checklist

Updated: 2026-02-08
Owner: MemoryKernel

## Trigger Conditions

- Manifest or handoff validation fails after promotion.
- Service envelope behavior diverges from expected contract.
- Consumer reports deterministic fallback regression linked to producer change.

## Rollback Steps

1. Revert producer baseline to last-known-good tag and commit.
2. Regenerate handoff payload from reverted baseline.
3. Re-run mandatory producer gates:
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - `cargo test --workspace --all-targets --all-features`
   - `./scripts/verify_producer_contract_manifest.sh --memorykernel-root <root>`
   - `./scripts/verify_producer_handoff_payload.sh --memorykernel-root <root>`
4. Publish rollback communication note and updated decision status addendum.

## Success Criteria

- Producer gates are green on reverted baseline.
- Consumer handoff validation passes on reverted payload.
- Runtime cutover posture remains explicitly NO-GO until bilateral confirmation.
