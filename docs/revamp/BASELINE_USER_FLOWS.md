# Baseline User Flows (Current System)

Status: Captured  
Date: 2026-02-08

## Purpose
Define the current operational user flows that must remain functional or improve during the revamp.

## Core Actor
IT support engineer operating on macOS laptop, high ticket throughput, keyboard-heavy workflow.

## Flow Set
### F1: Open app and reach operational state
1. Launch app.
2. Verify app initializes without blocking.
3. Confirm user can navigate tabs immediately.

Success criteria:
1. No initialization deadlock.
2. Navigation responsive.

### F2: Draft response from ticket context
1. Load/enter ticket content.
2. Generate response.
3. Review citation/confidence cues.
4. Copy/export output.

Success criteria:
1. Response generation path works.
2. Copy/export actions available.

### F3: Draft load restores ticket context
1. Open existing draft.
2. Confirm ticket context is restored in UI.

Success criteria:
1. Ticket metadata and context present after load.

### F4: MemoryKernel-assisted enrichment
1. Generate response with enrichment enabled.
2. Confirm enriched context appears when service healthy.
3. Confirm fallback path when service unavailable.

Success criteria:
1. Enrichment improves response when available.
2. Fallback deterministic and non-blocking on failure.

### F5: Ops diagnostics and governance checks
1. Open Ops area.
2. Trigger key health/governance checks.
3. Review status signals.

Success criteria:
1. Operator can identify integration state quickly.

### F6: Settings and integration config update
1. Open settings.
2. Change integration configuration.
3. Save and verify validation behavior.

Success criteria:
1. Invalid config blocked.
2. Valid config persists.

### F7: Audit log inspection
1. Open audit logs.
2. Filter by severity.
3. Search for event/message.
4. Page through entries.

Success criteria:
1. Large logs remain navigable.

### F8: Keyboard-first navigation
1. Switch major tabs using shortcuts.
2. Open command palette.
3. Execute common actions from keyboard.

Success criteria:
1. No dead-end requiring mouse for core path.

## Revamp Mapping Requirement
Each replacement surface in revamp must explicitly map to these baseline flows in acceptance tests.
