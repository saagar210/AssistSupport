import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

const root = process.cwd();
const evidencePath =
  process.env.ASSISTSUPPORT_PHASE6_OPS_EVIDENCE_PATH ??
  'docs/revamp/evidence/PHASE6_OPS_HARDENING_LATEST.json';

const checks = [
  'pnpm run check:memorykernel-governance',
  'pnpm run check:memorykernel-handoff',
  'pnpm run check:memorykernel-handoff:service-v3-candidate',
  'pnpm run check:llm-golden-set',
  'pnpm run check:rollback-readiness',
  'pnpm run test:security-regression',
];

function runCheck(command) {
  const start = Date.now();
  const result = spawnSync(command, {
    cwd: root,
    shell: true,
    stdio: 'inherit',
    env: process.env,
  });
  const durationMs = Date.now() - start;
  return {
    command,
    duration_ms: durationMs,
    exit_code: result.status ?? 1,
    passed: result.status === 0,
  };
}

function writeEvidence(relPath, payload) {
  const absPath = path.join(root, relPath);
  fs.mkdirSync(path.dirname(absPath), { recursive: true });
  fs.writeFileSync(absPath, `${JSON.stringify(payload, null, 2)}\n`, 'utf8');
}

function main() {
  const startedAt = new Date().toISOString();
  const results = [];
  let failed = false;

  for (const command of checks) {
    const result = runCheck(command);
    results.push(result);
    if (!result.passed) {
      failed = true;
      break;
    }
  }

  const payload = {
    executed_at: startedAt,
    completed_at: new Date().toISOString(),
    total_checks: checks.length,
    completed_checks: results.length,
    passed_checks: results.filter((result) => result.passed).length,
    failed: failed,
    checks: results,
  };
  writeEvidence(evidencePath, payload);
  console.log(`Wrote Phase 6 ops hardening evidence: ${path.join(root, evidencePath)}`);

  if (failed) {
    process.exit(1);
  }
}

main();
