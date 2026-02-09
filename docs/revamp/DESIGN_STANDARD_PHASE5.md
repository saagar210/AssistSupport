# Phase 5 Design Standard (Apple-Native Ops Console)

Status: Active (execution contract)  
Scope: Revamp UI/UX only; no behavioral/security changes unless explicitly approved as a bug fix  
Audience: Internal IT support engineers on macOS workstations

This document is the single source of truth for how Phase 5 should look and feel. If a UI decision is not justified by this standard (or by an explicit artifact in `docs/revamp/`), it should not ship in Phase 5.

## 1) Design Intent (Non-Negotiable)

1. Calm operations console: dense, legible, low-noise, long-session friendly.
2. Apple-native clarity: the UI should feel at-home on macOS (calm, precise, subtle depth).
3. Trust through clarity: state, provenance, and policy posture must be obvious.
3. Speed over novelty: keyboard-first, low friction, no “cute” UI.
4. Consistency over variety: one system, not a collage of styles.

## 2) Visual Language (Hard Rules)

1. Color:
   - Use semantic tokens only (surface/text/border/accent + intent colors).
   - No ad-hoc hex/rgba for hover/active/focus/shadows in revamp surfaces.
   - Intent colors are reserved for meaning (risk/warn/info/success), not decoration.
2. Typography:
   - Body: macOS-native system font stack (SF via `system-ui` on macOS).
   - Mono: system mono stack (`ui-monospace`/`SF Mono`).
   - Note: revamp currently imports bundled fonts for consistency; Phase 5 may remove those imports once the Apple-native token set is finalized.
   - Use weight and spacing for hierarchy before using color.
3. Density:
   - Default is dense, with clear hierarchy.
   - No “comfort mode” work in Phase 5 unless explicitly approved.
4. Elevation:
   - Use shared shadow tokens only (e.g. `--shadow-xs`, `--card-shadow`).
   - Avoid strong drop shadows; prefer subtle separation via borders and surface contrast.
5. Vibrancy (default):
   - Revamp surfaces default to macOS-style “vibrancy” (translucent surfaces + blur).
   - Must honor `prefers-reduced-transparency: reduce` by disabling blur/translucency and using solid surfaces.
5. Motion:
   - Allowed: calm entrance fades and small opacity transitions.
   - Disallowed: bouncy easing, continuous animations, attention-stealing micro-motion.
   - Must honor `prefers-reduced-motion` (no exceptions).

## 3) Component Behavior Standards

1. Navigation (sidebar/tabbar):
   - Active state is unmistakable without relying on color alone.
   - Keyboard focus is always visible; focus rings are never removed.
   - Collapsed sidebar must remain fully usable and discoverable.
2. Tables/lists (queue):
   - Row selection is obvious.
   - Risk/priority indicators are consistent, restrained, and deterministic.
   - Empty and degraded states always present an actionable next step.
3. Workbench (draft):
   - The 4-stage funnel is always legible: Intake -> Diagnose -> Draft -> Handoff.
   - Handoff actions (copy/export/Jira) read as “finalization” with explicit gating.
4. AI surfaces:
   - AI readiness is explicit and persistent (model loaded, KB indexed, enrichment ready/offline).
   - “No citation = no claim” is enforced in UX; any override requires a reason and emits an audit event.

## 4) Accessibility and Keyboard (Must Pass)

1. Focus:
   - Every interactive element must have a visible focus state.
   - Do not rely on color alone for focus or active state.
2. Contrast:
   - Text and controls must maintain strong contrast for prolonged reading.
3. Keyboard:
   - All primary workflows are possible without mouse:
     - Queue triage -> open draft -> generate -> handoff -> return to queue.

## 5) Tokenization Rules (Implementation Contract)

1. Revamp scope:
   - Token-first styling is required inside `.app-shell-revamp`.
   - Legacy styles outside revamp may remain, but revamp must not leak legacy ad-hoc values.
2. Allowed patterns:
   - `var(--token)` or `var(--token, safe-fallback)` only.
   - Bridging legacy vars to revamp tokens is allowed inside `.app-shell-revamp` to minimize churn.
3. Disallowed patterns:
   - New `rgba(...)`, `#...`, or `box-shadow: 0 ... rgba(...)` in revamp code.
   - One-off spacing values when a spacing token exists.

## 6) Phase 5 Execution Discipline (How Changes Ship)

1. Work is delivered in slices:
   - Small, coherent diff.
   - Full verification suite.
   - Evidence updated under `docs/revamp/evidence/phase5/ux-*/`.
2. If a slice fails verification:
   - Stop new scope.
   - Fix until green.
   - Re-run the full required suite.
3. If a slice changes information architecture:
   - Freeze cosmetic polishing until IA is stable to avoid rework.

## 7) Reference Implementation Locations (Current Source of Truth)

1. Tokens: `/Users/d/Projects/AssistSupport/src/styles/revamp/tokens.css`
2. Theme + legacy bridges (revamp-scoped): `/Users/d/Projects/AssistSupport/src/styles/revamp/theme.css`
3. Motion + reduced-motion rules: `/Users/d/Projects/AssistSupport/src/styles/revamp/motion.css`
4. Revamp shell gating: `/Users/d/Projects/AssistSupport/src/App.tsx`
5. Flag policy (env authoritative outside dev for policy flags): `/Users/d/Projects/AssistSupport/src/features/revamp/flags.ts`
