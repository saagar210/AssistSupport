# UX Revamp Input Spec (Locked After Phase 4)

Status: Locked input for Phase 5 UX work  
Audience: Implementers and reviewers  
Goal: Define exactly what the UX revamp must support so Phase 5 is presentation/interaction only (no workflow scope churn).

## Global Product Constraints

1. Single-user local macOS workstations.
2. Internal operational data only; default posture is local/offline-capable.
3. MemoryKernel is optional enrichment only:
   - Adapter boundary only
   - Deterministic fallback preserved
4. Local AI must be predictable:
   - Default recommended model
   - “No citation = no claim” contract
   - Copy gating + audited override

## Primary Workflow (Screens That Must Exist)

### 1) Draft Workbench (`draft`)
Required states:
1. Empty draft
2. Draft with ticket content entered
3. Generation in progress (cancel available)
4. Generation success with:
   - `### OUTPUT`
   - `### IT SUPPORT INSTRUCTIONS`
5. Generation failure (actionable next steps)
6. Degraded AI states (must be explicit, non-blocking):
   - model not loaded
   - embeddings not loaded
   - KB not configured
   - MemoryKernel unavailable

Required controls:
1. Generate
2. Save draft
3. Copy response (gated by citations + confidence mode)
4. Copy override modal (reason required; logs audit event)
5. Export response (same gating policy as copy, or explicit override path)
6. Evidence viewer (sources list + preview)

### 2) Follow-ups / Queue (`followups`)
Required states:
1. Empty inbox
2. Non-empty list with filters/views
3. Open item and load into Draft

Required controls:
1. Open draft
2. Mark/track follow-up status
3. Navigate to queue views (if queue-first revamp flag enabled)

### 3) Sources (`sources`)
Required states:
1. Empty/no KB configured
2. Search results
3. Source preview (chunk view)

Required controls:
1. Search
2. Jump from Draft to Sources with a prefilled query
3. Expand/collapse previews

### 4) Knowledge (`knowledge`)
Required states:
1. Index status view
2. Document browser view
3. Namespace browser view

Required controls:
1. Rebuild/refresh index
2. Browse namespaces and docs
3. Inspect chunks and metadata

### 5) Operations (`ops`)
Required states:
1. Governance checks view
2. Diagnostics view
3. Runbook view
4. Recovery actions view

Required controls:
1. Run preflight validations
2. Export diagnostic evidence (local)
3. View audit logs

### 6) Settings (`settings`)
Required sections:
1. Local model management
   - Recommended model list (single default)
   - Progressive disclosure of other models
   - Custom GGUF loader
2. KB configuration
   - folder path selection
   - embeddings enabled toggle
3. AI Status & Guarantees panel
   - model loaded state
   - embeddings loaded state
   - KB configured + index counts
   - MemoryKernel preflight status
4. Integration configuration
   - Jira config (SSRF-safe validation)
   - Search API token config (if enabled)
   - MemoryKernel token config (if enabled)

## Explicit Non-Goals (Not Required for Phase 5)

1. Network ingestion UX (`ingest` tab) is disabled by default by policy.
2. Admin tabs (`analytics`, `pilot`, `search`) are hidden by default.
3. Stable chunk IDs / source URL / hierarchical sections / reranking are not required for Phase 5 (post-revamp follow-ups).

## Feature Flags That UX Must Respect

1. `VITE_ASSISTSUPPORT_ENABLE_ADMIN_TABS`
2. `VITE_ASSISTSUPPORT_ENABLE_NETWORK_INGEST`
3. Revamp preview toggles:
   - `VITE_ASSISTSUPPORT_REVAMP_WORKSPACE`
   - `VITE_ASSISTSUPPORT_REVAMP_INBOX`
   - `VITE_ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2`

