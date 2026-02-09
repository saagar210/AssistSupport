# Phase 4 Feature Lock Decisions

Status: Locked (pre-UX revamp)  
Audience: Internal IT support engineers (single-user macOS workstations)

Purpose: Make explicit keep/remove/demote decisions **before** the UX rebuild so we do not redesign twice.

## Decisions (Keep / Demote / Disable)

### Operator-primary (keep as first-class)
1. `Draft` (`draft`)
   - Canonical authoring surface.
   - Must remain useful without AI or MemoryKernel.
2. `Follow-ups` (`followups`)
   - Queue/history surface for saved drafts and handoff traceability.
3. `Sources` (`sources`)
   - Evidence search and source preview for citations and grounding.
4. `Knowledge` (`knowledge`)
   - KB browsing, indexing operations, and deeper exploration.
5. `Ops` (`ops`)
   - Runbooks, diagnostics, governance checks, recovery tooling.
6. `Settings` (`settings`)
   - Model management, KB configuration, integration configuration.

### Admin/advanced (demoted behind an explicit opt-in flag)
These remain implemented and accessible for internal leads/debug, but are **not** operator-primary:
1. `Analytics` (`analytics`)
2. `Pilot` (`pilot`)
3. `Search` (`search`)

Enable with:
- `VITE_ASSISTSUPPORT_ENABLE_ADMIN_TABS=1`

Rationale:
- Avoid crowding the primary operator workflow during day-to-day ticket handling.
- Keep the tools available for evaluation and leadership review without requiring a second UX redesign.

### Network ingestion (disabled by policy by default)
The following are intentionally disabled unless explicitly enabled:
1. `Ingest` tab (`ingest`) modes:
   - Web Page (URL)
   - YouTube
   - GitHub remote clone
   - Batch (network sources)

Enable with:
- UI: `VITE_ASSISTSUPPORT_ENABLE_NETWORK_INGEST=1`
- Backend: `ASSISTSUPPORT_ENABLE_NETWORK_INGEST=1`

Rationale:
- Product is offline-first and local-only by default.
- Network ingestion is valuable, but it adds operational dependency and a larger attack surface.
- Keep it available for future use without requiring a future redesign.

## Command Palette Policy (No “Toast Placeholders”)

Decision:
- Remove command palette entries that only display “planned but not available”.

Rationale:
- Placeholder commands create expectation debt and undermine operator trust.
- Phase 5 UX will add real implementations where needed.

## Local AI Model Policy (Single Recommended Default)

Decision:
- Recommended default model: `Llama 3.1 8B Instruct (Q4_K_M)`.
- Other supported models are available only via progressive disclosure.
- Custom GGUF loading remains supported but is treated as “unverified” unless allowlisted (warning + audit).

Rationale:
- Smaller models have shown higher rates of “confidently wrong” behavior in early iterations.
- A single default improves predictability across the team and reduces configuration drift.

## “No Citation = No Claim” and Copy Gating

Decision:
- Copy/export is gated unless:
  - confidence mode = `answer`
  - citations are present
- Override requires a typed reason and logs a local audit event (no response text is logged).

Rationale:
- Operators need a guardrail against ungrounded-but-plausible output.
- Local audit trails support compliance and post-incident learning.

