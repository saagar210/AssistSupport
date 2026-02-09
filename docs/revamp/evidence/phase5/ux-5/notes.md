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

## Slice: Legacy Typography + Overlay Tokenization
- Bridged legacy `--font-*` vars to revamp fonts in `.app-shell-revamp`.
- Updated Draft readiness borders to use semantic tokens (`--success/--warning/--info`).
- Updated copy override modal overlay/shadow to prefer theme tokens (`--overlay-bg`, `--card-shadow`).

## Slice: Queue Calm Entrance
- Added a single, calm `as-fade-in` entrance animation to the Queue Command Center container (`.as-queue`).
- Reduced-motion is respected via the revamp motion tokens.

## Slice: Draft Revamp Motion + Token Cleanup
- Added a calm mount animation for the revamp draft workbench container only when running inside the revamp shell.
- Swapped the view-toggle active shadow to tokenized `--shadow-xs` with a safe fallback.
- Tokenized diagnosis clear-button hover background to `--bg-hover`.

## Slice: Revamp Shell Tokenization (Nav + Topbar Controls)
- Tokenized revamp shell nav/topbar surfaces to avoid scattered hardcoded rgba values.
- Reused existing semantic tokens (`--card-bg`, `--bg-hover`, `--input-bg`, `--accent-*`) and added two shell-scoped surface tokens in the revamp theme (`--as-shell-nav-bg`, `--as-shell-topbar-bg`) to preserve the glass treatment without ad-hoc CSS.

## Slice: Command Palette Polish (Token + Reduced Motion)
- Tokenized command palette overlay/shadows to reuse existing semantic tokens and avoid hardcoded rgba values.
- Removed undefined CSS variable usage (`--text-tertiary`, `--accent-subtle`, `--accent-color`) in favor of existing semantic tokens.
- Added reduced-motion handling so the palette does not animate for `prefers-reduced-motion`.

## Slice: Queue Command Center Token Cleanup
- Introduced a small set of revamp “glass” + accent surface tokens in `src/styles/revamp/tokens.css`.
- Updated queue command center inputs and selection styling to use those tokens (no raw rgba values).

## Slice: Revamp UI Primitives Token Cleanup
- Added intent/button/panel surface tokens in `src/styles/revamp/tokens.css` to keep primitives consistent without scattering ad-hoc rgba values.
- Updated revamp primitives to be token-only (no raw rgba/hex): buttons, badges, panels, empty states, skeletons.

## Slice: Revamp Settings Shell Coherence
- Added revamp-shell-scoped CSS overrides in `src/components/Settings/SettingsTab.css` so Settings aligns with the ops-console visual language when the revamp shell is enabled.
- No behavior changes: only styling, scoped under `.app-shell-revamp` to keep legacy mode untouched.

## Slice: Revamp Style Guardrail
- Added `pnpm run check:revamp-style` (via `scripts/check_revamp_style.sh`) to prevent reintroducing hardcoded rgba/hex colors in revamp CSS outside token/theme files.
- This is a developer guardrail only; it does not change runtime behavior.

## Slice: Queue Command Center Clarity Polish
- Added explicit “queue unavailable” state with a retry action when draft loading fails (still local-only; no new network surfaces).
- Improved list accessibility and operator flow: listbox semantics, active descendant tracking, and click-to-select without changing keyboard triage behavior.
