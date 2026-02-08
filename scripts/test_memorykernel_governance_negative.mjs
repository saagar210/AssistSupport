import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';

const repoRoot = process.cwd();
const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'as-mk-neg-'));

const requiredFiles = [
  'config/memorykernel-integration-pin.json',
  'config/memorykernel-producer-manifest.json',
  'docs/MEMORYKERNEL_COMPATIBILITY_MATRIX.md',
  'docs/OPERATIONS.md',
  'docs/implementation/JOINT_CHECKPOINT_STATUS_2026-02-08.md',
  'docs/implementation/WEEKLY_INTEGRATION_REVIEW_2026-02-08.md',
  'docs/implementation/MEMORYKERNEL_ROLLBACK_DRILL_2026-02-08.md',
  'docs/implementation/MEMORYKERNEL_RUNTIME_DIAGNOSTICS_MATRIX.md',
  'docs/implementation/ACTIVE_RUNTIME_STATUS_2026-02-08.md',
  'docs/implementation/NEXT_EXECUTION_QUEUE.md',
  'docs/implementation/RUNTIME_CUTOVER_DECISION_RECORD_2026-02-08.md',
  'docs/implementation/JOINT_RUNTIME_CUTOVER_GATE_REVIEW_2026-02-08.md',
  'docs/implementation/PHASE_EXECUTION_STATUS_2026-02-08.md',
  'docs/implementation/SERVICE_V3_CUTOVER_DECISION_CHECKPOINT_2026-02-08.md',
  'docs/implementation/MONOREPO_CONSOLIDATION_AUTHORITY_MODEL.md',
  'docs/implementation/MONOREPO_MIGRATION_GATE_CHECKLIST.md',
  'docs/implementation/MONOREPO_OPERATOR_HANDOFF_RUNBOOK.md',
];

function fail(message) {
  console.error(`Governance negative-fixture test failed: ${message}`);
  process.exit(1);
}

function copyFixture(targetRoot) {
  for (const relPath of requiredFiles) {
    const src = path.join(repoRoot, relPath);
    if (!fs.existsSync(src)) {
      fail(`required source file missing: ${relPath}`);
    }
    const dst = path.join(targetRoot, relPath);
    fs.mkdirSync(path.dirname(dst), { recursive: true });
    fs.copyFileSync(src, dst);
  }
}

function runValidator(scriptName, cwd) {
  const scriptPath = path.join(repoRoot, 'scripts', scriptName);
  try {
    execFileSync('node', [scriptPath], {
      cwd,
      stdio: 'pipe',
      env: process.env,
    });
    return { ok: true, output: '' };
  } catch (error) {
    const stdout = error.stdout ? String(error.stdout) : '';
    const stderr = error.stderr ? String(error.stderr) : '';
    return {
      ok: false,
      output: `${stdout}${stderr}`,
    };
  }
}

function expectPass(scriptName, cwd) {
  const result = runValidator(scriptName, cwd);
  if (!result.ok) {
    fail(`${scriptName} expected PASS but failed:\n${result.output}`);
  }
}

function expectFail(scriptName, cwd, expectedSnippet) {
  const result = runValidator(scriptName, cwd);
  if (result.ok) {
    fail(`${scriptName} expected FAIL but passed`);
  }
  if (expectedSnippet && !result.output.includes(expectedSnippet)) {
    fail(
      `${scriptName} failed, but output missing expected snippet \"${expectedSnippet}\"\nOutput:\n${result.output}`
    );
  }
}

function withFixture(name, mutate, assertion) {
  const fixtureRoot = path.join(tempRoot, name);
  fs.mkdirSync(fixtureRoot, { recursive: true });
  copyFixture(fixtureRoot);
  mutate(fixtureRoot);
  assertion(fixtureRoot);
}

function replaceInFile(filePath, from, to) {
  const content = fs.readFileSync(filePath, 'utf8');
  if (!content.includes(from)) {
    fail(`expected text not found in ${filePath}: ${from}`);
  }
  fs.writeFileSync(filePath, content.replace(from, to), 'utf8');
}

function parseJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function writeJson(filePath, data) {
  fs.writeFileSync(filePath, `${JSON.stringify(data, null, 2)}\n`, 'utf8');
}

withFixture(
  'baseline',
  () => {},
  (cwd) => {
    expectPass('validate_memorykernel_pin_sync.mjs', cwd);
    expectPass('validate_memorykernel_governance_bundle.mjs', cwd);
  }
);

withFixture(
  'pin-matrix-drift',
  (cwd) => {
    const matrixPath = path.join(cwd, 'docs', 'MEMORYKERNEL_COMPATIBILITY_MATRIX.md');
    replaceInFile(
      matrixPath,
      '- MemoryKernel release tag: `v0.4.0`',
      '- MemoryKernel release tag: `v9.9.9`'
    );
  },
  (cwd) => {
    expectFail(
      'validate_memorykernel_pin_sync.mjs',
      cwd,
      'compatibility matrix baseline tag does not match pin manifest'
    );
  }
);

withFixture(
  'pin-manifest-drift',
  (cwd) => {
    const manifestPath = path.join(cwd, 'config', 'memorykernel-producer-manifest.json');
    const manifest = parseJson(manifestPath);
    manifest.commit_sha = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
    writeJson(manifestPath, manifest);
  },
  (cwd) => {
    expectFail(
      'validate_memorykernel_pin_sync.mjs',
      cwd,
      'mirrored producer manifest hash mismatch'
    );
  }
);

withFixture(
  'governance-active-status-drift',
  (cwd) => {
    const activeStatusPath = path.join(
      cwd,
      'docs',
      'implementation',
      'ACTIVE_RUNTIME_STATUS_2026-02-08.md'
    );
    replaceInFile(
      activeStatusPath,
      '- MemoryKernel release tag: `v0.4.0`',
      '- MemoryKernel release tag: `v9.9.9`'
    );
  },
  (cwd) => {
    expectFail(
      'validate_memorykernel_governance_bundle.mjs',
      cwd,
      'ACTIVE_RUNTIME_STATUS_2026-02-08.md must include: - MemoryKernel release tag: `v0.4.0`'
    );
  }
);

withFixture(
  'governance-cutover-record-drift',
  (cwd) => {
    const decisionRecordPath = path.join(
      cwd,
      'docs',
      'implementation',
      'RUNTIME_CUTOVER_DECISION_RECORD_2026-02-08.md'
    );
    replaceInFile(
      decisionRecordPath,
      'Runtime cutover execution: **GO**',
      'Runtime cutover execution: **NO-GO**'
    );
  },
  (cwd) => {
    expectFail(
      'validate_memorykernel_governance_bundle.mjs',
      cwd,
      'RUNTIME_CUTOVER_DECISION_RECORD_2026-02-08.md must include: Runtime cutover execution: **GO**'
    );
  }
);

console.log('MemoryKernel governance negative-fixture tests passed.');
