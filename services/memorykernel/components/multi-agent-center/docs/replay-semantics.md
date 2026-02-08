# Replay Semantics

Replay reconstructs workflow and step envelopes from trace rows.

- `replay` performs audit replay only (verifies event-chain reconstruction).
- `replay --rerun-provider` loads the original run's normalized workflow snapshot and
  per-step context package snapshots from SQLite, then executes a new run with
  `replay_of_run_id` set to the source run.
- Provider rerun does not re-resolve memory directly when replay snapshots are available.
- Replay run preserves source `as_of` and records a new run manifest hash.
- Trust gate decisions for source runs remain auditable from `step_gate_decisions`; rerun uses the
  replay configuration trust source for fresh execution semantics.
