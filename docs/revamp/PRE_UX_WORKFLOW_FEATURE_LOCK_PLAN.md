# Pre-UX Workflow and Feature Lock Plan

Status: Proposed  
Purpose: Lock workflow and feature scope before full UX revamp implementation

## 1) Why This Exists

The team wants to avoid reworking UX twice.  
This plan defines exactly what workflow and feature changes should happen before visual redesign so the revamp can be executed once on a stable product shape.

## 2) Current Product Shape (As Implemented)

Current core surfaces:
1. Draft/Workspace response generation.
2. Follow-ups/Inbox queue view.
3. Sources and ingestion.
4. Knowledge and search.
5. Analytics and coaching.
6. Ops diagnostics and recovery workflows.
7. Settings/integration configuration.
8. Search API + MemoryKernel adapters with deterministic fallback behavior.

Current strengths:
1. Strong local-first security and governance controls.
2. MemoryKernel boundary discipline and fallback reliability.
3. Deep test coverage and operational evidence generation.
4. Keyboard support and command palette foundations already in place.

Current gaps (workflow-level):
1. Core operator workflow is still distributed across tabs rather than one primary operating loop.
2. Case ownership and queue accountability are present but not fully first-class in every path.
3. State transitions from ticket intake to final handoff are not uniformly explicit.
4. Some advanced surfaces are useful but crowd primary operator focus.

## 3) External Patterns We Should Borrow (and Why)

Pattern references:
1. Jira Service Management request/incident/problem/change queues and work categories.
2. Freshservice unified service catalog + SLA + workflow automation.
3. Zendesk/Intercom agent-assist and bot/human workload separation.
4. PagerDuty/Opsgenie escalation sequencing and on-call handoff reliability.
5. WCAG and APG interaction patterns for keyboard-first professional tooling.

What these tools consistently do well:
1. Single intake and triage lane with explicit ownership.
2. Strong queue semantics (priority, SLA, escalation, assignment).
3. AI assist as support layer, not hidden replacement for deterministic workflow.
4. Clear separation of “human-required” vs “automation-handled” work.
5. Incident/handoff artifacts generated as first-class outputs.

## 4) Product Workflow We Should Lock Before UX

Target operator workflow (single canonical loop):
1. Intake
2. Triage
3. Diagnose
4. Respond
5. Handoff
6. Learn and improve

Definition of each stage:
1. Intake:
   - ticket body + metadata normalized into a structured case object.
2. Triage:
   - set priority, owner, SLA target, and queue bucket.
3. Diagnose:
   - gather KB/search context and optional MemoryKernel enrichment.
4. Respond:
   - compose answer with confidence and citation grounding.
5. Handoff:
   - produce explicit action record and next-owner context.
6. Learn:
   - capture rating/quality signals and KB gap candidates.

## 5) Feature Decisions Before UX Revamp

## Keep (no removal)
1. MemoryKernel adapter boundary and preflight rules.
2. Deterministic fallback and non-blocking enrichment behavior.
3. Ops diagnostics and governance checks.
4. Response quality analytics and coaching.
5. Command palette and keyboard shortcut framework.

## Change (workflow behavior)
1. Make queue-first workflow the default operator entry path.
2. Make case state machine explicit across all major screens.
3. Make ownership and SLA fields mandatory in triage flow.
4. Standardize degraded-state messaging across Draft, Queue, and Ops.

## Add before UX redesign
1. Unified case object contract (single data shape used by queue, draft, analytics).
2. Explicit workflow state machine with transition guards:
   - new -> triaged -> diagnosing -> response-ready -> handoff-complete.
3. Triage policy engine:
   - priority rules
   - SLA target assignment
   - escalation trigger criteria.
4. Work split contract:
   - “human required” vs “automation suggested” vs “auto-complete candidate”.
5. Response package output contract:
   - customer message
   - internal notes
   - evidence/citations
   - follow-up actions.
6. Handoff packet contract:
   - owner
   - status
   - risk flags
   - unresolved blockers
   - next check step.
