# Joint Execution Playbook: AssistSupport + MemoryKernel

Updated: 2026-02-08 (Checkpoint C GO, Checkpoint D planning GO, hash-gate active)  
Owners: AssistSupport + MemoryKernel (Joint)

## 1) Executive Summary

This playbook is the canonical execution blueprint for the next integration phases between AssistSupport and MemoryKernel, designed to minimize coordination churn while preserving production safety.

Current locked baseline:
- AssistSupport repo: `/Users/d/Projects/AssistSupport`
- MemoryKernel pin (consumer-validated):
  - release tag: `v0.3.2`
  - commit SHA: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
  - service contract: `service.v2`
  - API contract: `api.v1`
  - integration baseline: `integration/v1`
- Integration mode: service-first
- Invariant: deterministic fallback always keeps Draft flow functional when enrichment is unavailable.

Primary outcome for this playbook:
- Run parallel producer/consumer work with explicit entry/exit gates and shared evidence, so each side can move autonomously without contract drift.

## 2) Current Integration State

Operational state (as of this playbook):
- Single Tauri backend adapter boundary is the only MemoryKernel integration boundary in AssistSupport.
- Startup preflight validates:
  - `GET /v1/health` contract versions
  - `POST /v1/db/schema-version` readiness
- Contract governance is enforced in AssistSupport CI:
  - pin: `/Users/d/Projects/AssistSupport/config/memorykernel-integration-pin.json`
  - matrix: `/Users/d/Projects/AssistSupport/docs/MEMORYKERNEL_COMPATIBILITY_MATRIX.md`
  - producer manifest mirror: `/Users/d/Projects/AssistSupport/config/memorykernel-producer-manifest.json`
- Consumer error behavior:
  - primary machine routing via `error.code`
  - transitional compatibility via `legacy_error`
  - non-2xx envelope policy enforced in tests.

### Checkpoint state
- Checkpoint A: `GO`
- Checkpoint B: `GO`
- Checkpoint C (steady-state service.v2 window): `GO` (approved 14-day window)
- Checkpoint D (service.v3 RFC kickoff): `GO` for planning, not runtime cutover

### Locked joint decisions
1. `error_code_enum` validation mode is set equality (order-independent).
2. `service.v3` keeps non-2xx `api_contract_version` absent unless a future RFC explicitly changes this.
3. Producer-manifest hash validation is active in AssistSupport CI via pin + mirrored manifest SHA-256 integrity checks; authenticated remote validation is enforced when `MEMORYKERNEL_REPO_READ_TOKEN` is configured.
4. Pin + compatibility matrix + mirrored producer manifest must be updated atomically in one PR.

## 3) Phase Plan (Next 4 Phases)

### Phase 1: Governed Steady-State Integration

