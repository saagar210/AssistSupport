---
name: git-workflow
description: Enforce branch safety, atomic commits, conventional commit messages, PR notes generation, and multi-session autosquash protocol. Trigger for any task that edits files.
---

# Git Workflow Skill

## Objective

Produce senior-level, audit-friendly Git history automatically from normal feature prompts.

## Inputs

- Task summary in plain language.
- Current repo state.
- Base branch (default `master`).

## Hard rules

- Never work on `main` or `master`.
- Branch must match `codex/<type>/<slug>`.
- Every commit must pass commitlint and be atomic by concern.
- Run reviewer/fixer loop before finalizing a logical commit.
- Never bypass failing required gates.

## Procedure

1. Ensure branch:

- Infer type from task (`feat`, `fix`, `refactor`, `chore`, `docs`, `test`, `perf`, `ci`).
- Slugify task to kebab-case.
- Create/switch branch `codex/<type>/<slug>` if needed using `pnpm git:branch:create -- "<task>" <type>`.

2. Plan commit chunks:

- Group changed files by concern.
- Keep each chunk independently testable.

3. Stage one chunk:

- Stage only files in current chunk.
- Reject chunk if generated artifacts or oversized files are staged.

4. Propose commit message:

- Generate one Conventional Commit candidate from staged diff using `pnpm git:commit:propose`.
- Require explicit confirmation before commit.
- Run commitlint on proposed message.

5. Validate chunk:

- Run relevant fast checks.
- Run reviewer/fixer loop:
  - read-only reviewer
  - apply accepted fixes
  - re-review until no P0/P1 findings

6. Commit:

- Create commit with validated message.
- Record commit in session log.

7. Repeat for remaining chunks.

8. Pre-PR cleanup:

- Autosquash fixup commits onto their targets.
- Rebase on latest `origin/master`.

9. PR draft output:

- Generate markdown sections:
  - What
  - Why
  - How
  - Testing
  - Risks
  - Performance impact
  - Lockfile rationale (if needed)
  - Screenshots placeholder (if UI changed)

10. Interruption protocol:

- If session is interrupted, run save/resume scripts and continue from last chunk.

## Outputs

- Branch name.
- Ordered commit list with message + scope.
- PR-ready markdown summary.
- Any unresolved risks/gates.
