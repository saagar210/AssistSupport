# Monorepo Worktree Topology

## Branches
- `codex/monorepo-migration` (primary migration branch)

## Worktrees
- `/Users/d/Projects/AssistSupport`
  - branch: `codex/monorepo-migration`
  - purpose: canonical integration and commits
- `/Users/d/Projects/as-mono-integration`
  - detached at migration HEAD
  - purpose: sandbox import/path rewiring rehearsal
- `/Users/d/Projects/as-mono-validation`
  - detached at migration HEAD
  - purpose: test-only execution lane
- `/Users/d/Projects/as-mono-recovery`
  - detached at migration HEAD
  - purpose: rollback and recovery operation lane

## Lane Rules
1. All authoritative commits are created from `/Users/d/Projects/AssistSupport`.
2. Validation commands run from `/Users/d/Projects/as-mono-validation` when lane isolation is needed.
3. Recovery lane remains clean unless rollback workflows are required.
4. No commits are made to `master` until all gates pass.

## Notes
Git prevents checking out the same branch in multiple worktrees. Detached validation/recovery worktrees are used intentionally to preserve lane isolation without violating git constraints.
