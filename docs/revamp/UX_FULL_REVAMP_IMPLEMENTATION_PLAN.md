# AssistSupport UX Full Revamp Implementation Plan

Status: Approved for execution  
Scope: AssistSupport UX rebuild only (MemoryKernel runtime contract remains service.v2/api.v1 pinned)  
Audience: Internal IT support engineers on macOS workstations

## 1) Objective and Definition of Done

### Objective
Replace the current AssistSupport user experience with a faster, clearer, keyboard-first, operations-grade UI that support engineers actually prefer using for full ticket lifecycles.

### Definition of done
The revamp is complete only when all of the following are true:
1. Core operator journey (intake -> triage -> diagnose -> draft -> handoff) is faster and clearer than current baseline.
2. Deterministic fallback behavior is preserved for all MemoryKernel failure modes.
3. Accessibility and keyboard operability are first-class and test-enforced.
4. The app remains compliant-ready for existing internal control requirements.
5. All revamp feature flags can be safely disabled without breaking legacy operation paths until final cutover.

## 2) Non-Negotiable Constraints

1. MemoryKernel integration boundary remains in the adapter layer only.
2. Enrichment remains optional and non-blocking.
3. Deterministic fallback remains unchanged for offline, timeout, malformed payload, version mismatch, and non-2xx responses.
4. Runtime service.v3 cutover remains blocked; this plan does not change runtime baseline.
5. Security posture cannot regress:
   - auth-required policy in non-dev modes
   - token handling through secure store paths
   - no new unauthenticated privileged network surfaces
6. Compliance posture cannot regress:
   - auditability
   - deterministic evidence generation
   - rollback readiness artifacts

## 3) What We Are Solving

Current pain points this plan resolves:
1. UI visual hierarchy is inconsistent and cognitively heavy in high-volume queue workflows.
2. Draft composition layout is functional but not optimized for rapid ticket operations.
3. State messaging (loading/empty/degraded/error) is not consistently actionable.
4. Navigation and command patterns are split across pages rather than centered on operator flow.
5. Instrumentation is strong, but not yet organized into UX-focused product signals for rollout decisions.

## 4) External Design Inputs (Research-Derived Principles)

The following principles are driving the revamp:
1. Progressive disclosure for advanced controls so primary workflows stay simple and low-error.
2. Unified request/incident workspace model with shared context and clear task ownership.
3. Strong empty-state and degraded-state guidance that always offers a next action.
4. Keyboard-first execution for high-throughput operators.
5. Strict accessibility baselines (contrast, focus visibility, semantic interaction patterns).

Reference inputs:
1. Atlassian Jira Service Management feature model (request, incident, problem, change, knowledge): <https://www.atlassian.com/software/jira/service-management/features>
2. Freshworks helpdesk operating model (omnichannel queue, routing, AI assist, analytics): <https://www.freshworks.com/helpdesk/>
3. PagerDuty incident orchestration patterns (guided remediation, stakeholder comms, accountability): <https://www.pagerduty.com/platform/incident-management/>
4. NN/g progressive disclosure guidance: <https://www.nngroup.com/articles/progressive-disclosure/>
5. IBM Carbon empty state pattern guidance: <https://carbondesignsystem.com/patterns/empty-states-pattern/>
6. WCAG 2.2 contrast and focus guidance:
   - <https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html>
   - <https://www.w3.org/WAI/WCAG22/Understanding/focus-visible.html>
7. WAI-ARIA APG interaction patterns: <https://www.w3.org/WAI/ARIA/apg/patterns/>
8. GitHub keyboard shortcut and command palette references:
   - <https://docs.github.com/en/get-started/accessibility/keyboard-shortcuts>
   - <https://docs.github.com/en/get-started/accessibility/github-command-palette>

## 5) Revamp Architecture (Target UX Operating Model)

1. One operations shell with three primary modes:
   - Queue Command Center
   - Draft Workbench
   - Analytics and Handoff
