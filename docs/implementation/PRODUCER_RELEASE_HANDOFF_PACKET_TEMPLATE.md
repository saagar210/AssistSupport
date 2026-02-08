# Producer Release Handoff Packet Template

## Purpose
Use this template to produce a deterministic handoff packet for AssistSupport with zero ambiguity.

## Required metadata
- `release_tag`
- `commit_sha`
- `expected_service_contract_version`
- `expected_api_contract_version`
- `integration_baseline`
- `manifest_contract_version`
- `manifest_path`
- `changelog_path`
- `handoff_mode` (`stable` or `service-v3-candidate`)
- `active_runtime_baseline` (always reflects currently pinned runtime release)

## Required evidence
- Full producer verification command set and outcomes:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo test --workspace --all-targets --all-features`
  - `./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel`
  - `./scripts/verify_contract_parity.sh --canonical-root /Users/d/Projects/MemoryKernel`
  - `./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root /Users/d/Projects/MemoryKernel`
  - `./scripts/run_trilogy_smoke.sh --memorykernel-root /Users/d/Projects/MemoryKernel`
  - `./scripts/run_trilogy_compliance_suite.sh --memorykernel-root /Users/d/Projects/MemoryKernel --skip-baseline`

## Producer impact statement (required)
```text
Consumer impact:
- Runtime behavior changes: <none|describe>
- Required consumer mapping changes: <none|describe>
- Required test updates: <none|describe>
- Repin required: <yes/no> (target tag/sha)
- Rollback instruction: repin to <previous tag/sha>
```

## Generation commands
Stable release handoff:
```bash
./scripts/generate_producer_handoff_payload.sh \
  --mode stable \
  --memorykernel-root /Users/d/Projects/MemoryKernel \
  --out-json /Users/d/Projects/MemoryKernel/docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json
```

Service.v3 rehearsal candidate handoff (no runtime cutover):
```bash
./scripts/generate_producer_handoff_payload.sh \
  --mode service-v3-candidate \
  --memorykernel-root /Users/d/Projects/MemoryKernel \
  --out-json /Users/d/Projects/MemoryKernel/docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json
```

## Output artifact
- `/Users/d/Projects/MemoryKernel/docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json`
