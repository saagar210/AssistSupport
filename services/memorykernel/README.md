# MemoryKernel Trilogy Monorepo

This repository is the consolidated monorepo for the trilogy:

- `MemoryKernel`: typed memory storage and deterministic context resolution.
- `OutcomeMemory`: trust projection and gating.
- `MultiAgentCenter`: orchestration, execution traces, and replay.

MemoryKernel remains the canonical contract authority (`contracts/integration/v1/*`), and the monorepo now enforces cross-component parity and compatibility gates in one CI pipeline.

## What You Get

- Typed records: `constraint`, `decision`, `preference`, `event`, `outcome`.
- Append-only history with explicit lineage links (`supersedes`, `contradicts`).
- Provenance fields on writes (`source_uri`, optional `source_hash`).
- Separate truth semantics and confidence (`truth_status` is not `confidence`).
- Deterministic policy (`query ask`) and recall (`query recall`) retrieval.
- Explainable outputs with selected and excluded items plus reasons.
- Integrated `mk outcome ...` host commands for OutcomeMemory operations.

## Repository Layout

- `crates/memory-kernel-core`: domain model, validation, resolver, context package assembly.
- `crates/memory-kernel-store-sqlite`: schema, migrations, persistence, snapshot/export/restore.
- `crates/memory-kernel-cli`: `mk` CLI surface.
- `crates/memory-kernel-api`: stable local API wrapper.
- `crates/memory-kernel-service`: HTTP service and OpenAPI surface.
- `components/outcome-memory`: trust/gating subsystem.
- `components/multi-agent-center`: orchestration and trace/replay subsystem.
- `contracts/v1`: versioned CLI JSON schemas and fixtures.
- `contracts/integration/v1`: trilogy integration schemas and fixtures (canonical source).
- `docs/spec`: normative contracts and behavior definitions.
- `docs/implementation`: release gates, closeout reports, and phase records.
- `docs/compliance`: seven-standard compliance matrix and suite docs.

## Quick Start

### Prerequisites

- Rust stable toolchain (`rustup` + `cargo`)
- SQLite-compatible local filesystem
- Optional for hosted checks: GitHub CLI (`gh`) authenticated to your account

### 1) Initialize a local DB

```bash
cargo run -p memory-kernel-cli -- --db ./memory_kernel.sqlite3 db migrate
```

### 2) Add a policy record

```bash
cargo run -p memory-kernel-cli -- --db ./memory_kernel.sqlite3 memory add constraint \
  --actor user \
  --action use \
  --resource usb_drive \
  --effect deny \
  --writer local-dev \
  --justification "USB policy baseline" \
  --source-uri file:///policy/usb.md \
  --truth-status asserted \
  --authority authoritative \
  --confidence 0.95
```

### 3) Ask a policy question

```bash
cargo run -p memory-kernel-cli -- --db ./memory_kernel.sqlite3 query ask \
  --text "Am I allowed to use a USB drive?" \
  --actor user \
  --action use \
  --resource usb_drive
```

### 4) Run recall query

```bash
cargo run -p memory-kernel-cli -- --db ./memory_kernel.sqlite3 query recall \
  --text "usb policy history"
```

## CLI Command Map

Run help:

```bash
cargo run -p memory-kernel-cli -- --help
```

Top-level command groups:

- `db`: schema status, migration, export/import, backup/restore, integrity checks
- `memory`: add/link/list memory records
- `query`: policy ask and recall retrieval
- `context`: context package lookup
- `outcome`: OutcomeMemory host surface

## Service and API

- API crate: `crates/memory-kernel-api`
- Service binary: `crates/memory-kernel-service`
- OpenAPI: `openapi/openapi.yaml`

## Trilogy Integration

MemoryKernel is the canonical source for `contracts/integration/v1/*`.

- OutcomeMemory and MultiAgentCenter must parity-match that contract pack.
- Cross-project compatibility and locked release evidence are recorded in:
  - `docs/implementation/trilogy-compatibility-matrix.md`
  - `docs/implementation/trilogy-closeout-report-latest.md`

## Quality Gates

### Local quality gates

```bash
./scripts/run_trilogy_phase_8_11_closeout.sh --soak-iterations 1
./scripts/verify_contract_parity.sh
./scripts/verify_trilogy_compatibility_artifacts.sh
./scripts/run_trilogy_smoke.sh
./scripts/run_trilogy_soak.sh --iterations 3
./scripts/run_trilogy_compliance_suite.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
```

### Hosted workflows

- CI: `.github/workflows/ci.yml`
- Release: `.github/workflows/release.yml`
- Compliance suite: `docs/compliance/TRILOGY_COMPLIANCE_TEST_SUITE.md`

## Documentation Index

- Core requirements: `docs/spec/requirements.md`
- Domain model: `docs/spec/domain.md`
- Resolver behavior: `docs/spec/resolver.md`
- Context package contract: `docs/spec/context-package.md`
- CLI contract: `docs/spec/cli-contract.md`
- Integration contract: `docs/spec/integration-contract.md`
- Traceability: `docs/traceability-matrix.md`
- Test catalog: `docs/testing/test-catalog.md`
- Security: `docs/security/threat-model.md`, `docs/security/trust-controls.md`
- Operations: `docs/operations/migration-runbook.md`, `docs/operations/recovery-runbook.md`

## Current Release

- GitHub release: `v0.2.0`
- URL: `https://github.com/saagar210/MemoryKernel/releases/tag/v0.2.0`

## Troubleshooting

### Error: missing sibling repository path in CI/local builds

This monorepo expects OutcomeMemory and MultiAgentCenter under:

- `components/outcome-memory`
- `components/multi-agent-center`

All included scripts detect these paths automatically.
