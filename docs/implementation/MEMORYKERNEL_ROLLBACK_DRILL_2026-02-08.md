# MemoryKernel Rollback Drill

Updated: 2026-02-08

## Runtime Target
- release_tag: `v0.4.0`
- commit_sha: `7e4806a34b98e6c06ee33fa9f11499a975e7b922`
- service/api/baseline: `service.v3` / `api.v1` / `integration/v1`

## Rollback Target
- release_tag: `v0.3.2`
- commit_sha: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- service/api/baseline: `service.v2` / `api.v1` / `integration/v1`

## Commands Executed (Consumer)
```bash
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
pnpm run check:memorykernel-handoff
pnpm run test:memorykernel-contract
pnpm run test:memorykernel-cutover-dry-run
```

## Commands Executed (Producer)
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_producer_contract_manifest.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_producer_handoff_payload.sh --memorykernel-root /Users/d/Projects/MemoryKernel
```

## Drill Result
- Status: READY
- Result: PASS
- Notes: Consumer fallback and startup preflight behavior remain deterministic under runtime and rollback-target assumptions.

## Linked Evidence
- `/Users/d/Projects/AssistSupport/docs/implementation/RUNTIME_CUTOVER_DECISION_RECORD_2026-02-08.md`
- `/Users/d/Projects/MemoryKernel/docs/implementation/SERVICE_V3_ROLLBACK_READINESS_REFRESH_2026-02-08.md`
