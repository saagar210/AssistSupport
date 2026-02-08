# Trilogy Release Gate

This checklist is the pre-release gate for synchronized MemoryKernel + OutcomeMemory + MultiAgentCenter rollout.

## Required Inputs

- All three workspaces available locally under a shared parent directory.
- MemoryKernel is the canonical source for `contracts/integration/v1/*`.
- OutcomeMemory and MultiAgentCenter parity updates merged before running final gate.

## Gate Commands (MemoryKernel Root)

```bash
./scripts/run_trilogy_phase_8_11_closeout.sh --soak-iterations 1
./scripts/verify_contract_parity.sh
./scripts/verify_trilogy_compatibility_artifacts.sh
./scripts/run_trilogy_smoke.sh
./scripts/run_trilogy_soak.sh --iterations 3
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo run -p memory-kernel-cli -- outcome benchmark run --volume 100 --volume 500 --volume 2000 --repetitions 3 --append-p95-max-ms 8 --replay-p95-max-ms 250 --gate-p95-max-ms 8 --json
```

## Pass Criteria

- Contract parity checker exits 0 with no file-set or content mismatches.
- Trilogy smoke script exits 0 with deterministic policy/recall checks and Outcome host checks.
- Rust quality gates pass with zero warnings (`clippy -D warnings`) and no test failures.
- Benchmark command exits 0 with threshold triplet present and no violations.

## Failure Policy

- Any contract parity mismatch blocks release.
- Any deterministic mismatch in smoke checks blocks release.
- Any benchmark threshold violation blocks release.
- Any attempt to change integration contract semantics without version bump blocks release.
