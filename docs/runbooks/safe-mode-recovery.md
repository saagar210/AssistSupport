# Safe Mode Recovery Runbook

## When to use this runbook

Use this when AssistSupport starts in recovery mode or when normal startup fails with a database integrity or migration conflict message.

## Recovery mode means

- The app detected a workspace problem before normal startup completed.
- The database was not trusted enough to continue with the normal shell.
- You must choose between repair, restore, or manual conflict resolution.

## Before you start

- Stop any other AssistSupport instance that may still be running.
- Do not manually edit the SQLite DB, WAL, or SHM files unless directed by a deeper incident response.
- If you have a recent backup ZIP, keep it available before attempting restore.

## Step 1: Read the recovery reason

Check the recovery screen summary and note whether the cause is:

- integrity or corruption failure,
- foreign-key consistency failure,
- migration conflict,
- or a restore-required startup issue.

## Step 2: Choose the least destructive path

Prefer paths in this order:

1. `Repair database` when the issue is integrity-related and the app offers repair.
2. `Restore from backup` when repair fails or the workspace is known-bad.
3. Manual conflict resolution only when the issue is a migration conflict that requires data review.

## Step 3: If you attempt repair

- Run the in-app repair action first.
- After repair, use `Retry startup`.
- If the app exits recovery mode, immediately export a fresh backup.
- If recovery mode returns, switch to restore instead of repeating repair indefinitely.

## Step 4: If you restore from backup

- Use a known-good backup ZIP.
- Remember the shipped backup scope:
  - drafts,
  - templates,
  - variables,
  - trees,
  - settings,
  - KB folder configuration.
- The backup does not restore:
  - local KB source files,
  - vector-store contents,
  - downloaded models,
  - external service tokens unless explicitly backed by the secure store outside the ZIP.
- The restore path writes into a fresh encrypted database and archives the previous DB files before swap-over.

## Step 5: If the issue is a migration conflict

- Do not keep retrying startup.
- Capture the conflict summary from the recovery screen.
- Preserve the affected workspace files.
- Open the migration logic referenced in the summary and resolve the conflicting records deliberately.
- Re-run startup only after the conflicting state is understood.

## Verification

Recovery is complete only when:

- AssistSupport exits recovery mode,
- `Settings > Data Backup` can export successfully,
- `Run Quick Health Check` reports the database as healthy,
- and critical data (drafts/templates/settings) appears intact.

## Escalate when

- Repair fails twice with the same integrity result.
- Backup restore fails on a known-good archive.
- Migration conflict details are unclear or point to data loss risk.
- Recovery succeeds but the app immediately re-enters recovery mode on next launch.
