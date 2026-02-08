# AssistSupport Revamp Program Charter

Status: Approved for Execution  
Date: 2026-02-08  
Program Owner: AssistSupport Engineering  
Execution Mode: Phase-gated, evidence-driven, reversible

## 1) Mission
Rebuild AssistSupport for production-ready internal IT operations with no sacred legacy assumptions, while preserving integration safety with MemoryKernel and maintaining deterministic fallback behavior at all times.

## 2) Program Objectives
1. Replace Draft-first UX with queue-first agent workflow UX.
2. Re-architect frontend/backend boundaries for long-term maintainability.
3. Govern local LLM behavior with explicit model, prompt, and evaluation contracts.
4. Preserve MemoryKernel integration safety boundaries and failure behavior.
5. Exit with a release candidate suitable for work-machine deployment with no major structural work pending.

## 3) Program Scope
### In Scope
1. AssistSupport information architecture and UX rebuild.
2. Frontend module restructuring (feature-sliced boundaries).
3. Tauri command and adapter hardening where required by the rebuild.
4. Local LLM runtime governance and evaluation.
5. Test, security, and compliance evidence refresh required by the rebuild.
6. Work-machine handoff runbooks and release gates.

### Out of Scope
1. MemoryKernel producer runtime behavior changes.
2. Runtime service.v3 cutover without explicit bilateral GO record.
3. Non-essential net-new integrations not required for core IT workflow.

## 4) Success Criteria
1. Core IT support workflows complete in the new UI with keyboard-first paths.
2. All revamp phase gates pass with evidence artifacts checked in.
3. Full canonical test and governance suite passes.
4. Release candidate packet is complete and reversible (rollback drill proven).
5. Work-machine handoff runbook is executable with no major design rework remaining.

## 5) Phase Structure
1. Phase 0: Program Lock and Risk Freeze
2. Phase 1: Baseline Capture and Safety Net
3. Phase 2: Target Architecture and Contracts
4. Phase 3: Foundation Refactor
5. Phase 4: UX Rebuild (Inbox + Workspace)
6. Phase 5: Local LLM Runtime Governance
7. Phase 6: Integration and Ops Hardening
8. Phase 7: Security and Compliance Closure
9. Phase 8: Release Candidate and Work-Machine Handoff

## 6) Execution Policy
1. No phase starts without entry criteria satisfied.
2. No phase closes without exit criteria and evidence artifacts.
3. Any uncertainty is resolved via ADR before implementation proceeds.
4. No irreversible change without tested rollback path.

## 7) Mandatory Invariants
See `docs/revamp/NON_NEGOTIABLES.md`.

## 8) Governance
1. Decision records: `docs/revamp/adr/`.
2. Release gates: `docs/revamp/RELEASE_GATES.md`.
3. Risks and mitigations: `docs/revamp/RISK_REGISTER.md`.
4. Baseline and evidence: `docs/revamp/evidence/`.

## 9) Current Program State
1. Program charter approved.
2. Execution started with Phase 0 + Phase 1 artifacts.
3. Runtime cutover posture remains controlled by explicit bilateral decision record.