7. Macro/template lifecycle policy:
   - draft template
   - reviewed template
   - deprecated template.
8. Queue workload balancing signals:
   - assignment skew
   - at-risk concentration
   - backlog pressure.

## Remove or demote before UX redesign
1. Demote experimental surfaces from primary nav (keep accessible in Ops or feature-flag route).
2. Hide low-frequency advanced controls behind progressive disclosure.
3. Remove duplicate pathways that perform the same action with different labels.

## 6) Proposed Navigation Model to Support the Locked Workflow

Primary navigation (operator-facing):
1. Queue
2. Workspace
3. Knowledge
4. Analytics
5. Ops
6. Settings

Secondary entry model:
1. Command palette remains universal and mirrors all core actions.
2. Keyboard shortcuts map to workflow stages, not just tab names.

## 7) Pre-UX “Feature Complete” Delivery Slices

## Slice A: Case Contract and Workflow Engine
Deliverables:
1. Unified case schema and adapter mappers.
2. Workflow state transitions with tests.
3. Stage-specific validation and error taxonomy.

Acceptance:
1. Every case follows one state model.
2. Invalid transitions are blocked and surfaced clearly.

## Slice B: Queue and Triage Enforcement
Deliverables:
1. Ownership and SLA required fields in triage.
2. Priority/escalation rule engine.
3. Queue buckets derived from deterministic rules.

Acceptance:
1. Triage cannot complete without required metadata.
2. Escalation and at-risk rules are test-covered.

## Slice C: Response and Handoff Contract
Deliverables:
1. Standard response package model.
2. Handoff packet model and export path.
3. Human/automation task split markers.

Acceptance:
1. Every completed case emits a consistent handoff record.
2. Analytics can consume response package metadata without ad-hoc transforms.

## Slice D: Surface Rationalization
Deliverables:
1. Primary vs secondary surface classification.
2. Duplicate action path cleanup.
3. Feature-flag policy for experimental controls.

Acceptance:
1. No duplicate primary-path actions remain.
2. Experimental tools are available but do not distract primary flow.

## 8) Verification Gates for Pre-UX Lock

Required commands:
1. `pnpm run typecheck`
2. `pnpm run test`
3. `pnpm run test:memorykernel-contract`
4. `pnpm run test:memorykernel-phase3-dry-run`
5. `pnpm run test:ci`
6. `pnpm run check:memorykernel-pin`
7. `pnpm run check:memorykernel-governance`
8. `pnpm run check:memorykernel-handoff`
9. `pnpm run check:memorykernel-handoff:service-v3-candidate`

Pre-UX lock is complete only if all commands pass after the workflow changes are merged.

## 9) Risk Register for This Pre-UX Step

1. Risk: Feature growth delays revamp.
   - Mitigation: enforce only workflow-critical additions from this plan.
2. Risk: Overfitting to enterprise ITSM patterns adds complexity for single-user mode.
   - Mitigation: keep strict “single-user local workstation” default behavior.
3. Risk: New workflow engine breaks existing tested paths.
   - Mitigation: compatibility wrappers + transition contract tests.
4. Risk: Added triage strictness slows operators initially.
   - Mitigation: defaults/autofill + command palette quick-complete actions.

## 10) Decision Checkpoint Before UX Execution

Before starting visual redesign, confirm all of the following:
1. Case state machine merged and stable.
2. Queue-first default behavior locked.
3. Ownership/SLA/escalation rules active.
4. Response and handoff contracts finalized.
5. Duplicate and distracting surfaces demoted.
6. Full verification suite green.

If any checkpoint item is incomplete, UX revamp should remain blocked.

## 11) What Happens After This Lock

Once this pre-UX lock is complete:
1. Proceed with `UX_FULL_REVAMP_IMPLEMENTATION_PLAN.md`.
2. Revamp focuses on presentation, interaction efficiency, and operator ergonomics.
3. No major workflow or feature model changes should be introduced during visual redesign without explicit exception approval.
