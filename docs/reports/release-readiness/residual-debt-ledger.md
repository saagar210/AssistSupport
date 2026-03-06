# Residual Debt Ledger

Updated: 2026-03-06

| Debt ID | Area | Description | Impact | Target Removal Window | Owner | Status |
|---|---|---|---|---|---|---|
| D-001 | frontend/settings | `SettingsTab.tsx` compatibility-heavy orchestration decomposed into `useSettingsOperationalState`, `useSettingsAuditLogs`, and `useSettingsWorkspaceState`. | Medium maintainability drag and regression risk. | Weeks 3-5 | eng-1 | done |
| D-002 | frontend/draft | `DraftTab.tsx` orchestration decomposed into `useDraftDiagnosisWorkflow`, `useDraftGenerationWorkflow`, and `useDraftPersistenceWorkflow`. | High complexity + slower feature iteration. | Weeks 5-8 | eng-1 | done |
| D-003 | ops/sqlite | Maintenance observability and cadence automation for WAL checkpoints/optimize completed (including startup run-if-due policy). | Medium reliability and diagnostics gap. | Weeks 8-10 | eng-1 | done |
| D-004 | degraded-mode UX | SearchAPI/MemoryKernel fallback messaging not fully standardized end-to-end. | Medium user guidance inconsistency during outages. | Weeks 8-10 | eng-1 | done |
