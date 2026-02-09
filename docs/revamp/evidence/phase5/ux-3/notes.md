# Phase 5 UX-3: Draft Workbench Redesign (Evidence Notes)

Goal
- Make the Draft workbench feel ops-grade and calm while preserving Phase 4 workflow + local AI contract behavior.

Scope
- CSS + layout changes only (no behavior changes) behind existing revamp flags.
- Reduce rail clutter when the revamp app shell is enabled (avoid 4-column layout).

Key Changes
- DraftTab revamp layout mode (`revampModeEnabled`) enables a 2-column grid:
  - Left: Intake (top) + Diagnose (bottom)
  - Right: Draft/Handoff response panel spanning full height
- Workflow strip updated to reflect all four stages consistently (Intake/Diagnose/Draft/Handoff).
- When the revamp app shell is enabled, workspace-specific rail content moves into the shell rail to avoid double rails.

Non-negotiables Verified
- No new network surfaces enabled by default.
- Workflow remains Intake -> Diagnose -> Draft -> Handoff.
- “No citation = no claim” copy gating and audit remain unchanged.

