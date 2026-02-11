# Decisions

## 2026-02-11 — Queue metadata normalization before use
- **Decision:** Add explicit runtime normalization for persisted queue metadata entries before they are consumed by queue calculations.
- **Why:** Existing `loadQueueMeta` accepted any parsed object shape, allowing invalid `priority/state/owner` values from corrupted localStorage to skew summaries.
- **Alternatives considered:**
  - Keep current fallback behavior and rely on tests (rejected: no guard against malformed persisted objects).
  - Introduce external schema validator dependency (rejected: unnecessary weight for small local shape).
- **Impact:** Preserves current storage contract while improving resilience and deterministic behavior.

## 2026-02-11 — Preserve original valid timestamp formatting
- **Decision:** Normalize `updatedAt` validity without reformatting valid timestamps to canonical ISO milliseconds.
- **Why:** Existing behavior and tests assume persisted strings are preserved when valid; formatting-only mutation created unnecessary diff/noise.
- **Alternative rejected:** Always serialize with `toISOString()` (rejected due to avoidable compatibility churn).
