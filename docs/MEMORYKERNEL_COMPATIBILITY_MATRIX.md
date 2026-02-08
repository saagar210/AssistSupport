# AssistSupport ↔ MemoryKernel Compatibility Matrix

Updated: 2026-02-08

This document defines the immutable consumer-side integration baseline for MemoryKernel service-first integration in AssistSupport.

## Baseline Pin

- MemoryKernel repo: `https://github.com/saagar210/MemoryKernel`
- MemoryKernel release tag: `v0.4.0`
- MemoryKernel commit SHA: `7e4806a34b98e6c06ee33fa9f11499a975e7b922`
- Expected service contract version: `service.v3`
- Expected API contract version: `api.v1`
- Expected integration baseline: `integration/v1`
- Expected producer manifest SHA-256: `79970ec83f37fe0a272cbb62e998ab35628125dbaedaa50c2b9d370d3c53d772`
- Pin manifest (source of truth): `/Users/d/Projects/AssistSupport/config/memorykernel-integration-pin.json`
- Producer manifest mirror: `/Users/d/Projects/AssistSupport/config/memorykernel-producer-manifest.json`

## Approved Version Pairs

| AssistSupport version | AssistSupport commit | MemoryKernel tag | MemoryKernel commit | Service contract | API contract | Integration baseline | Status |
|---|---|---|---|---|---|---|---|
| 1.0.0 | cc6cb15 | v0.4.0 | 7e4806a34b98e6c06ee33fa9f11499a975e7b922 | service.v3 | api.v1 | integration/v1 | Approved |

## Forward Readiness Note

- AssistSupport consumer fallback/error handling supports typed producer error codes (`error.code`) with deterministic legacy fallback behavior.
- Pin and matrix updates must happen in one PR for any future MemoryKernel baseline changes.

## Consumer-Side Enforcement

1. Startup preflight must pass both checks:
   - `GET /v1/health` with exact contract versions.
   - `POST /v1/db/schema-version` successful response.
2. If preflight fails or versions mismatch:
   - MemoryKernel enrichment is disabled.
   - Draft flow remains fully usable (deterministic fallback).
3. MemoryKernel integration tests must pass in CI via:
   - `pnpm run test:memorykernel-contract`
4. Producer error-code governance:
   - Consumer validates producer `error_code_enum` via set-equality (order-independent).
   - Drift fails CI until consumer mapping/tests and governance artifacts are updated together.
5. Producer manifest integrity governance:
   - Consumer validates mirrored manifest SHA-256 against pin metadata.
   - Optional strict mode can validate remote producer manifest hash from pinned release tag (`ASSISTSUPPORT_VALIDATE_REMOTE_MANIFEST=1`).
   - In GitHub Actions, remote validation runs automatically when `MEMORYKERNEL_REPO_READ_TOKEN` is configured.

## Pin Update Policy

When `config/memorykernel-integration-pin.json` changes, the PR must also update:
- `/Users/d/Projects/AssistSupport/docs/MEMORYKERNEL_COMPATIBILITY_MATRIX.md`
- `/Users/d/Projects/AssistSupport/config/memorykernel-producer-manifest.json`

All three governance files are enforced as an atomic update in CI.
