# Phase 4 Feature Inventory (Current Reality)

Scope: AssistSupport UI + local backend (Tauri) as currently implemented.

Goal: Produce a single, repo-grounded inventory of screens, features, and domain objects so we can:
- lock the workflow before UX redesign
- avoid a second redesign
- decide keep/consolidate/remove/defer with minimal ambiguity

## Navigation Surface (Tabs)

Source of truth:
- UI tab type: `src/types/index.ts` (`export type Tab = ...`)
- Shell tab type/order: `src/features/app-shell/types.ts` (`export type TabId = ...`, `TAB_ORDER`)
- Active tab renderer: `src/features/app-shell/renderActiveTab.tsx`
- Sidebar: `src/components/Layout/Sidebar.tsx`
- Mobile tab bar: `src/components/Layout/TabBar.tsx`
- Command palette nav + actions: `src/features/app-shell/commands.ts`

| Tab ID | Label (Sidebar) | Renderer | Primary Purpose | Funnel Stage | Notes |
|---|---|---|---|---|---|
| `draft` | Draft | `WorkspacePage` -> `DraftTab` (or `WorkspaceRevampPage` when flag enabled) | Compose responses | Draft | Canonical workflow home |
| `followups` | Follow-ups | `InboxPage` -> `FollowUpsTab` or `QueueFirstInboxPage` | Saved drafts/history + triage | Handoff (and Ops) | Also functions like an inbox |
| `sources` | Sources | `SourcesPage` | Search KB sources | Intake/Draft support | Likely becomes “Evidence” in Diagnose |
| `ingest` | Ingest | `IngestPage` | Add content to KB | Intake support | Needs evaluation vs Knowledge |
| `knowledge` | Knowledge | `KnowledgePage` | Browse/indexed docs | Intake support | Overlaps with Sources |
| `analytics` | Analytics | `AnalyticsPage` | Quality + usage | Ops | Keep but out-of-band |
| `pilot` | Pilot | `PilotPage` | Feedback dashboard | Ops | Likely consolidate into Analytics/Ops |
| `search` | Search | `SearchPage` | Hybrid Postgres search | Ops/Advanced | Decide if internal-only or remove |
| `ops` | Ops | `OpsPage` | Runbooks/triage/evals | Ops | Keep; may rename |
| `settings` | Settings | `SettingsPage` | Config | Out-of-band | Includes Local AI status panel |

## Command Palette “Feature Placeholders”

Source: `src/features/app-shell/commands.ts`

These currently *toast* “planned but not available” and therefore create expectation debt:
- Templates quick-launch
- Batch processing
- Voice input

Decision needed in Phase 4:
- Either implement minimal versions, or remove/disable these commands until real.

## Domain Objects (Persisted / User-visible)

Source: `src/types/index.ts` and `src-tauri/src/db/*`

| Object | Create/Edit Surface | Used In | Funnel Stage | Notes |
|---|---|---|---|---|
| Draft / SavedDraft | Draft tab, Follow-ups | Handoff, history | Draft/Handoff | Core |
| ResponseTemplate | Draft tab | Draft | Draft | Existence depends on UI exposure |
| DecisionTree | Diagnose panel | Diagnose | Diagnose | Builtin + custom |
| Checklist | Diagnose panel | Diagnose | Diagnose | Generated + updated |
| KB Documents/Chunks | Ingest/Knowledge/Sources | Draft citations | Intake support | KB indexing quality is a known risk |
| Audit entries | Ops/Settings | Ops | Ops | Security/compliance friendly |
| Integrations config (Jira, MemoryKernel, Search API) | Settings/Ops | Draft enrichment/handoff | Out-of-band | Must remain deterministic fallback |

## Immediate Questions (to answer before UX revamp)

1) Which tabs should remain first-class vs moved behind “Advanced/Ops”?
2) Is `followups` primarily “Handoff history” or a true “Inbox/Queue”?
3) Can `sources` + `knowledge` be collapsed into one “Knowledge” experience?
4) Should `search` (hybrid Postgres) remain exposed to IT operators, or be internal tooling?
5) Should placeholder command palette features ship minimally or be removed for now?
