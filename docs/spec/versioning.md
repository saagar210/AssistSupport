# Versioning and Change Policy (Normative)

## Scope

This policy applies to:
- CLI output JSON contracts
- JSON Schemas in `contracts/`
- Service API contracts (OpenAPI) when introduced

## Rules

1. Any backward-incompatible output or schema change MUST bump the contract version.
2. Any additive output or schema change MUST update schema files and fixtures in the same change set.
3. Any contract change MUST include a `CHANGELOG.md` entry.
4. No contract changes are allowed without passing contract compatibility tests.
5. Integration contract changes under `contracts/integration/v1/*` MUST pass cross-repo parity checks.
6. Trilogy compatibility artifacts from sibling repos MUST pass MemoryKernel consumer validation before release.
7. While `service.v2` is active, non-2xx service envelopes MUST preserve `legacy_error` and MUST NOT add `api_contract_version`.

## Version Bump Required

A version bump is required when any of the following occur:
- field removal
- field rename
- type change
- semantic meaning change of an existing field
- stricter validation that can reject previously accepted payloads
- removal of `legacy_error` from service non-2xx envelopes

## Changelog Requirements

Each contract-affecting change MUST include:
- previous version
- new version
- compatibility impact
- migration guidance for consumers

## Current Baselines

- CLI contract: `cli.v1`
- Service contract: `service.v2`
- API envelope contract: `api.v1`

## Service Error Envelope Lifecycle

- `legacy_error` is transitional but mandatory for `service.v2`.
- Consumers may rely on `error.code` as primary and `legacy_error` as compatibility fallback during `service.v2`.
- Any removal of `legacy_error` or addition of `api_contract_version` to non-2xx envelopes requires `service.v3` and explicit migration notes.
