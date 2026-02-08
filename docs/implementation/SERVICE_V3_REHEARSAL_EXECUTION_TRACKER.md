# Service.v3 Rehearsal Execution Tracker (MemoryKernel)

Updated: 2026-02-08
Owner: MemoryKernel

## Working Branch Plan
- Primary branch: `codex/service-v3-rehearsal-producer`
- Backup branch: `codex/service-v3-rehearsal-fixes`

## Phase Execution Tasks

### Task P1: Rehearsal branch setup
- Status: Ready (checkpoint D planning approved)
- Owner: MemoryKernel
- Definition of done:
  - branch created from `main`
  - rehearsal artifact checklist added

### Task P2: service.v3 artifact preparation
- Status: In Progress (planning artifacts drafted, runtime changes not started)
- Owner: MemoryKernel
- Definition of done:
  - OpenAPI/spec/docs aligned
  - manifest updated for candidate

### Task P3: Producer verification suite
- Status: Ready (verification command set locked)
- Owner: MemoryKernel
- Definition of done:
  - full producer suite green
  - handoff packet prepared

### Task P4: Joint rehearsal handoff
- Status: Pending (awaiting AssistSupport rehearsal-entry confirmation)
- Owner: Joint
- Definition of done:
  - consumer rehearsal starts from immutable candidate tag/sha
  - cutover gate checklist signed

## Required Commands
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
./scripts/verify_producer_contract_manifest.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_contract_parity.sh --canonical-root /Users/d/Projects/MemoryKernel
./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_smoke.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_compliance_suite.sh --memorykernel-root /Users/d/Projects/MemoryKernel --skip-baseline
```

## Exit Criteria
1. All tasks complete with evidence references.
2. Candidate release handoff published.
3. Joint go/no-go decision recorded.
