# 0003: App Command Permission Zones

- Status: Accepted
- Date: 2026-03-10

## Context

AssistSupport exposes a large Tauri command surface through a single main window. Before this remediation pass, the repo relied on plugin defaults but did not define app-owned permission zones for its own commands. That created three problems:

1. The command surface was hard to review because there was no explicit authority map.
2. Sensitive areas such as recovery, secret storage, KB mutation, and deployment operations were not grouped in a durable governance artifact.
3. Lower-level engineers had no clear place to update when adding or reclassifying commands.

The app is still single-window, so this ADR is not about pretending we already have perfect least privilege between multiple webviews. It is about making the authority model explicit, reviewable, and ready for future segmentation.

## Decision

- Define app-owned permission groups in `src-tauri/permissions/default.toml`.
- Map the main desktop capability in `src-tauri/capabilities/default.json` to those grouped permissions instead of relying on an implicit all-commands surface.
- Keep the current main window broadly capable enough for the shipped product, but split authority into named zones:
  - startup,
  - diagnostics and recovery,
  - model/vector runtime,
  - knowledge base,
  - drafts/templates,
  - customization/workspace,
  - integrations/secrets,
  - search sidecar,
  - jobs/evals,
  - operations/analytics.
- Keep build-time command enumeration available so future work can tighten or re-scope individual commands without rediscovering the full surface.

## Consequences

### Positive

- The command surface now has an explicit, reviewable authority map.
- Future windows or webviews can reuse these zones instead of rebuilding command lists from scratch.
- Tests can now assert that the manifest covers every registered app command.

### Tradeoffs

- The main window still carries broad authority because the product remains single-window.
- Command additions now require permission-manifest maintenance, which is intentional governance overhead.

## Follow-up

- Revisit finer-grained capabilities if the app adds isolated windows or privileged admin surfaces.
- Keep `src-tauri/tests/permission_manifest.rs` green when commands are added or renamed.
