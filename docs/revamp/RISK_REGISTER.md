# Revamp Risk Register

Status: Active  
Date: 2026-02-08

## Risk Severity Scale
1. Critical: material outage/data/security/compliance impact.
2. High: major workflow disruption or integrity risk.
3. Medium: partial degradation with workaround.
4. Low: limited localized impact.

## Active Risks
| ID | Risk | Severity | Trigger Signal | Mitigation | Fallback | Owner |
|---|---|---|---|---|---|---|
| R-001 | Scope expansion during rebuild creates unstable partial states | High | Mid-phase backlog inflates with unscheduled work | Strict in/out scope per phase; defer non-blocking asks | Freeze new asks until next phase gate | Eng Lead |
| R-002 | Integration drift vs MemoryKernel contract artifacts | High | Pin/matrix/manifest mismatch or handoff check failures | Keep governance checks mandatory in CI and pre-merge | Revert to last known-good pin and evidence bundle | Integration Owner |
| R-003 | UX rewrite regresses core IT workflow throughput | High | Workflow acceptance suite failures or increased completion time | Build workflow acceptance tests before replacing surfaces | Feature-flag rollback to stable UI paths | Product + Frontend Owner |
| R-004 | Local LLM quality regressions hidden by visual polish | High | Golden-set score drop or confidence instability | Introduce eval harness and threshold gate | Force fallback model profile and disable risky prompt path | LLM Runtime Owner |
| R-005 | Security regressions introduced by deep refactor | Critical | Security tests fail, new high severity finding | Maintain security tests and threat model updates phase-by-phase | Block merge and revert phase branch | Security Owner |
| R-006 | Rollback path not executable under release pressure | Critical | Rollback drill not performed or undocumented | Mandatory rollback rehearsal per major gate | Halt release candidate progression | Release Owner |
| R-007 | Legacy compatibility code becomes permanent debt | Medium | Temp shims survive beyond phase closure | Require deprecation/removal gate for each shim | Create removal hotfix before next gate | Eng Lead |
| R-008 | Historical docs create conflicting decisions | Medium | Team references stale implementation docs | `docs/revamp/` set as source of truth; ADR supersession policy | Archive stale docs with superseded note | Program Owner |

## Resolved Risks (to be appended during execution)
- None yet in revamp track.

## Risk Review Cadence
1. Update on each phase closeout.
2. Escalate immediately on Critical triggers.
