# Performance Baselines

- Baselines are generated from `pnpm perf:*` outputs and used by CI compare gates.
- Update baselines only when a performance shift is intentional.
- PRs that update baselines should include `perf-baseline-update` context in the PR body and reviewer sign-off.