2. Persistent right-rail diagnostics and context intelligence:
   - MemoryKernel status
   - policy/risk hints
   - evidence and citations
3. Command palette as first-class navigation and action layer:
   - open ticket
   - jump queue filters
   - trigger generate/copy/save/handoff
4. Explicit state system:
   - loading, empty, no-results, degraded, blocked, success
   - each with exact operator next steps
5. Design-token driven theming with consistent spacing/typography/state colors.

## 6) Detailed Phase Plan

## Phase UX-1: Foundation and Information Architecture Reset

### Goal
Create a stable revamp foundation that standardizes layout, navigation, and design tokens before deeper feature rebuilds.

### Implementation steps
1. Define revamp token schema:
   - color roles (surface, text, emphasis, warning, risk)
   - typography scale for dense operations UI
   - spacing and elevation standards
2. Implement shell-level layout primitives:
   - top command bar
   - left nav (task domains)
   - center workspace
   - right diagnostics rail
3. Establish state components:
   - loading skeletons
   - empty-state cards
   - degraded-state panel with fallback explanation
4. Normalize nav IA labels and page routing names for operator mental model.

### Exit criteria
1. All major screens use shared layout primitives.
2. No ad-hoc color/spacing values remain in revamp paths.
3. Shared state components are used in Draft, Queue, Analytics.

### Required verification
1. `pnpm run typecheck`
2. `pnpm run test`
3. `pnpm run test:ci`
4. `pnpm run test:e2e:smoke`

## Phase UX-2: Queue Command Center Rebuild

### Goal
Make triage fast, legible, and keyboard-dominant under high ticket volume.

### Implementation steps
1. Rebuild queue list with clear priority bands:
   - at-risk
   - unassigned
   - in-progress
   - waiting/external
2. Add split-pane details preview with zero-context-switch triage.
3. Standardize queue actions with consistent keybindings and command palette parity.
4. Add operator workload and SLA risk indicators with deterministic thresholds.
5. Add degraded behavior messaging when enrichment is unavailable.

### Exit criteria
1. Full queue triage can be completed with keyboard only.
2. Queue actions are test-covered for click + keyboard parity.
3. At-risk and unassigned workflows are visibly and behaviorally deterministic.

### Required verification
1. `pnpm run test:e2e:revamp`
2. `pnpm run test:revamp-queue-rehearsal`
3. `pnpm run test`
4. `pnpm run test:ci`

## Phase UX-3: Draft Workbench Full Redesign

### Goal
Turn Draft into a focused operator workbench with clear progression and faster response authoring.

### Implementation steps
1. Rebuild Draft into staged workflow panels:
   - Intake context
   - Diagnostic analysis
   - Response composer
2. Introduce composition modes:
   - concise
   - standard
   - deep-dive
3. Add response confidence and quality cues inline (non-blocking).
4. Add stronger guidance for first response + follow-up drafts.
5. Improve copy/export/handoff ergonomics and reduce pointer travel.
6. Ensure all MemoryKernel statuses map to explicit operator-readable messages.

### Exit criteria
1. Draft flow presents explicit current step and next step at all times.
2. Operators can generate, refine, and hand off without leaving Draft context.
3. No hidden failure states; all errors/fallbacks are actionable.

### Required verification
1. `pnpm run test`
2. `pnpm run test:memorykernel-contract`
3. `pnpm run test:ci`
4. `pnpm run test:e2e:smoke`

## Phase UX-4: Analytics and Operator Coaching Modernization

### Goal
Make analytics directly actionable for team leads and shift operators.

### Implementation steps
1. Rebuild analytics around decisions, not raw metrics:
   - response quality risk
   - queue pressure
   - handoff reliability
2. Add operator scorecard with explicit top remediation actions.
3. Add drill-down links from scorecards to affected drafts/queues.
4. Add shift handoff packet UX with one-click export and audit trace.

### Exit criteria
1. Every analytics panel includes “what to do next.”
2. Drill-down paths are validated and deterministic.
3. Handoff packet generation is test-covered end-to-end.

