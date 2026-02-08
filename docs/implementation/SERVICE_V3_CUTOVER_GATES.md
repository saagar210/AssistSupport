# Service.v3 Cutover Gates

Updated: 2026-02-08

## Baseline
- release_tag: `v0.4.0`
- commit_sha: `7e4806a34b98e6c06ee33fa9f11499a975e7b922`
- service_contract_version: `service.v3`
- api_contract_version: `api.v1`

## Required Producer Artifacts
1. Producer manifest aligned to runtime baseline.
2. Stable producer handoff payload (`handoff_mode=stable`).
3. Green producer verification suite evidence.

## Required Consumer Evidence
1. Atomic pin + matrix + mirrored manifest update.
2. Green contract suite and full CI.
3. Updated runtime decision record and checkpoint packet.
4. Rollback drill evidence against current runtime baseline.

## Non-2xx Envelope Policy (`service.v3`)
- Required fields: `service_contract_version`, `error.code`, `error.message`
- Optional fields: `error.details`
- Forbidden fields: `legacy_error`, `api_contract_version`

## Fail-Fast Rollback Conditions
Rollback immediately to last approved baseline (`v0.3.2` / `cf331449...`) if any condition is true:
1. Contract mismatch or schema drift.
2. Deterministic fallback regression.
3. Preflight no longer passes against intended runtime target.
4. Any CI gate failure in cutover validation scope.

## Sign-Off Checklist (AssistSupport + MemoryKernel)
- [x] Joint GO/NO-GO documented for rehearsal entry.
- [x] Joint GO/NO-GO documented for runtime cutover.
- [x] Bilateral decision records published.
- [x] Runtime baseline promotion recorded.
