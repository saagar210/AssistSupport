## UI Hard Gates (Required for frontend/UI changes)

1) Read-only reviewer agent must output `UIFindingV1[]`.
2) Fixer agent must apply findings in severity order: `P0 -> P1 -> P2 -> P3`.
3) Required states per changed UI surface: loading, empty, error, success, disabled, focus-visible.
4) Required pre-done gates:
   - eslint + typecheck + stylelint
   - visual regression (Playwright snapshots)
   - accessibility regression (axe)
   - responsive parity checks (mobile + desktop)
   - Lighthouse CI thresholds
5) Done-state is blocked if any required gate is `fail` or `not-run`.
