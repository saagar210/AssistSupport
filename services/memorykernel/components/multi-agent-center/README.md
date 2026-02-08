# MultiAgentCenter

MultiAgentCenter is the orchestration and traceability layer in the trilogy:

- `MemoryKernel`: authoritative memory retrieval and context packages
- `OutcomeMemory`: trust projection and gating
- `MultiAgentCenter`: workflow execution, policy/trust gate decisions, and replayable audit traces

This project focuses on deterministic execution, reproducible replay, and strict memory identity handling.

## What It Does

- Executes multi-step workflows from YAML.
- Pulls context from MemoryKernel using API-backed policy and recall queries.
- Applies optional trust gating from OutcomeMemory.
- Persists append-only run and step traces in SQLite.
- Supports replay and rerun from persisted snapshots for auditability.

## Core Guarantees

- Deterministic workflow normalization + manifest hashing.
- `context_queries[].mode` supports `policy` (default) and `recall`.
- Recall defaults are stable when `record_types` is missing/empty.
- Invalid recall record types fail fast with explicit validation errors.
- Trust persistence for `memory_ref` requires all:
  - `memory_id`
  - `version`
  - `memory_version_id`

## Repository Layout

- `crates/multi-agent-center-cli`: CLI entrypoint.
- `crates/multi-agent-center-orchestrator`: run engine and context/gate flow.
- `crates/multi-agent-center-trace-sqlite`: trace persistence and constraints.
- `crates/multi-agent-center-domain`: domain models and compatibility structures.
- `contracts/integration/v1`: shared trilogy schemas/fixtures.
- `docs/architecture.md`: architecture and invariants.
- `docs/trace-schema.md`: trace schema contract.
- `docs/trilogy-compatibility.md`: cross-project compatibility assumptions.
- `trilogy-compatibility.v1.json`: machine-readable compatibility artifact.

## Prerequisites

- Rust stable toolchain
- Monorepo root checked out with:
  - `crates/memory-kernel-*`
  - `components/outcome-memory`
  - `components/multi-agent-center`

## Quick Start

### 1) Run a mock workflow

```bash
cargo run -p multi-agent-center-cli -- \
  run \
  --workflow examples/workflow.mock.yaml \
  --trace-db /tmp/multi-agent-center.trace.sqlite \
  --non-interactive
```

### 2) Run a memory-backed workflow

```bash
cargo run -p multi-agent-center-cli -- \
  run \
  --workflow examples/workflow.memory.yaml \
  --trace-db /tmp/multi-agent-center.trace.sqlite \
  --memory-db /tmp/memory-kernel.sqlite3 \
  --non-interactive
```

### 3) Replay a run

```bash
cargo run -p multi-agent-center-cli -- \
  replay \
  --trace-db /tmp/multi-agent-center.trace.sqlite \
  --run-id <RUN_ID>
```

### 4) Inspect trace records

```bash
cargo run -p multi-agent-center-cli -- trace runs --trace-db /tmp/multi-agent-center.trace.sqlite
cargo run -p multi-agent-center-cli -- trace events --trace-db /tmp/multi-agent-center.trace.sqlite --run-id <RUN_ID>
```

## CLI Surface

Top-level commands:

- `run`
- `trace`
- `replay`
- `export`

Get help:

```bash
cargo run -p multi-agent-center-cli -- --help
```

## Trilogy Integration Notes

- Contract pack must stay parity-identical with `MemoryKernel/contracts/integration/v1/*`.
- `--memory-db` uses MemoryKernel API-backed context retrieval (not direct table scraping).
- `--trust-db` enables optional OutcomeMemory trust gating.

## Quality Gates

Run from repo root:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test -p multi-agent-center-cli --test integration_contracts
```

Hosted workflow:

- Trilogy guard: `.github/workflows/trilogy-guard.yml`

## Current Release

- GitHub release: `v0.1.0`
- URL: `https://github.com/saagar210/MultiAgentCenter/releases/tag/v0.1.0`

## Troubleshooting

### Error: failed to read `../../crates/memory-kernel-.../Cargo.toml` or `../outcome-memory/.../Cargo.toml`

Cause:
- Incomplete monorepo checkout or wrong working directory.

Fix:
- Run commands from the monorepo root or from `components/multi-agent-center` inside this repository.

### Workflow fails due recall record types

Cause:
- Invalid `task.context_queries[].record_types` value in workflow YAML.

Fix:
- Use valid recall types only:
  - `decision`, `preference`, `event`, `outcome`
