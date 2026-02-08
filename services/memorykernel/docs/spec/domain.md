# Domain Specification (Normative)

## Entity: MemoryRecord

`MemoryRecord` is immutable after insertion.

### Required Fields

- `memory_version_id: ULID` (`MKR-019`, `MKR-028`)
- `memory_id: ULID` (`MKR-019`)
- `version: u32 >= 1`
- `created_at: RFC3339 UTC`
- `effective_at: RFC3339 UTC`
- `truth_status: asserted|observed|inferred|speculative|retracted`
- `authority: authoritative|derived|note`
- `writer: non-empty string`
- `justification: non-empty string`
- `provenance.source_uri: non-empty URI string`
- `payload: typed payload consistent with record type`

### Identity Invariants

- `memory_version_id` identifies one immutable stored version.
- `memory_id` groups versions of the same memory entity.
- `(memory_id, version)` MUST be unique (`MKR-030`).

### Conditional Fields

- `confidence: Option<f32>`
  - MUST be in `[0.0, 1.0]` when present.
  - MUST be present for `inferred` and `speculative` statuses.

### Optional Fields

- `provenance.source_hash: Option<String>` (`sha256:<hex>` if present)
- `provenance.evidence: Vec<String>`

### Lineage

- `supersedes: Vec<memory_version_id>`
- `contradicts: Vec<memory_version_id>`
- Lineage links MUST be append-only records in persistence (`MKR-003`, `MKR-029`).

## Payload Types

- `constraint`
  - `scope.actor: String`
  - `scope.action: String`
  - `scope.resource: String`
  - `effect: allow|deny`
  - `note: Option<String>`
- `decision`
  - `summary: String`
- `preference`
  - `summary: String`
- `event`
  - `summary: String`
- `outcome`
  - `summary: String`

## Forbidden States

- Empty `writer` or `justification` on any write (`MKR-010`)
- Missing `source_uri` (`MKR-004`)
- Invalid enum values (`MKR-023`)
- `version == 0`
- Non-UTC timestamps (`MKR-018`)
- Lineage targeting `memory_id` instead of `memory_version_id` (`MKR-029`)
