# Migration Runbook

## Scope

Use this runbook for local schema migration planning and execution.

## Pre-Checks

1. Inspect schema state:
   - `mk db schema-version`
2. Plan migrations without mutation:
   - `mk db migrate --dry-run`
3. Create backup before mutation:
   - `mk db backup --out <path>`

## Apply Migration

1. Execute:
   - `mk db migrate`
2. Validate schema state:
   - `mk db schema-version`
3. Validate data integrity:
   - `mk db integrity-check`

## Rollback Trigger

Rollback if:
- migration command fails mid-process
- integrity check reports violations
- post-migration retrieval produces deterministic regressions

## Rollback Action

1. Restore backup:
   - `mk db restore --in <path>`
2. Re-check:
   - `mk db schema-version`
   - `mk db integrity-check`
