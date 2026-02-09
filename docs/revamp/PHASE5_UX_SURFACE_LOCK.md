# Phase 5 UX Surface Lock (Pre-Revamp Contract)

Status: Active (Phase 5 pre-revamp entry artifact)  
Scope: AssistSupport UX revamp only (no new feature surfaces)  
Audience: Internal IT support engineers on macOS workstations

This document freezes the user-facing surface area that Phase 5 is allowed to redesign.
If Phase 5 implementation discovers a missing workflow or a required new screen, this file
must be updated first, then the Phase 5 plan must be re-reviewed to avoid a second redesign.

## Non-Negotiable Invariants (Must Hold Throughout Phase 5)

1. Offline-first: the app must function without external network connectivity.
2. Policy gates remain authoritative outside dev:
   - `ASSISTSUPPORT_ENABLE_ADMIN_TABS` cannot be enabled via localStorage in non-dev builds.
   - `ASSISTSUPPORT_ENABLE_NETWORK_INGEST` cannot be enabled via localStorage in non-dev builds.
   - Evidence: `/Users/d/Projects/AssistSupport/src/features/revamp/flags.ts`
3. Copy/export trust policy remains enforced:
   - "No citation = no claim" gating persists.
   - Overrides require a reason and emit a local audit event (no response text logged).
   - Evidence: `/Users/d/Projects/AssistSupport/src/components/Draft/ResponsePanel.tsx`
4. Four-stage funnel remains the primary operator mental model:
   - Intake -> Diagnose -> Draft -> Handoff
   - Evidence: `/Users/d/Projects/AssistSupport/docs/revamp/FEATURE_LOCK_WORKFLOW.md`
5. Revamp is flag-gated and reversible until Phase 5 closeout:
   - Shell: `VITE_ASSISTSUPPORT_REVAMP_APP_SHELL`
   - Inbox: `VITE_ASSISTSUPPORT_REVAMP_INBOX`
   - Workspace: `VITE_ASSISTSUPPORT_REVAMP_WORKSPACE`
   - Evidence: `/Users/d/Projects/AssistSupport/src/App.tsx`, `/Users/d/Projects/AssistSupport/src/features/revamp/flags.ts`

## Frozen UX Surface Map (Screens + Responsibilities)

### 1) App Shell (Navigation + Global Entry Points)

Purpose:
- Provide fast navigation between operator areas; keep global state visible (AI readiness, model selection entry points, command palette).

In-scope redesign:
- Sidebar visual hierarchy, spacing, iconography, collapsed behavior.
- Top bar composition (title, search/command palette affordances, model selector placement).

Out-of-scope behavioral changes:
- Tab enablement policy logic (may be restyled, not redefined).
- Shortcut semantics (may be clarified in UI, not changed without explicit approval).

Primary files:
- `/Users/d/Projects/AssistSupport/src/App.tsx`
- `/Users/d/Projects/AssistSupport/src/components/Layout/Sidebar.tsx`
- `/Users/d/Projects/AssistSupport/src/components/Layout/Header.tsx`
- `/Users/d/Projects/AssistSupport/src/components/shared/CommandPalette.tsx`

### 2) Draft (Primary Workbench)

Purpose:
- Convert ticket context into a support response with explicit provenance and safe handoff.

Frozen workflow steps:
- Intake: ticket/context capture (including evidence capture).
- Diagnose: decision tree selection + checklist.
- Draft: generation + editing + sources/citations.
- Handoff: copy/export/Jira actions (gated).

In-scope redesign:
- Panel layout, density, typography, callouts.
- AI readiness strip presentation (without changing data semantics).
- Sources presentation (scannability, expand/collapse).

Out-of-scope behavioral changes:
- No new generation modes beyond existing ones in Phase 5.
- No new ingestion pipeline behavior (KB/MemoryKernel stays as-is).

Primary files:
- `/Users/d/Projects/AssistSupport/src/components/Draft/DraftTab.tsx`
- `/Users/d/Projects/AssistSupport/src/components/Draft/InputPanel.tsx`
- `/Users/d/Projects/AssistSupport/src/components/Draft/DiagnosisPanel.tsx`
- `/Users/d/Projects/AssistSupport/src/components/Draft/ResponsePanel.tsx`
- `/Users/d/Projects/AssistSupport/src/components/Draft/AiReadinessBanner.tsx`

### 3) Inbox / Queue (Triage + Work Intake)

Purpose:
- Make it easy to select the next unit of work and jump into Draft with context.

In-scope redesign:
- Queue list density, priority signaling, empty/degraded states.
- Queue-to-draft navigation clarity.

Out-of-scope behavioral changes:
- Queue model semantics remain stable; we only refine presentation.

Primary files:
- `/Users/d/Projects/AssistSupport/src/features/inbox/InboxPage.tsx`
- `/Users/d/Projects/AssistSupport/src/features/revamp/screens/QueueCommandCenterPage.tsx`

### 4) Knowledge (Local KB)

Purpose:
- Configure and validate local knowledge ingestion; make indexing state obvious and actionable.

In-scope redesign:
- Presentation of KB status, source path, indexing progress, and error recovery.

Out-of-scope behavioral changes:
- No ingestion algorithm changes in Phase 5 (Phase 4 already closed ingestion-quality blocking fixes).

Primary files:
- `/Users/d/Projects/AssistSupport/src/components/Knowledge/KnowledgeTab.tsx` (or equivalent if renamed)

### 5) Sources (Evidence / Referenced Documents)

Purpose:
- Inspect and navigate sources used by Draft responses.

In-scope redesign:
- Readability, filtering, and “send to draft” affordances.

Primary files:
- `/Users/d/Projects/AssistSupport/src/components/Sources/SourcesTab.tsx` (or equivalent if renamed)

### 6) Settings (Configuration + Audit + Backup)

Purpose:
- Configure model defaults, integration settings, exports/imports, and audit visibility.

In-scope redesign:
- Layout, grouping, and clarity of consequences.
- "AI Status & Guarantees" presentation (copy only; same policies).

Out-of-scope behavioral changes:
- No new settings keys without updating `validate_setting()` policy.

Primary files:
- `/Users/d/Projects/AssistSupport/src/components/Settings/SettingsTab.tsx`

## Deferred (Explicitly Not In Phase 5 UX Scope)

These items are valuable but intentionally excluded from Phase 5 UX so we do not destabilize the system during a redesign:

1. Stable deterministic chunk IDs across re-ingests (requires DB migration).
2. Hierarchical section breadcrumb storage.
3. Confluence source URL preservation.
4. LLM reranking / cross-encoder style relevance reranking.

Reference: `/Users/d/Projects/AssistSupport/docs/monorepo/VAULTMIND_BOUNDARY.md` and Phase 4 ingestion closeout.

