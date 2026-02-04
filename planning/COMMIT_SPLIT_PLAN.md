# Commit Split Plan

Use this sequence to produce clean, reviewable commits by feature area.

## Commit 1 — Backend Feature Platform

**Message**
`feat(backend): add ops platform APIs for deployment, eval, triage, runbooks`

**Files**
- `src-tauri/src/db/mod.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/lib.rs`
- `src/types/index.ts`
- `src/hooks/useFeatureOps.ts`

**Intent**
- New DB schema/helpers for deployment artifacts/runs, eval history, triage clusters, runbook sessions.
- New commands for rollback, signed verification, eval history, triage history, and runbook listing.
- Frontend-facing type and hook contracts.

## Commit 2 — Frontend Ops + Trust UX

**Message**
`feat(ui): add operations workspace and trust/grounding UX polish`

**Files**
- `src/App.tsx`
- `src/components/Ops/index.ts`
- `src/components/Ops/OpsTab.tsx`
- `src/components/Ops/OpsTab.css`
- `src/components/Layout/Header.tsx`
- `src/components/Layout/Sidebar.tsx`
- `src/components/Layout/TabBar.tsx`
- `src/components/Draft/DraftTab.tsx`
- `src/components/Draft/ResponsePanel.tsx`
- `src/components/Draft/ResponsePanel.css`
- `src/components/Draft/RatingPanel.tsx`
- `src/components/Analytics/AnalyticsTab.tsx`
- `src/components/Analytics/AnalyticsTab.css`
- `src/components/Settings/SettingsTab.tsx`
- `src/components/Settings/SettingsTab.css`

**Intent**
- Add Ops tab (deployment/eval/triage/runbook/integrations).
- Add toasts, empty states, and rollback confirmation UX.
- Integrate confidence-gated answer cues and source-grounding panels.

## Commit 3 — Test Coverage + Mock Runtime

**Message**
`test(e2e): cover ops workflows with tauri mock support`

**Files**
- `src/test/e2eTauriMock.ts`
- `e2e/ops.spec.ts`
- `e2e/smoke.spec.ts` (if changed)
- `package.json`

**Intent**
- Extend mock Tauri IPC for ops commands with stateful behavior.
- Add Playwright coverage for deployment verification/rollback, eval runs, triage clustering, runbook sessions, and integrations.
- Add dedicated ops e2e script.

## Suggested Command Sequence

```bash
git add src-tauri/src/db/mod.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src/types/index.ts src/hooks/useFeatureOps.ts
git commit -m "feat(backend): add ops platform APIs for deployment, eval, triage, runbooks"

git add src/App.tsx src/components/Ops/index.ts src/components/Ops/OpsTab.tsx src/components/Ops/OpsTab.css src/components/Layout/Header.tsx src/components/Layout/Sidebar.tsx src/components/Layout/TabBar.tsx src/components/Draft/DraftTab.tsx src/components/Draft/ResponsePanel.tsx src/components/Draft/ResponsePanel.css src/components/Draft/RatingPanel.tsx src/components/Analytics/AnalyticsTab.tsx src/components/Analytics/AnalyticsTab.css src/components/Settings/SettingsTab.tsx src/components/Settings/SettingsTab.css
git commit -m "feat(ui): add operations workspace and trust/grounding UX polish"

git add src/test/e2eTauriMock.ts e2e/ops.spec.ts package.json
git commit -m "test(e2e): cover ops workflows with tauri mock support"
```
