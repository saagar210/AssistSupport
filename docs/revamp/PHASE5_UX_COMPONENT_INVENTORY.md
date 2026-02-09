# Phase 5 UX Component Inventory (Baseline + Foundation Targets)

Status: Active (Phase 5 pre-revamp entry artifact)  
Scope: AssistSupport UI components (legacy + revamp)  
Goal: Identify what we standardize before “pixel work” so the revamp stays coherent.

## 1) Foundation Components (Phase 5 Must Standardize First)

These are the primitives Phase 5 uses to avoid drifting into multiple styles:

1. `Panel`
2. `PanelHeader`
3. `Toolbar`
4. `StatusChip`
5. `SegmentedControl`
6. `Callout`
7. `Card`

Implementation approach:
- We will implement these as revamp-scoped components first (inside `.app-shell-revamp`) so we do not churn legacy CSS.
- Legacy components can be bridged via the existing legacy-token mapping in:
  - `/Users/d/Projects/AssistSupport/src/styles/revamp/theme.css`

## 2) Current Major UI Surfaces (Where Components Live Today)

### App Shell
- Sidebar: `/Users/d/Projects/AssistSupport/src/components/Layout/Sidebar.tsx`
- Header: `/Users/d/Projects/AssistSupport/src/components/Layout/Header.tsx`
- TabBar: `/Users/d/Projects/AssistSupport/src/components/Layout/TabBar.tsx`
- Revamp shell wrapper: `/Users/d/Projects/AssistSupport/src/features/revamp/shell/RevampShell.tsx`

### Draft (Workbench)
- Draft coordinator: `/Users/d/Projects/AssistSupport/src/components/Draft/DraftTab.tsx`
- Intake panel: `/Users/d/Projects/AssistSupport/src/components/Draft/InputPanel.tsx`
- Diagnose panel: `/Users/d/Projects/AssistSupport/src/components/Draft/DiagnosisPanel.tsx`
- Response panel (copy/export/sources): `/Users/d/Projects/AssistSupport/src/components/Draft/ResponsePanel.tsx`
- AI readiness banner: `/Users/d/Projects/AssistSupport/src/components/Draft/AiReadinessBanner.tsx`

### Inbox / Queue
- Entry router: `/Users/d/Projects/AssistSupport/src/features/inbox/InboxPage.tsx`
- Revamp queue: `/Users/d/Projects/AssistSupport/src/features/revamp/screens/QueueCommandCenterPage.tsx`

### Shared UX
- Buttons: `/Users/d/Projects/AssistSupport/src/components/shared/Button.tsx`
- Toasts: `/Users/d/Projects/AssistSupport/src/components/shared/Toast.tsx`
- Command palette: `/Users/d/Projects/AssistSupport/src/components/shared/CommandPalette.tsx`
- Keyboard shortcuts modal: `/Users/d/Projects/AssistSupport/src/components/shared/KeyboardShortcuts.tsx`
- Onboarding: `/Users/d/Projects/AssistSupport/src/components/shared/OnboardingWizard.tsx`

### Settings
- `/Users/d/Projects/AssistSupport/src/components/Settings/SettingsTab.tsx`

## 3) CSS Organization (Reality Check)

Current pattern:
- Many components still have their own `*.css` files (legacy), even when mounted in the revamp shell.
- Revamp shell bridges legacy tokens into revamp tokens to avoid a huge rewrite:
  - `/Users/d/Projects/AssistSupport/src/styles/revamp/theme.css`

Phase 5 discipline:
- New revamp primitives should centralize styling under revamp tokens (avoid scattering new CSS in legacy files).
- Existing legacy component CSS can remain if it reads tokens from the bridge variables.

## 4) Inventory Notes (Where Duplication Is Likely)

Likely duplication hotspots (to confirm during implementation):
1. Panels: multiple `panel-header` styles across Draft/Response/Diagnosis.
2. Status pills: AI readiness vs queue status vs settings status.
3. Segmented controls: response mode toggles vs layout density toggles.

## 5) Acceptance Criteria

1. Phase 5 foundation primitives exist and are used by new revamp screens before additional polishing.
2. There is one “source of truth” for status chips and callouts (no per-page versions).
3. CSS changes remain revamp-scoped unless a bug fix is required.

