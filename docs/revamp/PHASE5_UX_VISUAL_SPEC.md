# Phase 5 UX Visual Spec (Apple-Native + Vibrancy Default)

Status: Active (Phase 5 pre-revamp entry artifact)  
Scope: Revamp surfaces only (inside `.app-shell-revamp`)  
Audience: Operators on macOS workstations

This spec defines the visual rules that Phase 5 must implement. It is intentionally concrete:
token names, allowed patterns, and required fallbacks are all explicit.

## 1) Theme Direction

1. Apple-native: calm, precise, subtle depth, long-session friendly.
2. Vibrancy default: translucent surfaces + blur provide depth without heavy borders.
3. Fallback required: reduced transparency disables blur and uses solid surfaces.

## 2) Token Sources of Truth

1. Revamp token primitives:
   - `/Users/d/Projects/AssistSupport/src/styles/revamp/tokens.css`
2. Revamp theme + legacy bridge:
   - `/Users/d/Projects/AssistSupport/src/styles/revamp/theme.css`
3. Revamp motion + reduced motion:
   - `/Users/d/Projects/AssistSupport/src/styles/revamp/motion.css`

## 3) Typography Rules

1. End-state fonts (Apple-native):
   - Sans: `system-ui`
   - Mono: `ui-monospace` / `SF Mono` stack
2. Phase 5 transition rule:
   - Existing bundled fonts may remain temporarily, but any new revamp component must use tokenized font variables so we can switch later without churn.

## 4) Vibrancy Rules (Default)

### Required behavior

1. Revamp shell background uses layered gradients (already present) plus glass surfaces for nav/topbar/panels.
2. Glass surfaces must not use ad-hoc rgba values in component CSS; they must come from tokens:
   - `--as-glass-1`, `--as-glass-2`, `--as-glass-3`
   - `--as-shell-nav-bg`, `--as-shell-topbar-bg`
3. Blur is controlled by a single variable:
   - `--backdrop-blur`

### Allowed CSS pattern

```css
backdrop-filter: blur(var(--backdrop-blur)) saturate(1.15);
background: var(--as-shell-topbar-bg);
border: 1px solid var(--as-border-1);
```

## 5) Reduced Transparency Fallback (Required)

When the OS requests reduced transparency, revamp surfaces must become solid and blur must be disabled.

### Required media query

```css
@media (prefers-reduced-transparency: reduce) {
  .app-shell-revamp {
    --backdrop-blur: 0px;
    --as-shell-nav-bg: var(--as-surface-2);
    --as-shell-topbar-bg: var(--as-surface-2);
    --as-glass-1: var(--as-surface-2);
    --as-glass-2: var(--as-surface-2);
    --as-glass-3: var(--as-surface-3);
  }
}
```

## 6) Motion Rules

1. Only calm transitions are allowed; no bounce curves.
2. Reduced motion must set durations to 0:
   - Evidence: `/Users/d/Projects/AssistSupport/src/styles/revamp/motion.css`

## 7) Component Shape + Depth

1. Radii: only use revamp radius tokens:
   - `--as-radius-1`, `--as-radius-2`, `--as-radius-pill`
2. Shadows: only `--as-shadow-1` (and legacy bridge tokens when needed).
3. Borders: default is subtle; emphasize only for meaning (focus/selection/error).

## 8) Implementation Discipline

1. All new Phase 5 revamp styling must be token-first.
2. If a component needs a new visual value (spacing/color/blur), add a token and document it here.

