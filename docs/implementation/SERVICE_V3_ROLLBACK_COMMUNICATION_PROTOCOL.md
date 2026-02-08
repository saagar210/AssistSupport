# Service.v3 Rollback Communication Protocol (Producer)

Updated: 2026-02-08  
Scope: Communication and evidence protocol for rehearsal/cutover incidents.

## Purpose
Standardize how producer and consumer coordinate rollback decisions with minimal ambiguity and complete evidence traceability.

## Severity Classes
1. `SEV-1`: user-facing correctness or reliability failure with no immediate safe workaround.
2. `SEV-2`: contract/validation drift that can lead to incorrect integration behavior.
3. `SEV-3`: non-critical issue with safe fallback and no contract break.

## Trigger-to-Action Matrix
| Trigger | Minimum severity | Required action |
|---|---|---|
| Non-2xx envelope shape mismatch | SEV-1 | Immediate rollback to approved service.v2 baseline |
| Consumer deterministic fallback regression | SEV-1 | Immediate rollback + joint incident bridge |
| Candidate payload field drift | SEV-2 | Freeze promotion, remediate payload, re-verify |
| Missing release evidence artifact | SEV-2 | No-go for cutover until evidence is complete |

## Notification Timeline
1. Initial incident notice: within 15 minutes of trigger identification.
2. Producer rollback recommendation: within 30 minutes.
3. Consumer re-pin confirmation: within 30 minutes after rollback execution.
4. Joint incident summary: within 24 hours.

## Required Incident Notice Template
1. Incident ID
2. Trigger type
3. Affected contract mode (`service.v2` stable or `service.v3` candidate/release)
4. Current and target rollback tag/sha
5. Impact summary
6. Immediate actions
7. Next update ETA

## Required Reversal Evidence Bundle
1. Producer command outputs proving restored baseline:
   - alignment
   - parity/artifacts
   - smoke
2. Consumer command outputs proving restored compatibility:
   - handoff/pin checks
   - contract tests
   - CI aggregate
3. Final decision record:
   - rollback complete timestamp (UTC)
   - owner approvals
   - residual risk assessment

## Go/No-Go Communication Protocol
1. Producer sends proposed decision packet with evidence links.
2. AssistSupport confirms status against agreed acceptance criteria.
3. Joint decision explicitly declared as:
   - `GO (rehearsal continuation)`
   - `GO (runtime cutover)`
   - `NO-GO` (with remediation plan)
4. Decision log is stored in both repos' implementation docs.