### Required verification
1. `pnpm run test`
2. `pnpm run test:ci`
3. `pnpm run test:e2e:ops`

## Phase UX-5: Accessibility, Performance, and Reliability Hardening

### Goal
Bring revamp to production-grade quality for workstation deployment.

### Implementation steps
1. Accessibility hardening:
   - contrast checks against WCAG AA targets
   - focus-visible behavior on all keyboard-interactive elements
   - semantic ARIA pattern conformance for dialogs/menus/tables
2. Performance hardening:
   - initial render budget guardrails
   - expensive view memoization and virtualization where needed
3. Reliability hardening:
   - revamp-specific regression tests
   - deterministic degraded-mode rendering tests
4. Security and compliance evidence refresh for changed UX flows.

### Exit criteria
1. Accessibility checklist passes for all revamp screens.
2. Performance and stability metrics meet baseline improvement targets.
3. No P1/P2 regressions open in revamp paths.

### Required verification
1. `pnpm run typecheck`
2. `pnpm run test`
3. `pnpm run test:security-regression`
4. `pnpm run test:ci`
5. `pnpm run check:phase6-ops-hardening`

## Phase UX-6: Controlled Rollout and Legacy Decommission

### Goal
Ship revamp safely, retire legacy UI paths, and preserve rollback control.

### Implementation steps
1. Feature-flag rollout ladder:
   - internal dogfood
   - wider internal pilot
   - default-on
   - legacy-off
2. Capture rollout evidence:
   - operator feedback
   - issue rates
   - queue throughput deltas
3. Remove obsolete legacy screens/components after stability gates pass.
4. Finalize work-machine handoff package and runbook.

### Exit criteria
1. Revamp is default path with legacy path removed or disabled by policy.
2. Rollback procedures validated and documented.
3. All acceptance gates signed off in release packet.

### Required verification
1. `pnpm run test`
2. `pnpm run test:ci`
3. `pnpm run test:e2e:smoke`
4. `pnpm run test:e2e:revamp`
5. `pnpm run check:monorepo-readiness:full`

## 7) Risk Register (Revamp-Specific)

1. Risk: Visual churn slows operators.
   - Mitigation: preserve keyboard contracts and command aliases across old/new UI.
2. Risk: Over-design introduces complexity.
   - Mitigation: progressive disclosure; keep advanced controls behind explicit expansion.
3. Risk: Degraded-state confusion during MemoryKernel outages.
   - Mitigation: standardized degraded-state panel + deterministic fallback text.
4. Risk: Accessibility regressions in fast rebuild cycles.
   - Mitigation: mandatory focus/contrast checks and accessibility test checklist gate.
5. Risk: Scope explosion.
   - Mitigation: phase exit gates; no cross-phase merge without passing required verification.

## 8) Compliance and Security Guardrails During Revamp

1. Every phase must preserve:
   - audit event integrity
   - explicit operator attribution on key actions
   - secure token handling paths
2. Every phase must produce:
   - evidence artifact updates under `docs/revamp/evidence/`
   - updated risk notes in `docs/revamp/RISK_REGISTER.md`
3. No phase may bypass:
   - `pnpm run test:ci`
   - MemoryKernel contract checks for integration-touching UI changes
   - security regression checks before final rollout gate

## 9) Execution Discipline

1. No “big bang” merge: each phase merges only after exit gate evidence.
2. No hidden TODO debt: unresolved items are tracked as explicit follow-up entries with owner and gate impact.
3. If a phase fails exit criteria:
   - freeze new scope
   - remediate defects
   - rerun required verification

## 10) Immediate Next Actions

1. Approve this plan as the canonical UX revamp execution contract.
2. Start Phase UX-1 implementation branch and evidence log.
3. Run pre-phase baseline snapshot:
   - current UX screenshots
   - current operator task timings
   - current defect/risk baseline
4. Begin foundation token/layout refactor with feature flags and regression tests.
