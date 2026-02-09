# Phase 4 Closeout (Workflow + Feature Lock)

Date: 2026-02-09  
Status: CLOSED (ready for Phase 5 UX execution)  
Final SHA: `6251ec4`

## Objective (Phase 4)

Lock AssistSupport’s workflow shape and feature boundaries before the UX rebuild so Phase 5 is presentation/interaction work only.

## What Was Delivered

### Workflow + scope lock
1. Canonical workflow definition: `/Users/d/Projects/AssistSupport/docs/revamp/FEATURE_LOCK_WORKFLOW.md`
2. Local AI predictability contract: `/Users/d/Projects/AssistSupport/docs/revamp/LOCAL_AI_CONTRACT.md`
3. Current-feature inventory: `/Users/d/Projects/AssistSupport/docs/revamp/PHASE4_FEATURE_INVENTORY.md`
4. Explicit lock decisions (keep/demote/disable): `/Users/d/Projects/AssistSupport/docs/revamp/PHASE4_FEATURE_LOCK_DECISIONS.md`
5. UX revamp input spec (what Phase 5 must support): `/Users/d/Projects/AssistSupport/docs/revamp/UX_REVAMP_INPUT_SPEC.md`

### Operator trust controls (local AI)
1. Recommended default model policy (Llama 3.1 8B Instruct Q4_K_M), with progressive disclosure of other models.
2. Copy gating (“no citation = no claim”):
   - Copy requires citations + confidence mode `answer`.
   - Override requires a reason and creates a local audit entry (no response text logged).
3. AI status surfaced in Settings and Draft readiness banner.

### Security posture (local-only + offline-first)
1. Network ingestion is disabled by policy unless explicitly enabled:
   - Backend gate: `ASSISTSUPPORT_ENABLE_NETWORK_INGEST=1`
   - UI gate: `VITE_ASSISTSUPPORT_ENABLE_NETWORK_INGEST=1`
2. Jira base URL configuration is SSRF-validated with DNS pinning.
3. Backup ZIP import is bounded (entry count, entry size, total size) and detects URL-encoded traversal.
4. Local services (Search API / MemoryKernel service) moved toward “secure-by-default”:
   - explicit auth tokens supported
   - loopback bind enforcement where applicable

### Threat model artifact
1. `/Users/d/Projects/AssistSupport/AssistSupport-threat-model.md`

## Deferred (Explicitly Not Blocking Phase 5)

Knowledge ingestion quality improvements that are desirable but not required for the UX rebuild:
1. Stable chunk IDs across re-ingest
2. Hierarchical section breadcrumbs in chunk metadata
3. Preserve source URL/page title metadata
4. Optional reranking step

These are tracked in revamp planning docs and should be scheduled after Phase 5 if accuracy demands it.

## Verification (All PASS)

1. `pnpm run typecheck`
2. `pnpm run test` (128 tests)
3. `pnpm run test:ci`
4. `pnpm run test:memorykernel-contract` (writes evidence JSON)
5. `cd src-tauri && cargo test`
6. `cd src-tauri && cargo clippy -- -D warnings`
7. `python -m pytest search-api/tests -q` (25 tests)
8. `cd services/memorykernel && cargo test`

## Phase 5 Entry Criteria

Phase 5 UX may begin when:
1. This closeout file exists on the mainline branch.
2. All verification commands above remain green after any Phase 5 changes.
3. Phase 5 work does not introduce workflow changes outside `/Users/d/Projects/AssistSupport/docs/revamp/UX_REVAMP_INPUT_SPEC.md` without explicit exception approval.

