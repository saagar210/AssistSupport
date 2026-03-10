# Dependency Advisory Triage Runbook

## Goal

Handle JS, Python, Rust, and workflow dependency alerts in a repeatable way without normalizing long-lived waivers.

## Primary commands

```bash
pnpm audit --audit-level high
python3 -m pip_audit -r search-api/requirements.txt
pnpm run test:security:audit:rust
```

## First-pass decision tree

1. Confirm the alert is present on the current branch.
2. Decide whether the fix is:
   - direct dependency upgrade,
   - transitive override/pin,
   - or temporary waiver with owner and expiry.
3. Prefer removing the vulnerable dependency from the graph over extending a waiver.

## Rust-specific policy

- Treat new high-severity `cargo audit` findings as release blockers.
- If an upstream chain prevents immediate removal:
  - document the exact advisory ID,
  - assign an owner,
  - set a review date,
  - add the rationale to `scripts/security/run-cargo-audit.sh`,
  - and update `docs/security/DEPENDENCY_ADVISORY_BASELINE.md`.

## Python and JS policy

- Regenerate lockfiles only when required by the fix.
- Update runtime and test dependencies together when the search API is affected.
- Re-run smoke or readiness checks after dependency changes, not just unit tests.

## Workflow dependency policy

- Pin third-party GitHub Actions to full SHAs.
- Use Dependabot to track GitHub Actions updates.
- Avoid drifting toolchain versions between CI workflows without an explicit reason.

## Verification checklist

- Required audit command is green or the temporary waiver is documented.
- Runtime or smoke checks still pass after the dependency change.
- Baseline docs reflect the new advisory state.
- The remediation plan file records the discovery and final disposition.
