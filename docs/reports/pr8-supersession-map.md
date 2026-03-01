# PR #8 Supersession Map

Date: 2026-03-01
Source PR: [#8](https://github.com/saagar210/AssistSupport/pull/8)
Decision: **Close as superseded; do not merge branch wholesale**

## Summary

PR #8 is conflicting and stale. Its quality-gate intent is already represented by current workflows and tests on `master`. Remaining deltas are either obsolete, superseded by newer implementations, or not required for current release governance.

## File-Level Mapping

| PR #8 file | Current equivalent on `master` | Action |
| --- | --- | --- |
| `.github/workflows/ui-quality.yml` | `.github/workflows/ci.yml` (`UI Visual + A11y`) | Superseded |
| `.github/workflows/lighthouse.yml` | `.github/workflows/perf-foundation.yml` | Superseded |
| `tests/ui/home.a11y.spec.ts` | `tests/ui/app-shell.spec.ts` | Superseded |
| `tests/ui/home.visual.spec.ts` | `tests/ui/app-shell.spec.ts` + snapshots | Superseded |
| `.codex/verify.commands` (added UI gates) | `.codex/verify.commands` (current) | Superseded |
| `package.json` UI/perf scripts | `package.json` current script set | Superseded |
| `playwright.config.ts` visual baseline tuning | `playwright.config.ts` current | Superseded |
| `src/features/revamp/ui/badge.css` contrast tweak | Current styles and regression gates | Superseded |
| `src/components/Draft/*` incremental UX tweaks | Current component tree | No-port |
| New lint/style config files in PR #8 | Current lint/type/UI gates and CI lanes | No-port |

## Verified Gaps Requiring Port

- None identified as release-blocking in current gate set.

## Closure Action

1. Close PR #8 with link to this map.
2. If future need appears, port specific file-level deltas in a fresh codex branch with current test gates.
