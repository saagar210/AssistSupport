# Rollback Checklist

Updated: 2026-03-06

1. Confirm rollback trigger and incident severity with owner on call.
2. Freeze merges to `master` and communicate rollback window.
3. Revert latest release commit set using non-interactive `git revert` sequence.
4. Re-run mandatory gates:
   - `pnpm run ci:backend:gates:all`
   - `pnpm run check:contract-fixtures`
   - `pnpm run test:e2e:journey`
5. Validate command lifecycle map still matches command registry post-revert.
6. Validate lock policy gate passes and exception report is generated.
7. Publish rollback status update with impacted features and expected follow-up.
8. Create follow-up issue for root cause and prevention actions.
