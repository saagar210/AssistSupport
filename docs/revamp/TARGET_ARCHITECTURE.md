# Target Architecture (AssistSupport Revamp)

Status: Drafted for Phase 2 Gate  
Date: 2026-02-08

## 1) Architectural Goal
Move from tab-centric, mixed-responsibility code to a queue-first, domain-bounded architecture that supports high-throughput IT support workflows and safe MemoryKernel coexistence.

## 2) Top-Level Domains
1. `app-shell`: global navigation, layout, command palette, cross-domain shortcuts.
2. `inbox`: queue list, SLA indicators, triage, assignment actions.
3. `workspace`: ticket context, diagnosis, response composition, action rail.
4. `knowledge`: source management, search diagnostics, KB quality feedback.
5. `ops`: deployment checks, runtime diagnostics, operational workflows.
6. `settings`: app configuration and integration controls.
7. `shared-ui`: reusable components, tokens, interaction primitives.
8. `integration-memorykernel`: all MemoryKernel transport/contracts/fallback handling.
9. `integration-jira`: ticket fetching/updating, transition actions.
10. `llm-runtime`: model routing, prompt contracts, confidence policy.

## 3) Layering Rules
1. UI components may call only domain hooks/services.
2. Domain services may call only adapter interfaces.
3. Adapters own all transport concerns and external contract handling.
4. No cross-domain imports except through documented shared interfaces.

## 4) Data Flow (Target)
1. `inbox` selects work item.
2. `workspace` resolves ticket with local context + optional enrichment.
3. `llm-runtime` generates with governed prompt/model contracts.
4. `integration-*` adapters execute side effects.
5. `ops` and audit capture runtime evidence.

## 5) MemoryKernel Boundary (Unchanged Invariant)
1. Single adapter boundary in Tauri backend.
2. All preflight, query, timeout, and error envelope logic isolated there.
3. Deterministic fallback preserved regardless of enrichment state.

## 6) Legacy Replacement Strategy
1. Strangler pattern: new feature modules introduced behind flags.
2. Legacy components removed only after flow-level parity acceptance.
3. Compatibility shims require explicit removal ticket before phase closure.

## 7) Observability Requirements
1. Structured audit events for key user actions and integration faults.
2. Explicit runtime state values (`healthy`, `degraded`, `disabled`, `offline`).
3. Diagnostics visible in Ops and exportable for review.
