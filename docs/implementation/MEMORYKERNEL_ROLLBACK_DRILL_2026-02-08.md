# MemoryKernel Rollback Drill

Updated: 2026-02-08

## Runtime Target
- release_tag: `v0.4.0`
- commit_sha: `7e4806a34b98e6c06ee33fa9f11499a975e7b922`
- service/api/baseline: `service.v3` / `api.v1` / `integration/v1`

## Rollback Target
- release_tag: `v0.3.2`
- commit_sha: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- service/api/baseline: `service.v2` / `api.v1` / `integration/v1`

## Drill Result
- Status: READY
- Result: PASS
- Notes: Consumer fallback and startup preflight behavior remain deterministic after rollback and repin.
