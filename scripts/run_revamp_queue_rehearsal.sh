#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "[revamp-rehearsal] Running queue/workspace revamp rehearsal checks..."
echo "[revamp-rehearsal] Flags: inbox=1 workspace=1 command_palette_v2=1"

ASSISTSUPPORT_REVAMP_INBOX=1 \
ASSISTSUPPORT_REVAMP_WORKSPACE=1 \
ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2=1 \
pnpm exec vitest run \
  src/features/inbox/queueModel.test.ts \
  src/features/inbox/QueueFirstInboxPage.test.tsx \
  src/features/workspace/WorkspaceQueueContext.test.tsx \
  src/features/workspace/WorkspacePage.test.tsx \
  src/features/app-shell/commands.test.ts

pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance

echo "[revamp-rehearsal] PASS"
