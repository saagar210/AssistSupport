---
name: Dependency alert
about: Track dependency vulnerabilities and advisory warnings
labels: dependencies,security
---

## Alert Summary

- Source (`pnpm audit` / `cargo audit` / workflow run URL):
- Date:
- Vulnerability count:
- Advisory warning count:

## Impacted Surface

- [ ] Frontend dependencies
- [ ] Rust dependencies
- [ ] Search API dependencies

## Triage

- Owner:
- Severity:
- Affected package(s):
- Proposed mitigation:
- Target milestone/date:

## Verification Plan

- [ ] Re-run `pnpm audit --audit-level high`
- [ ] Re-run `cd src-tauri && cargo audit`
- [ ] Attach logs/artifacts

## Closure Criteria

- [ ] Mitigation merged
- [ ] Verification commands pass
- [ ] Baseline docs updated if counts changed
