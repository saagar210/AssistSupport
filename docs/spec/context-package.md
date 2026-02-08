# Context Package Specification (Normative)

## Artifact Contract

A Context Package is the only supported retrieval artifact in v1 (`MKR-008`).
When emitted by CLI commands, the output MUST additionally include top-level
`contract_version` per `docs/spec/cli-contract.md`.

Context Packages are produced for both:
- policy queries (`query ask`)
- mixed-record recall queries (`query recall`)

## Required Top-Level Fields

- `context_package_id: String`
- `generated_at: RFC3339 UTC`
- `query`
- `determinism`
- `answer`
- `selected_items[]`
- `excluded_items[]`
- `ordering_trace[]`

## determinism Object

- `ruleset_version: String`
- `snapshot_id: String`
- `tie_breakers: String[]`

## selected_items[] / excluded_items[]

Each item MUST include:
- `rank`
- `memory_version_id`
- `memory_id`
- `record_type`
- `version`
- `truth_status`
- `confidence`
- `authority`
- `why`

`why` MUST include:
- `included: bool`
- `reasons: String[]`
- `rule_scores` for selected records

For recall queries, exclusion reasons MUST include explicit cause strings for
retracted/superseded/non-overlap filtering (`MKR-045`).

## Serialization

- JSON output MUST be stable (`MKR-020`).
- Determinism metadata MUST always be present (`MKR-021`).
- `memory_version_id` MUST be present for all selected/excluded items (`MKR-028`).
