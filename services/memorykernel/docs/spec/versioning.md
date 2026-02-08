# Versioning and Change Policy (Normative)

## Scope

This policy applies to:
- CLI output JSON contracts
- JSON Schemas in `contracts/`
- Service API contracts (OpenAPI)

## Rules

1. Any backward-incompatible output or schema change MUST bump the contract version.
2. Any additive output or schema change MUST update schema files and fixtures in the same change set.
3. Any contract change MUST include a `CHANGELOG.md` entry.
4. No contract changes are allowed without passing contract compatibility tests.
5. Integration contract changes under `contracts/integration/v1/*` MUST pass cross-repo parity checks.
6. Trilogy compatibility artifacts from sibling repos MUST pass MemoryKernel consumer validation before release.
7. In `service.v3`, non-2xx service envelopes MUST omit both `legacy_error` and `api_contract_version`.
8. Producer baseline metadata in `contracts/integration/v1/producer-contract-manifest.json` MUST be kept current and validated in CI/release gates.

## Version Bump Required

A version bump is required when any of the following occur:
- field removal
- field rename
- type change
- semantic meaning change of an existing field
- stricter validation that can reject previously accepted payloads
- any non-2xx envelope field changes for service responses

## Changelog Requirements

Each contract-affecting change MUST include:
- previous version
- new version
- compatibility impact
- migration guidance for consumers

## Current Baselines

- CLI contract: `cli.v1`
- Service contract: `service.v3`
- API envelope contract: `api.v1`

## Service Error Envelope Lifecycle

- `legacy_error` was transitional for `service.v2` and is removed in `service.v3`.
- Consumers should rely on `error.code` as the canonical machine-readable signal.
- Any future non-2xx envelope field additions/removals require explicit version bump and migration notes.

## Consumer Coordination Policy

- `service.v3` producer baseline SHOULD remain stable for at least one sprint (`14` days).
- Additive `service.v3` error-code changes require:
  - standard notice: `10` business days before release
  - emergency exception: `24` hour notice with same-day docs/spec/tests updates
