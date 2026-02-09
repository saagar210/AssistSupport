# Phase 5 UX-1 Evidence (Revamp Shell Foundation)

Date: 2026-02-09  
Goal: Introduce a flag-gated revamp app shell and styling system without changing core workflows.

## What Shipped In UX-1

1. A new revamp shell scaffold (`RevampShell`) that can host existing tab content.
2. A small, offline-first design token set (scoped to `.app-shell-revamp` usage).
3. A visible "AI Status & Guarantees" rail to communicate local-only AI posture and predictability.

## How To Enable

Set:

```bash
VITE_ASSISTSUPPORT_REVAMP_APP_SHELL=1
```

Then start the app normally (`pnpm dev` or the Tauri runner).

## What Must Not Change (UX-1 Contract)

1. Funnel workflow remains Intake -> Diagnose -> Draft -> Handoff.
2. Policy flags remain disabled by default (network ingest, admin tabs).
3. No new external network surfaces are introduced.

## Verification (Local)

```bash
cd /Users/d/Projects/AssistSupport
pnpm install
pnpm run typecheck
pnpm run test
pnpm run test:ci
pnpm run test:e2e:smoke
```

