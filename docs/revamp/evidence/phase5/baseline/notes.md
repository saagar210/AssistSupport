# Phase 5 Baseline Snapshot

Date: 2026-02-09  
Purpose: Establish a pre-revamp reference point for Phase 5 rollout evidence.

## What This Baseline Captures

1. Current operator shell and navigation (legacy).
2. Current Draft workbench flow and degraded-mode behavior.
3. Current queue / follow-ups flow.

## How To Reproduce Locally

1. Run smoke E2E:

```bash
cd /Users/d/Projects/AssistSupport
pnpm run test:e2e:smoke
```

2. Validate core checks:

```bash
pnpm run typecheck
pnpm run test
pnpm run test:ci
```

## Notes

- Phase 5 revamp work is feature-flagged and can be enabled via `VITE_ASSISTSUPPORT_REVAMP_APP_SHELL=1`.
- Network ingestion and admin tabs remain policy-gated and disabled by default.

