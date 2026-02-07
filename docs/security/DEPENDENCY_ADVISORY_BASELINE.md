# Dependency Advisory Baseline

Date: 2026-02-07
Source of truth: `pnpm audit --audit-level high` and `cd src-tauri && cargo audit`

## Current Snapshot

- Frontend (`pnpm audit --audit-level high`): 0 known vulnerabilities.
- Rust (`cargo audit`):
  - Vulnerabilities: 0
  - Warnings: 22
    - Unmaintained: 20
    - Unsound: 2

## Advisory Clusters and Ownership

| Cluster | Example advisory IDs | Current owner | Disposition | Review cadence |
|---|---|---|---|---|
| Tauri/GTK3 Linux transitive stack | RUSTSEC-2024-0411, RUSTSEC-2024-0412, RUSTSEC-2024-0413, RUSTSEC-2024-0414, RUSTSEC-2024-0415, RUSTSEC-2024-0416, RUSTSEC-2024-0418, RUSTSEC-2024-0419, RUSTSEC-2024-0420 | Platform Engineering | Tracked risk, monitor upstream migration path | Weekly dependency watch |
| URLPattern Unicode transitive crates | RUSTSEC-2025-0075, RUSTSEC-2025-0080, RUSTSEC-2025-0081, RUSTSEC-2025-0098, RUSTSEC-2025-0100 | Platform Engineering | Tracked risk, monitor tauri-utils dependency upgrades | Weekly dependency watch |
| Lance/DataFusion/Tantivy transitive crates | RUSTSEC-2024-0384, RUSTSEC-2024-0436, RUSTSEC-2025-0134, RUSTSEC-2026-0002 | Search Platform | Tracked risk, evaluate upgrades and replacement options | Weekly dependency watch |

## Action Policy

1. Any non-zero vulnerability count is actionable and blocks release candidate sign-off.
2. Warnings (unmaintained/unsound) require an open tracking issue with owner and disposition.
3. Baseline document is updated when advisory count or clusters change.
4. Weekly dependency workflow artifacts are retained for 30 days and linked from tracking issues.
