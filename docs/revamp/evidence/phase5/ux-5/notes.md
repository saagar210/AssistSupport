# Phase 5 UX-5 Evidence: Revamp Visual System Bridge

## What Changed
- Added a "legacy token bridge" inside the revamp shell so components that still reference the older `--bg-*`, `--text-*`, `--border-*`, `--accent-*` tokens render coherently in the graphite revamp theme.
- Added reduced-motion support for revamp motion tokens.

## Why
- Phase 5 is intentionally incremental. The revamp shell already has its own token system (`--as-*`). Bridging the old tokens in-shell prevents a large mechanical rewrite and keeps UI consistent while we continue migrating screens.

## Invariants Preserved
- No workflow changes.
- No new network surfaces.
- No changes to MemoryKernel contract behavior.

## Follow-ups
- As more components migrate to `--as-*` tokens directly, we can gradually reduce reliance on the bridged legacy variables.
