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
        'Cutover-decision checkpoint: `CLOSED` (bilateral decision recorded)',
      ],
    },
    {
      relPath: 'docs/implementation/RUNTIME_CUTOVER_DECISION_RECORD_2026-02-08.md',
      snippets: [
        'Runtime cutover execution: **NO-GO**',
        'Approved Runtime Cutover Window',
        'Status: **Not approved**',
        'Support Platform On-Call Lead',
        'MemoryKernel Producer On-Call Lead',
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
    {
      relPath: 'docs/implementation/SERVICE_V3_CUTOVER_GATES.md',
      snippets: [
        'Required Producer Artifacts',
        'Required Consumer Evidence',
        'Fail-Fast Rollback Conditions',
        'Sign-Off Checklist (AssistSupport + MemoryKernel)',
      ],
    },
    {
      relPath: 'docs/implementation/REMAINING_ROADMAP_EXECUTION_PLAN.md',
      snippets: [
        '## Phase 4: Rehearsal Closure',
        '## Phase 5: Consumer Cutover-Prep Controls',
        '## Phase 6: Cutover Governance + Rollback Readiness',
        'runtime cutover remains disabled',
      ],
    },
    {
      relPath: 'docs/implementation/JOINT_RUNTIME_CUTOVER_GATE_REVIEW_2026-02-08.md',
      snippets: [
        '## Gate Decisions',
        'Runtime cutover: **NO-GO**',
        'Immutable `service.v3` runtime release tag + SHA are not yet published and approved.',
      ],
    },
    {
      relPath: 'docs/implementation/PHASE4_CONSUMER_REHEARSAL_CLOSEOUT_2026-02-08.md',
      snippets: [
        '- Rehearsal continuation verdict: `GO`',
        '- Runtime cutover verdict: `NO-GO`',
      ],
    },
    {
      relPath: 'docs/implementation/PHASE5_CONSUMER_CUTOVER_PREP_2026-02-08.md',
      snippets: [
        'adapter boundary remains enforced',
        'pin + matrix + manifest must be updated atomically in one PR',
        'consumer-side controls complete',
      ],
    },
    {
      relPath: 'docs/implementation/PHASE6_CONSUMER_CUTOVER_GOVERNANCE_2026-02-08.md',
      snippets: [
        'Runtime cutover remains disabled until explicit joint approval',
        'Incident Communication Template',
        'Rollback Success Checklist',
      ],
    },
    {
      relPath: 'docs/implementation/SERVICE_V3_CUTOVER_DAY_DRY_RUN_SESSION_PLAN.md',
      snippets: [
        '## Session Goal',
        '## Joint Agenda (60 minutes)',
        'pnpm run test:memorykernel-cutover-dry-run',
        'Runtime cutover verdict: `NO-GO`',
      ],
    },
    {
      relPath: 'docs/implementation/SERVICE_V3_CUTOVER_DAY_DRY_RUN_EXECUTION_2026-02-08.md',
      snippets: [
        '## Commands Executed',
        'pnpm run test:memorykernel-cutover-dry-run',
        '- Rehearsal continuation: `GO`',
        '- Runtime cutover: `NO-GO`',
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
    'pnpm run check:memorykernel-handoff:service-v3-candidate',
    'pnpm run check:memorykernel-boundary',
    'pnpm run check:memorykernel-cutover-policy',
    'pnpm run test:memorykernel-cutover-dry-run',
    'pnpm run test:memorykernel-contract',
  ]) {
    requireIncludes(operations, 'docs/OPERATIONS.md', snippet);
  }

  console.log('MemoryKernel governance bundle validation passed.');
}

main();
