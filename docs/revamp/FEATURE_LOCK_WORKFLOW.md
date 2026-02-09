# Feature Lock Workflow (Phase 4)

Status: Active (pre-UX revamp)  
Audience: Internal IT support engineers (macOS, single-user local workstation)  
Purpose: Lock the **operator workflow** and **feature boundaries** before the UX rebuild so we do not redesign twice.

## Non-Negotiables

1. Local-only operation: internal operational data stays on-device.
2. Optional AI: the app must remain useful when AI is disabled or unavailable.
3. Deterministic fallback: MemoryKernel enrichment is never required for core ticket flow.
4. Single integration boundary: MemoryKernel calls only through the adapter layer.
5. Operator clarity: the UI must always answer "What should I do next?"

## Primary Workflow (Four-Stage Funnel)

The product is organized around one canonical loop:

1. **Intake**
   - Goal: capture ticket context and normalize it into a draftable working set.
   - Inputs: ticket text, metadata, screenshots/attachments (optional), Jira context (optional).
   - Output: a clean “case snapshot” the operator can trust.

2. **Diagnose**
   - Goal: gather evidence and decide what’s true.
   - Tools: Knowledge Base search, decision trees/playbooks, optional MemoryKernel enrichment.
   - Output: cited evidence list + operator notes (what was checked, what remains unknown).

3. **Draft**
   - Goal: produce a ready-to-send response plus internal instructions.
   - Rules: citations required for factual claims; uncertainty must be explicit.
   - Output: response package:
     - End-user message (copy/send)
     - IT support instructions (internal steps)
     - Evidence/citations

4. **Handoff**
   - Goal: close the loop with explicit next steps and traceability.
   - Actions: export/copy response, update Jira (optional), log quality signals, record follow-ups.
   - Output: handoff packet (what happened, what’s next, who owns it).

## Feature Boundary Mapping

Every feature must belong to exactly one stage:

1. Intake
   - Ticket paste/import
   - Quick context capture (screenshots OCR if enabled)
2. Diagnose
   - KB search + source previews
   - MemoryKernel enrichment (optional, non-blocking)
   - Decision trees / checklists
3. Draft
   - Generate / alternative generation
   - Edit, templates/macros
   - Copy gating + copy override audit
4. Handoff
   - Export (md/txt/html)
   - Jira post/update (if configured)
   - Ratings / quality signals

## What Must Not Happen (Guardrails)

1. No feature should require a second UX redesign after Phase 5 starts.
2. No "hidden" AI behavior: operators must see what the AI used and what it did not.
3. No network ingestion feature ships enabled by default:
   - `Ingest` (URL/YouTube/GitHub remote) remains **disabled by policy** unless explicitly opt-in.
   - Enable only via `ASSISTSUPPORT_ENABLE_NETWORK_INGEST=1` (backend) and `VITE_ASSISTSUPPORT_ENABLE_NETWORK_INGEST=1` (UI).
4. No new ingestion surface without:
   - path traversal protection
   - archive bomb limits
   - auditability

## Verification Gate (Must Stay Green)

Before Phase 5 UX begins, these must pass:

1. `pnpm run typecheck`
2. `pnpm run test`
3. `pnpm run test:memorykernel-contract`
4. `pnpm run test:ci`
5. `cd src-tauri && cargo test`
6. `cd src-tauri && cargo clippy -- -D warnings`
