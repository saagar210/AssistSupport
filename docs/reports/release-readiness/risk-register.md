# Release Risk Register

Updated: 2026-03-06

| Risk ID | Severity | Area | Description | Mitigation | Owner | Status |
|---|---|---|---|---|---|---|
| R-001 | high | UI parity | `SettingsTab` monolith decomposition could regress hidden validation states. | Golden journey + panel parity assertions + decomposed orchestration hooks validated in journey and lifecycle tests. | eng-1 | done |
| R-002 | high | UI parity | `DraftTab` decomposition could alter queue handoff semantics. | Deterministic replay fixtures + command contract assertions + extracted generation/persistence/diagnosis workflow hooks validated in tests. | eng-1 | done |
| R-003 | medium | governance | Command lifecycle map drift if commands change without sync/validation. | CI lifecycle gate + per-PR lifecycle validation remain mandatory. | eng-1 | done |
| R-004 | medium | reliability | Runtime metrics may silently regress if probe scripts are bypassed. | Scorecard fail-on-missing + baseline capture automation + checkpoint report generation. | eng-1 | done |
| R-005 | medium | operations | SQLite maintenance cadence may remain manual and inconsistent. | Added optimize/checkpoint telemetry + settings diagnostics + startup run-if-due cadence automation (`ASSISTSUPPORT_DB_MAINTENANCE_INTERVAL_HOURS`). | eng-1 | done |
