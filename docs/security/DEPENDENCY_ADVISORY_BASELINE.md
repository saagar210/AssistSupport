# Dependency Advisory Baseline

Date: 2026-03-01
Default branch snapshot: `master` @ `affa91affa23caf64c1e48943ccc40813dc38730`

## Purpose

This file is the baseline referenced by `.github/workflows/dependency-watch.yml`.
It records known advisory state, active remediation PRs, and temporary exceptions.

## Current Baseline

### JavaScript (`pnpm audit`)

- Status on `master`: advisories present (2 moderate, 1 low)
- Modules observed:
  - `ajv` (moderate)
  - `lodash` (moderate)
  - `tmp` (low)
- Remediation PR: `#24` (`codex/fix/deps-ajv-6-14-0`)

### Python (`pip-audit -r search-api/requirements.txt`)

- Status on `master`: advisory present
- Module observed:
  - `flask==3.1.2` (`CVE-2026-27205`, fixed in `3.1.3`)
- Remediation PR: `#25` (`codex/fix/deps-flask-3-1-3`)

### Rust (`pnpm run test:security:audit:rust`)

- Status: passing with explicit temporary waivers
- Waiver source: `scripts/security/run-cargo-audit.sh`
- Owner: Platform Engineering
- Tracking issues:
  - Umbrella: `#11`
  - Child: `#12`, `#13`, `#14`, `#15`

## Exit Criteria to Update Baseline

1. Dependency remediation PR merges to `master`.
2. Post-merge audit lane confirms resolved advisory is absent.
3. This file is updated with new baseline date/SHA and residual advisories only.

## Policy

- Dependabot PRs are intake signals and may be superseded by codex-compliant branches.
- Any temporary Rust waiver must include owner, issue, expiry, and rationale.
