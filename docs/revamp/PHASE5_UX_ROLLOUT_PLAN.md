# Phase 5 UX Rollout Plan (Flag-Gated, Reversible)

Status: Active (Phase 5 pre-revamp entry artifact)  
Scope: Phase 5 revamp surfaces only  
Goal: Ship iteratively without breaking operators or forcing rework.

## 1) Flags (Source of Truth)

Flag resolution and policy rules:
- `/Users/d/Projects/AssistSupport/src/features/revamp/flags.ts`

Flags in scope:
1. `VITE_ASSISTSUPPORT_REVAMP_APP_SHELL`
2. `VITE_ASSISTSUPPORT_REVAMP_INBOX`
3. `VITE_ASSISTSUPPORT_REVAMP_WORKSPACE`
4. `VITE_ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2` (only adds commands; should not break legacy)

Policy flags (must remain env-authoritative outside dev builds):
1. `VITE_ASSISTSUPPORT_ENABLE_ADMIN_TABS`
2. `VITE_ASSISTSUPPORT_ENABLE_NETWORK_INGEST`

## 2) Rollout Mapping (What Each Flag Must Gate)

1. `ASSISTSUPPORT_REVAMP_APP_SHELL`
   - Gates: new shell layout + revamp theme surfaces (`.app-shell-revamp`).
   - Must not gate: core business logic, generation behavior, gating rules.
   - Primary switch site: `/Users/d/Projects/AssistSupport/src/App.tsx`

2. `ASSISTSUPPORT_REVAMP_WORKSPACE`
   - Gates: Draft workbench presentation (`WorkspaceRevampPage` vs legacy `DraftTab`).
   - Primary router: `/Users/d/Projects/AssistSupport/src/features/workspace/WorkspacePage.tsx`

3. `ASSISTSUPPORT_REVAMP_INBOX`
   - Gates: Queue-first inbox vs legacy follow-ups.
   - Primary router: `/Users/d/Projects/AssistSupport/src/features/inbox/InboxPage.tsx`

4. `ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2`
   - Gates: extra commands only (no removal of legacy).
   - Primary site: `/Users/d/Projects/AssistSupport/src/features/app-shell/commands.ts`

## 3) Rollout Sequence (Non-Ambiguous)

The revamp should ship in this order to avoid rework:

1. Shell first (`REVAMP_APP_SHELL`)
   - Establish tokens, layout primitives, and vibrancy defaults.
   - Keep legacy pages mounted inside revamp shell with token bridge.

2. Draft workbench (`REVAMP_WORKSPACE`)
   - The core workflow is Draft; redesign here pays down the most operator friction.

3. Inbox/Queue (`REVAMP_INBOX`)
   - Once Draft is stable, redesign the triage entry.

4. Settings polish (no new flag required if Settings is already stable under shell)

## 4) Reversibility Contract

At any point before Phase 5 closeout:
- Turning OFF `REVAMP_*` flags must restore the legacy UX without breaking core workflows.

Acceptance:
- A dev can run with all revamp flags disabled and complete the operator loop:
  queue -> open draft -> generate -> handoff -> return to queue.

## 5) Evidence Expectations

For each rollout slice, add evidence under:
- `/Users/d/Projects/AssistSupport/docs/revamp/evidence/phase5/ux-*/`

Each evidence bundle includes:
1. `notes.md` (tradeoffs, what is deferred)
2. `verification.txt` (commands run + PASS)
3. `screenshots/` (happy path + at least one degraded state)

