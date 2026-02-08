# Resolver Specification (Normative)

## Input Contract

Resolver input is:
- `records[]` snapshot (each with `memory_id` and `memory_version_id`)
- `query` normalized to `{actor, action, resource}`
- `as_of` UTC timestamp
- deterministic `snapshot_id`

## Candidate Selection

1. Include only `constraint` records for policy query (`MKR-011`).
2. Scope match rules:
   - Exact field match contributes specificity score.
   - `*` is wildcard and contributes zero specificity.
   - Non-wildcard mismatch rejects candidate.
3. Exclude from selected set:
   - `truth_status = retracted` (`MKR-015`)
   - records superseded by active links targeting their `memory_version_id` (`MKR-016`, `MKR-029`)
4. Excluded records MUST be emitted with explicit reasons.

### Recall Query Candidate Selection

1. Include records whose `record_type` is in the explicit recall scope. If scope is omitted, use `decision|preference|event|outcome` (`MKR-044`).
2. Normalize query text into deterministic lowercase alphanumeric terms.
3. Exclude from selected set:
   - `truth_status = retracted` (`MKR-045`)
   - records superseded by active links targeting their `memory_version_id` (`MKR-045`)
   - records with zero lexical overlap against query terms (`MKR-045`)
4. Excluded records MUST be emitted with explicit reasons.

## Deterministic Ordering Tuple

Sort descending by:
1. Scope specificity score
2. Authority rank (`authoritative > derived > note`)
3. Truth rank (`observed > asserted > inferred > speculative > retracted`)
4. Confidence (`None` treated as `0.5` for ordering only)
5. `effective_at`
6. `created_at`

Final tie-break chain:
7. `memory_id` ascending
8. `memory_version_id` ascending (`MKR-014`, `MKR-028`)

### Recall Ordering Tuple

Sort descending by:
1. Lexical match count against normalized query terms
2. Authority rank (`authoritative > derived > note`)
3. Truth rank (`observed > asserted > inferred > speculative > retracted`)
4. Confidence (`None` treated as `0.5` for ordering only)
5. `effective_at`
6. `created_at`

Final tie-break chain:
7. `memory_id` ascending
8. `memory_version_id` ascending (`MKR-046`)

## Answer Derivation

Given top-precedence effective candidates:
- only allow -> `allow`
- only deny -> `deny`
- allow and deny -> `inconclusive` (`MKR-017`)
- none -> `inconclusive`

Resolver conflict grouping MUST use `memory_version_id` identity for candidate discrimination.

## Determinism Rules

- All operations MUST be pure over input snapshot.
- Sorting comparator MUST be total and stable.
- Output MUST include `ruleset_version` and `tie_breakers` (`MKR-021`).
- Recall query comparators and scope defaults MUST be deterministic (`MKR-044`, `MKR-046`).
