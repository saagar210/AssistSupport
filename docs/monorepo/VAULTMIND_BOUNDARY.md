# VaultMind In-Monorepo Boundary (AssistSupport Monorepo)

Status: Active  
Updated: 2026-02-09

## What VaultMind Is (In This Repo)
VaultMind is a separate macOS-first Tauri desktop application that lives in this repository under:

- `tools/vaultmind/`

It is **not** a runtime dependency of AssistSupport. It can fail to build or run without preventing AssistSupport from building or running.

## Why It Lives Here
1. Single-repo workstation setup: one clone onto the work machine.
2. Shared governance: the same standards for local-only posture, safe ingest, and evidence-backed changes.
3. Planned interchange: VaultMind will be the preferred tool to produce and curate “knowledge packs” that AssistSupport can import.

## Non-Negotiables
1. Local-first: no external network traffic by default.
2. No secret material checked in: no tokens/keys/real customer data.
3. Explicit boundaries: changes to VaultMind must not silently change AssistSupport runtime behavior.

## Integration Model (Current)
1. **Filesystem interchange** (preferred):
   - VaultMind exports curated markdown/document bundles to a local folder.
   - AssistSupport ingests that folder via its existing disk ingestion path.
2. **No shared runtime linking**:
   - AssistSupport does not import VaultMind code or binaries.
   - VaultMind does not import AssistSupport code or binaries.

## Build/Test From Monorepo Root
VaultMind is intentionally buildable without modifying AssistSupport’s dependencies.

- Lint: `pnpm run vaultmind:lint`
- Test: `pnpm run vaultmind:test`
- Build: `pnpm run vaultmind:build`

## When This Boundary Changes
If we introduce a formal “Knowledge Pack” contract (file format + schema) that is validated in CI, add:
1. A versioned spec under `docs/implementation/` or `docs/monorepo/`.
2. A validation script under `scripts/`.
3. A readiness check under `scripts/run_monorepo_readiness.sh full`.

