# MemoryKernel Integration Brainstorm (AssistSupport)

Date: 2026-02-08

## Purpose

Define how AssistSupport should integrate with MemoryKernel in a way that is reliable during active development, safe for production-intended use, and easy to evolve without regressions.

## Current Snapshot

- AssistSupport repo: `saagar210/AssistSupport`, `master` at `a1cd0c3`.
- MemoryKernel repo: `saagar210/MemoryKernel`, `main` at `b62fa63`.
- MemoryKernel is actively changing; local working tree activity is expected.
- MemoryKernel currently exposes both:
  - local Rust API crate (`memory-kernel-api`)
  - local HTTP service (`memory-kernel-service`, default `127.0.0.1:4010`, `service.v2`)

## Why Integration Drift Is the Main Risk

Integration drift is when AssistSupport and MemoryKernel both keep moving, but assumptions between them no longer match.

### Drift Types

1. Contract drift
- Endpoint/field/type changes between versions.
- Example: request/response shape changes in `/v1/query/ask`.

2. Semantic drift
- Same fields, different meaning.
- Example: confidence or authority interpretation changes while schema stays valid.

3. Operational drift
- Runtime expectations diverge.
- Example: port, startup order, migration behavior, timeout assumptions.

4. Release cadence drift
- MemoryKernel ships faster than AssistSupport adoption cadence.
- AssistSupport accidentally pulls unvalidated changes.

5. Evidence/compliance drift
- “Suite passes” in producer repo but no proof consumer behavior still matches expected controls.

## AssistSupport-Owned Controls to Resolve Drift

These controls should be implemented from AssistSupport side, independent of producer pace.

1. Single integration boundary
- Create one AssistSupport adapter boundary for MemoryKernel calls.
- No direct calls to MemoryKernel APIs outside this boundary.

2. Version pinning policy
- Pin to a known MemoryKernel commit/tag (not floating `main`).
- Record pinned producer contract versions in AssistSupport docs/config.

3. Version handshake gate
- On startup, call MemoryKernel health/version metadata and verify expected `service_contract_version` and `api_contract_version`.
- If mismatched: disable MemoryKernel features, keep core AssistSupport available, show actionable UI warning.

4. Consumer contract tests
- In AssistSupport CI, run contract tests against the pinned MemoryKernel version.
- Validate required fields, required semantics, and deterministic fallback behavior.

5. Compatibility matrix in AssistSupport
- Track approved pairs, for example:
  - AssistSupport `x.y.z` ↔ MemoryKernel commit/tag + contract version.
- Treat non-listed pairs as unsupported by default.

6. Controlled upgrade workflow
- Upgrade MemoryKernel pin only via explicit PR that includes:
  - contract diff summary
  - updated compatibility matrix
  - passing consumer contract + smoke tests

7. Feature flag + fallback path
- Keep AssistSupport usable when MemoryKernel is unavailable.
- Use explicit feature flag for MemoryKernel-powered paths until integration stabilizes.

## Integration Architecture Options

## Option A: Service Integration (HTTP, recommended initial)

AssistSupport (Tauri backend) calls `memory-kernel-service` over localhost.

### Pros

- Clean process isolation and failure containment.
- Strong version handshake via service envelope (`service.v2`, `api.v1`).
- Easier local debugging and integration testing.
- No tight Rust crate coupling across repos.

### Cons

- Requires lifecycle management for companion service.
- Adds local networking, timeout, and retry concerns.

### Best Use

- First production integration milestone.

## Option B: Embedded Crate Integration (Rust link)

AssistSupport links MemoryKernel crates directly into Tauri backend.

### Pros

- No sidecar process and lower IPC overhead.
- Simpler deployment footprint once stable.

### Cons

- Higher compile-time coupling and upgrade friction.
- Workspace/dependency conflicts more likely during active MemoryKernel development.
- Harder blast-radius isolation when producer changes.

### Best Use

- Later stage after contract and release stability is proven.

## Option C: Hybrid (Service-first, Embed-later)

Start with service boundary; optionally embed selective APIs later if operationally justified.

### Pros

- Fast path to value now, with future optimization path.
- Preserves decoupling while both codebases mature.

### Cons

- Requires discipline to avoid maintaining two full paths too early.

### Best Use

- Current situation (active producer development + near-term integration need).

## Recommended Path

Adopt Option C with an explicit phase gate:

- Phase 1-2: Service-first integration only.
- Phase 3+: Evaluate embedding only if measurable need exists (latency, packaging simplicity, or reliability gains).

## Target Integration Scope (Thin Slice First)

Start with one high-value deterministic path:

1. Draft generation asks MemoryKernel policy query for ticket context (`query ask`).
2. AssistSupport uses returned context package as a structured grounding source.
3. If MemoryKernel unavailable or version-mismatched:
- continue with existing AssistSupport flow
- mark output as “MemoryKernel enrichment unavailable”

This gives immediate value without risking total feature coupling.

## Proposed AssistSupport Contracts (Consumer Expectations)

AssistSupport should treat these as hard requirements for v1 integration:

1. Health endpoint reachable and reports expected contract versions.
2. `query ask` response includes deterministic identifiers and explainable package payload.
3. Errors return stable machine-readable format for UI-safe mapping.
4. Timeout behavior bounded (for example 2-5s budget with fallback).
5. No requirement for network egress beyond localhost.

## Security and Reliability Guardrails

1. Localhost-only communication.
2. Strict input validation for all MemoryKernel request payloads before dispatch.
3. Redacted logging around sensitive request content.
4. Circuit breaker after repeated service failures.
5. Startup preflight checks with actionable diagnostics in Settings/Health UI.

## Delivery Phases

## Phase 0: Alignment and Contracts

- Lock initial compatible MemoryKernel commit/tag.
- Define AssistSupport consumer contract file and compatibility matrix.
- Define failure semantics and fallback UX.

## Phase 1: Service Adapter + Thin Slice

- Add MemoryKernel adapter in Tauri backend.
- Add handshake + health status command.
- Integrate `query ask` for Draft flow only.
- Add telemetry/audit events for enrichment on/off and failure reason.

## Phase 2: Hardening

- Add CI consumer contract suite against pinned producer version.
- Add chaos-style local tests: service down, timeout, version mismatch, malformed payload.
- Add release gate requiring matrix + contract test pass.

## Phase 3: Expansion

- Add recall-based enrichment where it improves result quality.
- Evaluate OutcomeMemory gate integration for confidence/risk workflows.
- Optionally evaluate embedded path only with data-backed justification.

## Go/No-Go Gates

Proceed from each phase only if:

1. Existing AssistSupport behavior remains available under all failure modes.
2. Contract tests pass against pinned producer version.
3. Runtime diagnostics are actionable for support engineers.
4. End-user quality improves on agreed ticket sets.

## Open Questions to Resolve Early

1. Ownership: who approves MemoryKernel pin upgrades for AssistSupport?
2. Packaging: should AssistSupport auto-start MemoryKernel service or require external runbook?
3. Data retention: which context packages should AssistSupport persist locally and for how long?
4. UX: where to surface enrichment confidence and failure reasons without clutter?

## Immediate Next Planning Artifact

Create an AssistSupport RFC for “MemoryKernel v1 Service Adapter” containing:

- pinned producer version
- request/response contract table
- timeout/fallback policy
- test plan and CI gates
- rollout and rollback steps
