# Phase 5 UX-2 Evidence (Queue Command Center)

Date: 2026-02-09  
Goal: Rebuild queue triage for speed, clarity, and keyboard-first operation behind revamp flags.

## What Changed

1. New Queue Command Center UI (banded list + sticky preview panel).
2. Preserves keyboard triage contract: J/K move, C claim, X resolve, O reopen, Enter open.
3. Keeps all state local-only (drafts + queue meta in localStorage).

## Flags / Gating

- `VITE_ASSISTSUPPORT_REVAMP_INBOX=1` (queue-first mode)
- Works best alongside `VITE_ASSISTSUPPORT_REVAMP_APP_SHELL=1` (revamp shell), but does not require it.

## Verification

See `verification.txt`.

