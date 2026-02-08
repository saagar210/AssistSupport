# Trace Schema

Canonical schema bootstrap SQL: `SCHEMA_V2` with migration manifest version `3` in `/Users/d/Projects/MultiAgentCenter/crates/multi-agent-center-trace-sqlite/src/lib.rs`.

Key tables:

- `workflow_snapshots`: normalized workflow JSON + source hashes.
- `runs`: run metadata (`run_id`, `as_of`, status, replay linkage, manifest hash/signature status).
- `steps`: per-step execution state and input/output hashes.
- `trace_events`: append-only event chain with `prev_event_hash` and `event_hash`.
- `step_context_packages`, `step_context_selected`, `step_context_excluded`: injected and excluded
  Context Package snapshots.
- `step_gate_decisions`: policy/trust/human decisions (including memory ref + ruleset/evidence).
  - Contract hardening: trust decisions with `subject_type='memory_ref'` require
    `memory_id`, `version`, and `memory_version_id`.
  - Enforced for new rows at schema level (CHECK + insert trigger).
  - Backward-compatible migration: legacy rows are preserved; enforcement applies to new inserts.
- `provider_calls`: provider metadata, request/response hashes, latency/tokens.
- `proposed_memory_writes`: proposed writes plus apply disposition.
