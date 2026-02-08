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

## Version Bump Required

A version bump is required when any of the following occur:
- field removal
- field rename
- type change
- semantic meaning change of an existing field
- stricter validation that can reject previously accepted payloads

## Changelog Requirements

Each contract-affecting change MUST include:
- previous version
- new version
- compatibility impact
- migration guidance for consumers
