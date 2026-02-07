# Dependency Triage Policy

## Scope

This policy governs dependency risk handling for:
- Frontend package dependencies (`pnpm audit`)
- Rust dependencies (`cargo audit`)
- Search API Python dependencies (`pip` + runtime smoke checks)

## Severity Handling

1. Critical/High vulnerabilities
- Owner assignment within one business day.
- Fix or mitigation merged before release candidate sign-off.
- Release blocked until resolved or explicitly risk-accepted in writing.

2. Medium/Low vulnerabilities
- Owner assignment within three business days.
- Scheduled remediation in the nearest sprint.

3. Advisory warnings (unmaintained/unsound)
- Must have active tracking issue and owner.
- Re-evaluate weekly with dependency-watch artifacts.
- Document upgrade path or containment rationale.

## Ownership Model

- Platform Engineering: Tauri/runtime/system dependency surface.
- Search Platform: Search API and vector/search stack dependencies.
- Application Engineering: Frontend dependency surface.

## Verification Gates

- `pnpm audit --audit-level high`
- `cd src-tauri && cargo audit`
- Weekly `.github/workflows/dependency-watch.yml`

## Release Gate

Release candidate cannot be approved when either condition is true:
- actionable vulnerability count > 0
- dependency alerts lack an assigned owner/disposition
