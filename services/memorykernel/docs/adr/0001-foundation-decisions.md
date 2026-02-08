# ADR-0001: Foundation Defaults Accepted

## Status
Accepted

## Decisions

1. Conflict policy uses deny-overrides at equivalent precedence.
2. Constraint records use structured scope (`actor`, `action`, `resource`).
3. Superseded records are inactive by default and shown in exclusions.
4. Contradictory records remain active and resolve at retrieval.
5. Truth status precedence: `observed > asserted > inferred > speculative > retracted`.
6. Confidence uses optional `0.0..1.0`; missing treated as `0.5` for ordering only.
7. `source_hash` uses `sha256:<hex>`.
8. Timestamps are UTC RFC3339; query has explicit `as_of`.
9. Every write requires `writer` and `justification`.
10. `effective_at` defaults to `created_at` when not supplied, and this normalization is explicit.
11. Public ids are ULID.
12. v1 query normalization is strict schema-based.
13. Retracted records are excluded from selected by default.
14. Unresolved authoritative-authoritative contradictions return `inconclusive`.
15. Confidence is mandatory for inferred/speculative.
16. Foundation actor scope is single-actor.
17. Export/import portability uses NDJSON + manifest and is implemented in the Foundation codebase.
18. Every stored version has explicit `memory_version_id`; lineage links and link CLI operate on `memory_version_id`.

## Consequences

- The system prioritizes determinism and explainability over fuzzy retrieval.
- CLI remains explicit and strict to prevent silent behavior.
- Entity identity (`memory_id`) and version identity (`memory_version_id`) are intentionally distinct and both required.
