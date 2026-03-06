# Remainder Items Tracker

Updated: 2026-03-06  
Program: AssistSupport remainder-closure stream

| Item | Status | Owner | Target Week | Notes |
|---|---|---|---|---|
| Guardrails frozen as mandatory (contracts, golden journeys, backend gates, pre-push) | done | eng-1 | Week 1 | Landed in CI/pre-push gate chain with backend summary artifacts. |
| Phase-exit criteria codified and enforced in scorecard | done | eng-1 | Week 1 | Runtime metrics now treated as mandatory fail conditions. |
| Command lifecycle/deprecation policy map with validation gate | done | eng-1 | Week 2 | `contracts/tauri/v1/command-lifecycle.json` + CI validator enforced. |
| Lock-across-await policy gate + exception report workflow | done | eng-1 | Week 2 | Clippy lock gate + lock exception report automation enforced. |
| `SettingsTab` compatibility facade over orchestrator/domain panels | done | eng-1 | Week 5 | Added `SettingsModelSection`, maintenance diagnostics panel, operational/audit hooks (`useSettingsOperationalState`, `useSettingsAuditLogs`), and workspace orchestration hook (`useSettingsWorkspaceState`). |
| `DraftTab` compatibility facade over orchestrator/feature modules | done | eng-1 | Week 8 | Added lifecycle reducer + replay fixtures + workflow strip + `DraftPanelsLayout`, diagnosis workflow hook (`useDraftDiagnosisWorkflow`), generation hook (`useDraftGenerationWorkflow`), and persistence/autosave hook (`useDraftPersistenceWorkflow`). |
| Runtime metric capture (`startup`, `generation`, `search`) fully automated | done | eng-1 | Week 10 | Runtime probes and baseline bootstrap now run in `perf:baseline:capture`. |
| DORA snapshot pipeline-fed from CI metadata/deployment records | done | eng-1 | Week 10 | `capture-dora.mjs` now ingests deployment-record artifacts (`DORA_DEPLOYMENT_RECORDS_PATH`) with git fallback and sample accounting. |
| SQLite maintenance telemetry (`PRAGMA optimize`, WAL checkpoint observability) | done | eng-1 | Week 10 | Maintenance commands record optimize/checkpoint timestamps, startup now runs maintenance automatically when cadence is due, and Settings surfaces storage/WAL/cadence diagnostics + run-maintenance action. |
| SearchAPI + MemoryKernel degraded/outage semantics standardized | done | eng-1 | Week 10 | Search health + failure paths now use deterministic degraded classification/guidance; scenario tests expanded for auth/rate/offline/timeout/wrong-service/invalid-config. |
| Weekly architecture + fortnightly release checkpoint automation | done | eng-1 | Week 12 | Generator scripts produce checkpoint artifacts under `docs/reports`. |
| Release-readiness pack (risk register, rollback, residual debt, waived-risk owners) | done | eng-1 | Week 12 | Completed with explicit ownership + sign-off matrix in `docs/reports/release-readiness/final-closeout-report.md`. |

## Phase Exit Criteria

- No missing runtime metrics in program scorecard.
- No unmanaged command lifecycle entries.
- No unreviewed lock-await exceptions.
- No open Sev-1 regressions.
