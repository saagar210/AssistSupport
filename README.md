# Memory Kernel

Personal AI Memory Kernel with typed, append-only, policy-controlled memories and deterministic Context Packages.

Current retrieval surfaces include:
- policy query (`query ask`) for constraint resolution
- recall query (`query recall`) for deterministic mixed-record retrieval

Host command surface also includes OutcomeMemory operations under `mk outcome ...` for
append-only outcome event logging, replay, projector health, and trust gate preview flows.

## Workspace Crates

- `crates/memory-kernel-core`: domain model, validation, resolver, Context Package builder.
- `crates/memory-kernel-store-sqlite`: SQLite schema, migrations, persistence APIs.
- `crates/memory-kernel-cli`: CLI contract for explicit writes, links, query, and context retrieval.
- `crates/memory-kernel-api`: stable programmatic API surface over core/store operations.
- `crates/memory-kernel-service`: local HTTP service exposing versioned integration endpoints.

## Normative Specs

- `docs/spec/requirements.md`
- `docs/spec/domain.md`
- `docs/spec/resolver.md`
- `docs/spec/context-package.md`
- `docs/spec/cli-contract.md`
- `docs/spec/migrations.md`
- `docs/spec/versioning.md`
- `docs/spec/service-contract.md`
- `docs/spec/integration-contract.md`

## Governance

- ADRs: `docs/adr/`
- Traceability: `docs/traceability-matrix.md`
- Tests catalog: `docs/testing/test-catalog.md`
- Performance budgets: `docs/testing/performance-budgets.md`
- Security docs: `docs/security/threat-model.md`, `docs/security/trust-controls.md`
- Operations runbooks: `docs/operations/migration-runbook.md`, `docs/operations/recovery-runbook.md`
- Phase checklists: `docs/implementation/`
- Contracts: `contracts/v1/`

## Quality Gates

- CI workflow: `.github/workflows/ci.yml`
- Local verification commands:
  - `./scripts/run_trilogy_phase_8_11_closeout.sh --soak-iterations 1`
  - `./scripts/verify_contract_parity.sh`
  - `./scripts/verify_trilogy_compatibility_artifacts.sh`
  - `./scripts/run_trilogy_smoke.sh`
  - `./scripts/run_trilogy_soak.sh --iterations 3`
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo test --workspace --all-targets --all-features`
