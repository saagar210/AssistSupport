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

## Platform Support (Current)
VaultMind is treated as a **macOS-first internal tool**. The monorepo readiness suite verifies it builds/tests on the
workstation target used by operators. We do not currently ship or validate a Linux distribution.

Security note:
- A Dependabot alert on `glib` (GHSA-wrw7-89jp-8q8g) was dismissed as `not_used` because the affected dependency only
  appears in `Cargo.lock` via the Linux-only `tauri/gtk` stack and is not present in the macOS dependency graph.
  Reassess this if Linux distribution is added.

## Build/Test From Monorepo Root
VaultMind is intentionally buildable without modifying AssistSupport’s dependencies.

- Lint: `pnpm run vaultmind:lint`
- Test: `pnpm run vaultmind:test`
- Build: `pnpm run vaultmind:build`

## Git Subtree Maintenance (How To Update VaultMind)
VaultMind is vendored into this repository as a **git subtree** rooted at `tools/vaultmind/`.

This gives us:
- A single AssistSupport clone for the workstation.
- Clean history separation (VaultMind history remains in the VaultMind repo).
- Predictable updates (pull latest VaultMind changes into this monorepo when we choose).

### One-Time Setup (Per Workstation Clone)
1. Verify `tools/vaultmind/` exists and is a subtree root.
2. Add a remote for the VaultMind source repository:
   - `git remote add vaultmind <VAULTMIND_REPO_URL>`
   - If it already exists, update it: `git remote set-url vaultmind <VAULTMIND_REPO_URL>`
3. Fetch:
   - `git fetch vaultmind --prune`

### Pull Latest VaultMind Changes Into This Monorepo
Use this when VaultMind advances in its own repo and we want to bring those changes into `tools/vaultmind/` here.

1. Fetch the latest refs:
   - `git fetch vaultmind --prune`
2. Pull the desired branch into the subtree:
   - `git subtree pull --prefix tools/vaultmind vaultmind main --squash`
3. Run the canonical monorepo gate (recommended):
   - `pnpm run check:monorepo-readiness:full`

Notes:
- If VaultMind uses a different default branch (e.g. `master`), replace `main` above.
- We keep subtree pulls squashed to avoid spamming AssistSupport history with VaultMind internal commits.

### Push Subtree Changes Back To VaultMind (Rare)
Use this only if we intentionally edit VaultMind code from inside this monorepo and want to upstream those changes.

1. Ensure your changes are committed locally in this monorepo.
2. Push the subtree prefix back to the VaultMind remote:
   - `git subtree push --prefix tools/vaultmind vaultmind main`

### Common Failure Modes
- “prefix … does not exist”: you’re not in the repo root or the subtree directory moved.
- Conflicts during `subtree pull`: VaultMind diverged. Resolve conflicts, re-run tests, then commit the merge result.
- Accidental Linux build scope: if you are enabling Linux distribution, follow the guardrail guidance in
  `scripts/check_vaultmind_platform_scope.sh` and update this document’s platform scope accordingly.

## When This Boundary Changes
If we introduce a formal “Knowledge Pack” contract (file format + schema) that is validated in CI, add:
1. A versioned spec under `docs/implementation/` or `docs/monorepo/`.
2. A validation script under `scripts/`.
3. A readiness check under `scripts/run_monorepo_readiness.sh full`.