| Field | Definition |
|---|---|
| Objective | Stabilize day-to-day integration with no drift and no regression in core Draft flow. |
| Owner | Joint |
| Dependencies | Current baseline pinned and green (`v0.3.2`, `service.v2`, `api.v1`). |
| Entry Criteria | Consumer CI green; producer CI green; manifest parity active. |
| Exit Criteria | Two consecutive weekly cycles with no unsignaled contract drift and no core Draft fallback regressions. |
| Deliverables | Weekly evidence bundle, drift-free pin/matrix/manifest state, signed integration review notes. |
| Verification Commands | AssistSupport: `pnpm run typecheck`, `pnpm run test`, `pnpm run test:memorykernel-contract`, `pnpm run test:ci`. MemoryKernel: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-targets --all-features`, `./scripts/verify_service_contract_alignment.sh --memorykernel-root <path>`. |

Sprint target: Week 1-2.

### Phase 2: Consumer Runtime Hardening + Observability

| Field | Definition |
|---|---|
| Objective | Improve runtime diagnostics and operational predictability without changing core enrichment semantics. |
| Owner | AssistSupport |
| Dependencies | Phase 1 exit; producer envelope policy unchanged. |
| Entry Criteria | Deterministic fallback tests green. |
| Exit Criteria | Enrichment lifecycle states and failure reasons are observable and triage-ready; rollback drill completed once. |
| Deliverables | Enhanced runtime lifecycle docs, diagnostics mapping table (`docs/implementation/MEMORYKERNEL_RUNTIME_DIAGNOSTICS_MATRIX.md`), incident runbook evidence (`docs/implementation/MEMORYKERNEL_ROLLBACK_DRILL_2026-02-08.md`), updated test evidence artifact policy. |
| Verification Commands | `pnpm run test:memorykernel-contract`, `pnpm run test:ci`, plus manual runbook validation from `/Users/d/Projects/AssistSupport/docs/OPERATIONS.md`. |

Sprint target: Week 3.

### Phase 3: Cross-Repo Automation and Release Handoff Compression

| Field | Definition |
|---|---|
| Objective | Reduce manual coordination by enforcing cross-repo governance checks and repeatable release handoffs. |
| Owner | Joint |
| Dependencies | Phase 2 exit; producer manifest stable for at least one sprint. |
| Entry Criteria | Both repos aligned on producer manifest schema and policy fields. |
| Exit Criteria | Repin workflow can be completed with one standard PR template and no ad hoc coordination steps; manifest hash enforcement remains green for two consecutive baseline validations. |
| Deliverables | Cross-repo validation design, release handoff template, expected evidence manifest for every baseline update, and hash-enforced producer-manifest parity checks. |
| Verification Commands | AssistSupport CI with governance gate; MemoryKernel parity/alignment scripts; one full dry-run of repin handoff from producer release tag to consumer merge. |

Sprint target: Week 4.

### Phase 4: Service.v3 Migration Readiness (Planning + Pre-Implementation)

| Field | Definition |
|---|---|
| Objective | Prepare controlled migration from `service.v2` with explicit `legacy_error` retirement gates, no runtime surprises. |
| Owner | Joint (MemoryKernel RFC lead, AssistSupport adoption lead) |
| Dependencies | Phase 3 exit; no unresolved high-severity integration issues. |
| Entry Criteria | Producer publishes service.v3 RFC draft with overlap and deprecation policy. |
| Exit Criteria | Joint-approved migration plan with acceptance tests, repin criteria, and rollback strategy signed by both sides. |
| Deliverables | Service.v3 migration spec, compatibility test plan, rollout timeline, deprecation communication protocol, and consumer rehearsal plan (`docs/implementation/SERVICE_V3_CONSUMER_REHEARSAL_PLAN.md`). |
| Verification Commands | RFC checklist pass; contract test matrix drafted for v2-only, v2+v3 overlap, and v3-final states. |

Sprint target: Week 5+ (planning gate only in this playbook).

## 4) Single Source of Truth Contract Governance Flow

### Artifacts of record
- Consumer pin: `/Users/d/Projects/AssistSupport/config/memorykernel-integration-pin.json`
- Consumer compatibility matrix: `/Users/d/Projects/AssistSupport/docs/MEMORYKERNEL_COMPATIBILITY_MATRIX.md`
- Consumer mirrored producer manifest: `/Users/d/Projects/AssistSupport/config/memorykernel-producer-manifest.json`
- Producer canonical manifest: `contracts/integration/v1/producer-contract-manifest.json` (MemoryKernel repo)

### Standard baseline update protocol
1. MemoryKernel publishes immutable tag + commit + updated canonical manifest.
2. AssistSupport mirrors producer manifest and updates pin + matrix in the same PR.
3. AssistSupport CI enforces atomic update of pin/matrix/manifest.
4. AssistSupport runs contract suite and publishes evidence artifact.
5. Merge only after all required checks are green.

### Additive error code protocol (service.v2)
- Standard notice: minimum 10 business days before producer release.
- Required producer package: manifest update + docs + tests + OpenAPI updates.
- Consumer action window: add mapping/tests before repin.
- CI behavior: unknown or drifted codes fail governance validation.

### Emergency change protocol
- Producer must provide notice within 24 hours and publish same-day docs/spec/tests.
- Consumer immediately freezes baseline upgrades except emergency security/availability patches.
- Joint triage checkpoint within one business day.
- Rollback default: stay on last green approved pin until consumer test gates are restored.

## 5) Service.v3 Migration Scaffold

### Preconditions
- Service.v2 stable window complete with no unresolved high-severity drift issues.
- Producer publishes service.v3 RFC and migration timelines.
- Consumer has passing compatibility suite and explicit v3 feature flag strategy.

### Overlap window strategy
- Maintain dual-compat parsing during overlap (`service.v2` + `service.v3` support).
- Keep deterministic fallback unchanged throughout overlap.
- Require explicit cutover checklist before removing v2 path.

### Deprecation gates
1. Producer confirms v3 contract locked and tested.
2. Consumer green against v3 baseline with parity checks.
3. Joint sign-off on support window end date.
4. Deprecation communication sent before any `legacy_error` removal.

### Consumer cutover acceptance criteria (hard gate)
1. Producer publishes immutable v3 tag + commit with updated manifest/OpenAPI/spec.
2. AssistSupport updates pin + matrix + mirrored manifest atomically in one PR.
3. `pnpm run check:memorykernel-pin` passes with v3 expectations.
4. `pnpm run test:memorykernel-contract` passes with service.v3 non-2xx envelope assertions.
5. Deterministic fallback tests remain green for offline/timeout/malformed/version-mismatch/non-2xx.
6. `pnpm run test:ci` passes.
7. Rollback rehearsal verifies re-pin to last approved baseline without Draft-flow regression.

## 6) Risk Register

| Risk | Severity | Trigger Signal | Mitigation | Owner |
|---|---|---|---|---|
| Contract drift between producer manifest and consumer mirror | High | CI pin/manifest sync failure | Keep atomic update gate and hash-validated parity checks | AssistSupport |
| Unannounced additive error codes | High | Unknown `error.code` appears in producer artifacts | Enforce notice policy + consumer pre-adoption test updates | MemoryKernel |
| Core Draft flow regression due to enrichment coupling | Critical | Draft generation blocked when MemoryKernel offline | Preserve non-blocking fallback invariants and continuous chaos-path testing | AssistSupport |
| Policy drift in non-2xx envelope shape | High | Contract tests fail on envelope fields | Keep producer alignment gate + consumer envelope assertions | Joint |
| Slow incident recovery during baseline changes | Medium | Time-to-restore exceeds runbook budget | Standardize rollback checklist + quarterly drill | Joint |
| Service.v3 migration ambiguity | Medium | Conflicting deprecation timing assumptions | Publish and sign RFC with explicit readiness gates | Joint |

## 7) Operating Cadence

### Weekly integration review checklist
- Confirm current approved baseline tag/SHA unchanged or intentionally updated.
- Review governance check status in both repos.
- Review any proposed producer contract changes and lead-time compliance.
- Verify no unresolved high-severity integration incidents.
- Confirm action owners and deadlines for open items.

### Release handoff checklist
- Producer supplies immutable tag, commit SHA, manifest, and change summary.
- Consumer updates pin/matrix/manifest in one PR.
- Consumer runs required verification commands and publishes evidence.
- Joint go/no-go decision logged with rollback path.

### Incident rollback checklist
- Freeze baseline upgrades.
- Revert to last approved pin/matrix/manifest trio.
- Re-run contract and CI verification.
- Publish incident summary with root cause and prevention action.

## 8) RACI Matrix

| Activity | Responsible | Accountable | Consulted | Informed |
|---|---|---|---|---|
| Producer contract/schema changes | MemoryKernel | MemoryKernel | AssistSupport | Joint stakeholders |
| Consumer pin/matrix/manifest repin PR | AssistSupport | AssistSupport | MemoryKernel | Joint stakeholders |
| Cross-repo compatibility sign-off | Joint | Joint leads | Security/QA owners | Team |
| Emergency integration incident response | Joint | AssistSupport | MemoryKernel | Team |
| Service.v3 migration planning | Joint | MemoryKernel (RFC), AssistSupport (adoption) | QA/Security | Team |

## 9) No-Regression Guardrails (AssistSupport Core Flow)

The following are hard invariants for every phase:
1. Draft flow must remain available when MemoryKernel is disabled, offline, mismatched, malformed, or timing out.
2. MemoryKernel enrichment must remain optional and non-blocking.
3. Startup preflight failures must produce actionable diagnostics, not fatal app startup failures.
4. Contract-test suite must remain mandatory for baseline changes.
5. Any change weakening fallback determinism is a release-blocking defect.

## 10) Parallel Work Split

### AssistSupport track (independent)
- Maintain and evolve consumer governance checks (`pin/matrix/manifest`).
- Expand deterministic fallback and lifecycle diagnostics test coverage.
- Keep operations runbook and incident playbooks current.

### MemoryKernel track (independent)
- Maintain canonical producer manifest and alignment gates.
- Manage service contract policy updates with required notice windows.
- Draft and socialize service.v3 RFC with explicit migration milestones.

### Joint synchronization points (scheduled)
- Weekly 30-minute integration checkpoint.
- Baseline update review checkpoint per release.
- Service.v3 readiness decision checkpoint.

## 11) Definition of Done for This Playbook

This playbook is considered active and adopted when:
- Both repos reference it in their implementation planning docs.
- Both sides agree to use the phase entry/exit gates for integration decisions.
- The handoff prompt below is executed and producer alignment response is captured.

## 12) Copy/Paste Handoff Prompt for MemoryKernel Codex

```text
MemoryKernel Codex (GPT-5.3),

AssistSupport published the canonical joint execution playbook at:
- /Users/d/Projects/AssistSupport/docs/implementation/JOINT_EXECUTION_PLAYBOOK.md

Please perform producer-side alignment review against this playbook.

What we need from you:
1) Confirm alignment with the 4-phase plan and identify any producer-side conflicts.
2) Confirm owner commitments for each relevant phase and provide exact deliverables.
3) Confirm governance compatibility with:
   - additive error-code protocol (10 business-day standard notice)
   - emergency protocol (24h notice + same-day docs/spec/tests)
   - service.v3 migration scaffold and deprecation gates.
4) Provide producer-side verification command set for each phase gate.
5) Provide any modifications needed to improve parallel execution and reduce cross-repo wait states.

Return format:
1) Alignment verdict (Aligned / Aligned with edits / Not aligned)
2) Required edits to the playbook (specific section-by-section)
3) Producer commitments with dates/sprint targets
4) Risks and blockers from producer perspective
5) A copy/paste response prompt addressed back to AssistSupport Codex containing your final aligned plan and next asks
```
