# AssistSupport ↔ MemoryKernel Compatibility Matrix

Updated: 2026-02-08

This document defines the immutable consumer-side integration baseline for MemoryKernel service-first integration in AssistSupport.

## Baseline Pin

- MemoryKernel repo: `https://github.com/saagar210/MemoryKernel`
- MemoryKernel release tag: `v0.3.0`
- MemoryKernel commit SHA: `b9e1b397558dfba1fa8a4948fcf723ed4b505e1c`
- Expected service contract version: `service.v2`
- Expected API contract version: `api.v1`
- Expected integration baseline: `integration/v1`
- Pin manifest (source of truth): `/Users/d/Projects/AssistSupport/config/memorykernel-integration-pin.json`

## Approved Version Pairs

| AssistSupport version | AssistSupport commit | MemoryKernel tag | MemoryKernel commit | Service contract | API contract | Integration baseline | Status |
|---|---|---|---|---|---|---|---|
| 1.0.0 | 60c1d76 | v0.3.0 | b9e1b397558dfba1fa8a4948fcf723ed4b505e1c | service.v2 | api.v1 | integration/v1 | Approved |

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

## Pin Update Policy

When `config/memorykernel-integration-pin.json` changes, the PR must also update this matrix document and pass the MemoryKernel consumer contract test gate.
