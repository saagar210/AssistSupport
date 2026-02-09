# Phase 5 UX Audit (Pre-Revamp Baseline + Priorities)

Status: Active (Phase 5 pre-revamp entry artifact)  
Scope: AssistSupport UX only (no workflow/contract changes)  
Audience: Internal IT support engineers on macOS workstations

This audit captures the current UX baseline as evidence and defines what Phase 5 must improve.
It is intentionally written in operator terms (what slows the engineer down, what causes errors).

## Operator Baseline Walkthroughs (What We Must Support)

### A) Happy Path: Ticket -> Response -> Handoff

1. Intake: paste ticket context, link ticket (if available).
2. Diagnose: select a decision tree, generate checklist, mark progress.
3. Draft: generate response, review citations, edit, produce final.
4. Handoff: copy/export/Jira post with policy gating.

Primary implementation surface:
- `/Users/d/Projects/AssistSupport/src/components/Draft/DraftTab.tsx`
- `/Users/d/Projects/AssistSupport/src/components/Draft/ResponsePanel.tsx`

### B) Degraded Path: Model Not Loaded

Expectation:
- The UI must make it obvious what is missing and provide a clear next action.

Primary surface:
- `/Users/d/Projects/AssistSupport/src/components/Draft/AiReadinessBanner.tsx`
- `/Users/d/Projects/AssistSupport/src/components/Settings/SettingsTab.tsx`

### C) Degraded Path: KB Not Indexed / Index Empty

Expectation:
- The UI should not present "high confidence" cues when there are no sources.
- The operator should get an actionable route to fix KB configuration/index.

Primary surfaces:
- `/Users/d/Projects/AssistSupport/src/components/Draft/ResponsePanel.tsx` (copy gating + sources)
- `/Users/d/Projects/AssistSupport/src/components/Knowledge/KnowledgeTab.tsx` (or equivalent)

### D) Degraded Path: MemoryKernel Offline / Degraded

Expectation:
- Degradation is visible in the status strip.
- Draft generation remains functional (deterministic fallback).

Primary surfaces:
- `/Users/d/Projects/AssistSupport/src/components/Draft/AiReadinessBanner.tsx`
- `/Users/d/Projects/AssistSupport/src/hooks/useMemoryKernelEnrichment.ts`

## Findings (Prioritized)

### P1 (Must Fix In Phase 5)

1. Visual hierarchy in Draft does not consistently highlight the "next right action".
   - Symptom: operators can miss whether they are in "Draft" versus "Handoff" mode.
   - Fix intent: unify action placement in the Draft workbench (one primary CTA cluster) and reduce duplicated controls.
   - Evidence: `/Users/d/Projects/AssistSupport/src/components/Draft/DraftTab.tsx`

2. Trust signaling is not uniform across states.
   - Symptom: confidence/citations are present, but scanning sources and understanding "why this answer" is still slower than it should be.
   - Fix intent: redesign source presentation to be scannable (headline + key excerpt + 1-click preview) and keep "no citation = no claim" visually explicit.
   - Evidence: `/Users/d/Projects/AssistSupport/src/components/Draft/ResponsePanel.tsx`

3. Degraded-state recovery paths are not consistently placed.
   - Symptom: "Model not loaded" / "KB not indexed" / "MemoryKernel offline" are visible, but the next action can be unclear.
   - Fix intent: standardize a single "Needs attention" pattern: state -> impact -> 1 primary fix action -> 1 secondary link to Settings/Knowledge.
   - Evidence: `/Users/d/Projects/AssistSupport/src/components/Draft/AiReadinessBanner.tsx`

### P2 (Strong Improvements, Not Strictly Blocking)

1. Sidebar and topbar do not yet feel Apple-native on macOS.
   - Symptom: the app reads as a web dashboard rather than a desktop tool.
   - Fix intent: introduce default vibrancy surfaces, consistent corner radii, and calmer typography rhythm.
   - Evidence: `/Users/d/Projects/AssistSupport/src/features/revamp/shell/RevampShell.tsx`, `/Users/d/Projects/AssistSupport/src/styles/revamp/theme.css`

2. Density inconsistency across panels.
   - Symptom: some panels are dense and efficient, others read as "form-like" with large whitespace.
   - Fix intent: normalize spacing and panel header patterns via shared primitives.

### P3 (Polish / Consistency)

1. Microcopy consistency.
   - Symptom: small label variants ("Generate", "Draft", "Respond", "Handoff") can drift.
   - Fix intent: lock a single vocabulary and a glossary for operator UI labels.
   - Reference: `/Users/d/Projects/AssistSupport/docs/revamp/FEATURE_LOCK_WORKFLOW.md`

2. Reduced transparency support for vibrancy.
   - Symptom: vibrancy is desirable by default but must degrade gracefully.
   - Fix intent: implement `prefers-reduced-transparency` fallback for revamp surfaces.
   - Evidence: `/Users/d/Projects/AssistSupport/src/styles/revamp/theme.css`

## Keep List (Do Not Break)

1. Copy gating + override audit behavior:
   - `/Users/d/Projects/AssistSupport/src/components/Draft/ResponsePanel.tsx`
2. Policy flags env-authoritative outside dev:
   - `/Users/d/Projects/AssistSupport/src/features/revamp/flags.ts`
3. Deterministic fallback behavior for MemoryKernel degradation:
   - `/Users/d/Projects/AssistSupport/src/hooks/useMemoryKernelEnrichment.ts`
4. Four-stage funnel framing:
   - `/Users/d/Projects/AssistSupport/docs/revamp/FEATURE_LOCK_WORKFLOW.md`

## Phase 5 Acceptance Criteria (Audit-Derived)

1. Operators can complete: queue -> open draft -> generate -> handoff -> return to queue without hunting for actions.
2. "Needs attention" states provide a single obvious fix action.
3. Sources are scannable in under 5 seconds (headline + excerpt + preview).
4. Vibrancy is default, but reduced transparency mode remains readable and stable.

