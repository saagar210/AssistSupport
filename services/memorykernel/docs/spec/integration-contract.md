# Integration Contract (Normative)

Cross-project integration contracts are versioned under:

- `contracts/integration/v1/schemas/`
- `contracts/integration/v1/fixtures/`

## v1 Shared Artifacts

- `context-package-envelope.schema.json`
- `trust-gate-attachment.schema.json`
- `proposed-memory-write.schema.json`
- `error-envelope.schema.json`
- `producer-contract-manifest.json`

MemoryKernel is the canonical source for these v1 artifacts.

## v1 Compatibility Rules

- Consumers MUST treat these schemas as additive-forward contracts.
- Removing required fields requires a version bump.
- New required fields require a version bump.
- Fixture updates that change semantic meaning require a version bump.
- Cross-repo parity checks MUST fail when file sets differ or file contents differ.
- Schema identity metadata MUST use valid `$id` values in every integration schema.

## Consumer Scope

- MultiAgentCenter uses `context-package-envelope`, `trust-gate-attachment`, and `proposed-memory-write`.
- OutcomeMemory emits trust/gate outputs that MUST remain compatible with `trust-gate-attachment` semantics.
- AssistSupport pins to `producer-contract-manifest.json` and validates pin + matrix + manifest sync in CI.

## Producer Manifest Requirements (`producer-contract-manifest.v1`)

- Path: `contracts/integration/v1/producer-contract-manifest.json`
- MUST include:
  - immutable producer baseline (`release_tag`, `commit_sha`)
  - expected service/api versions for consumer preflight
  - integration baseline (`integration/v1`)
  - canonical `service.v2` `error_code_enum`
  - stability window and additive-code notice policy
  - service.v3 transition guardrails for `legacy_error` lifecycle

## Operational Verification

Run from MemoryKernel root:

```bash
./scripts/verify_contract_parity.sh
./scripts/verify_trilogy_compatibility_artifacts.sh
./scripts/verify_producer_contract_manifest.sh
```
