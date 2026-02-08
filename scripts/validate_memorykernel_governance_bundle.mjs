import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const pinPath = path.join(root, 'config', 'memorykernel-integration-pin.json');

function fail(message) {
  console.error(`MemoryKernel governance bundle validation failed: ${message}`);
  process.exit(1);
}

function readText(relPath) {
  const absPath = path.join(root, relPath);
  if (!fs.existsSync(absPath)) {
    fail(`missing required file: ${relPath}`);
  }
  return fs.readFileSync(absPath, 'utf8');
}

function requireIncludes(text, relPath, snippet) {
  if (!text.includes(snippet)) {
    fail(`${relPath} must include: ${snippet}`);
  }
}

function main() {
  if (!fs.existsSync(pinPath)) {
    fail(`missing integration pin: ${pinPath}`);
  }

  const pin = JSON.parse(fs.readFileSync(pinPath, 'utf8'));
  const requiredDocs = [
    {
      relPath: 'docs/implementation/JOINT_CHECKPOINT_STATUS_2026-02-08.md',
      snippets: [
        `- MemoryKernel release tag: \`${pin.release_tag}\``,
        `- MemoryKernel commit: \`${pin.commit_sha}\``,
        'Checkpoint A (manifest mirrored + governance checks): `GO`',
        'Checkpoint B (consumer contract suite green): `GO`',
      ],
    },
    {
      relPath: 'docs/implementation/WEEKLY_INTEGRATION_REVIEW_2026-02-08.md',
      snippets: [
        `- MemoryKernel release tag: \`${pin.release_tag}\``,
        `- MemoryKernel commit SHA: \`${pin.commit_sha}\``,
        '- [x] Governance checks green in both repos.',
      ],
    },
    {
      relPath: 'docs/implementation/MEMORYKERNEL_ROLLBACK_DRILL_2026-02-08.md',
      snippets: [
        `- release_tag: \`${pin.release_tag}\``,
        `- commit_sha: \`${pin.commit_sha}\``,
        '- Status: READY',
      ],
    },
    {
      relPath: 'docs/implementation/MEMORYKERNEL_RUNTIME_DIAGNOSTICS_MATRIX.md',
      snippets: [
        'Draft flow must stay available under all MemoryKernel failure modes.',
        'MemoryKernel enrichment is optional and non-blocking.',
        '| `degraded` |',
      ],
    },
    {
      relPath: 'docs/implementation/SERVICE_V3_CONSUMER_REHEARSAL_PLAN.md',
      snippets: [
        'Planning-only rehearsal for service.v3 adoption.',
        'Validate deterministic fallback for offline/timeout/malformed/version-mismatch/non-2xx.',
      ],
    },
    {
      relPath: 'docs/implementation/SERVICE_V3_REHEARSAL_EXECUTION_TRACKER.md',
      snippets: [
        '## Phase Execution Tasks',
        'pnpm run test:memorykernel-contract',
      ],
    },
    {
      relPath: 'docs/implementation/PHASE_EXECUTION_STATUS_2026-02-08.md',
      snippets: [
        `- MemoryKernel release tag: \`${pin.release_tag}\``,
        `- MemoryKernel commit SHA: \`${pin.commit_sha}\``,
        '| Phase 2: Consumer runtime hardening + observability |',
      ],
    },
    {
      relPath: 'docs/implementation/PHASE1_STEADY_STATE_CLOSEOUT_2026-02-08.md',
      snippets: [
        `- MemoryKernel release tag: \`${pin.release_tag}\``,
        `- MemoryKernel commit SHA: \`${pin.commit_sha}\``,
        'Phase 1 steady-state execution closeout (consumer scope): `COMPLETE`',
      ],
    },
    {
      relPath: 'docs/implementation/PHASE3_CONSUMER_DRY_RUN_2026-02-08.md',
      snippets: [
        `- MemoryKernel release tag: \`${pin.release_tag}\``,
        `- MemoryKernel commit SHA: \`${pin.commit_sha}\``,
        'pnpm run check:memorykernel-handoff',
      ],
    },
  ];

  for (const doc of requiredDocs) {
    const text = readText(doc.relPath);
    for (const snippet of doc.snippets) {
      requireIncludes(text, doc.relPath, snippet);
    }
  }

  const operations = readText('docs/OPERATIONS.md');
  for (const snippet of [
    '## 2.1) MemoryKernel Runtime Lifecycle',
    'pnpm run check:memorykernel-pin',
    'pnpm run check:memorykernel-governance',
    'pnpm run check:memorykernel-handoff',
    'pnpm run test:memorykernel-contract',
  ]) {
    requireIncludes(operations, 'docs/OPERATIONS.md', snippet);
  }

  console.log('MemoryKernel governance bundle validation passed.');
}

main();
