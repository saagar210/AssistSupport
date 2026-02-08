# OutcomeMemory

OutcomeMemory is the trust and outcome projection layer in the trilogy:

- `MemoryKernel`: authoritative memory and deterministic context resolution
- `OutcomeMemory`: trust projection, replay, and gate preview
- `MultiAgentCenter`: orchestration and audit replay

This project is append-only, replay-driven, and contract-frozen for `v1`.

## What It Does

- Stores outcome events for memory records without mutating historical event rows.
- Replays events into trust state deterministically.
- Exposes trust and projection health via CLI (`projector status`, `projector check`).
- Supports gate preview decisions (`safe` or `exploration` mode).
- Provides benchmark guardrails with threshold-based non-zero exits.

## Key Guarantees

- Append-only outcome log semantics.
- Deterministic replay behavior.
- Version-isolated trust (`memory_id + version` boundaries).
- Stable host-embed API:
  - `run_cli`
  - `run_outcome_with_db`
  - `run_outcome`
  - `run_benchmark`

## Repository Layout

- `crates/memory-kernel-outcome-core`: trust/gate domain logic.
- `crates/memory-kernel-outcome-store-sqlite`: persistence + replay + benchmarks.
- `crates/memory-kernel-outcome-cli`: `mk` CLI integration and contracts.
- `contracts/integration/v1`: shared trilogy integration schemas/fixtures.
- `docs/v1-contract-freeze.md`: frozen contract baseline.
- `docs/performance.md`: benchmark policy and threshold semantics.
- `docs/integration-handoff.md`: host integration requirements.
- `trilogy-compatibility.v1.json`: machine-readable compatibility assumptions.

## Prerequisites

- Rust stable toolchain
- SQLite-compatible local filesystem
- Monorepo root checked out with:
  - `crates/memory-kernel-*`
  - `components/outcome-memory`

Workspace dependencies resolve via monorepo-local paths.

## Quick Start

### 1) Show command surface

```bash
cargo run -p memory-kernel-outcome-cli -- --help
cargo run -p memory-kernel-outcome-cli -- outcome --help
```

### 2) Run benchmark report

```bash
cargo run -p memory-kernel-outcome-cli -- \
  --db /tmp/outcome.sqlite3 \
  outcome benchmark run \
  --volume 100 --volume 500 --volume 2000 \
  --repetitions 3 \
  --append-p95-max-ms 8 \
  --replay-p95-max-ms 250 \
  --gate-p95-max-ms 8 \
  --json
```

### 3) Check projector health

```bash
cargo run -p memory-kernel-outcome-cli -- \
  --db /tmp/outcome.sqlite3 \
  outcome projector status --json
```

## Contract and Compatibility

- Frozen output versions:
  - `gate_preview.v1`
  - `projector_status.v1`
  - `projector_check.v1`
  - `benchmark_report.v1`
- Canonical trilogy contract pack parity:
  - `scripts/check_contract_pack_parity.sh v1`
- Compatibility artifact validation:
  - `scripts/check_trilogy_compatibility_artifact.sh trilogy-compatibility.v1.json`

## Quality Gates

Run from repo root:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
./scripts/check_contract_pack_parity.sh v1
./scripts/check_trilogy_compatibility_artifact.sh trilogy-compatibility.v1.json
```

Hosted workflows:

- Smoke: `.github/workflows/smoke.yml`
- Performance: `.github/workflows/perf.yml`

Hosted canonical contract variable:

- `MEMORYKERNEL_CANONICAL_REPO` (set to `saagar210/MemoryKernel` for deterministic parity resolution)

## Current Release

- GitHub release: `v0.1.0`
- URL: `https://github.com/saagar210/OutcomeMemory/releases/tag/v0.1.0`

## Integration Notes

When embedding in host projects, only use stable API entrypoints in:

- `crates/memory-kernel-outcome-cli/src/lib.rs`

Do not call private helper functions directly.

## Troubleshooting

### Error: failed to read `../../crates/memory-kernel-.../Cargo.toml`

Cause:
- Incomplete monorepo checkout or wrong working directory.

Fix:
- Run commands from the monorepo root or from `components/outcome-memory` inside this repository.

### CI parity fails with missing canonical pack path

Cause:
- Missing `MEMORYKERNEL_CANONICAL_REPO` and no sibling fallback path.

Fix:
- Set repository variable:
  - `MEMORYKERNEL_CANONICAL_REPO=saagar210/MemoryKernel`
