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

function requireUpdatedMarker(text, relPath) {
  const hasUpdatedMarker = /^Updated:\s+\d{4}-\d{2}-\d{2}(\b|$)/m.test(text);
  if (!hasUpdatedMarker) {
    fail(`${relPath} must include an ISO date Updated marker (Updated: YYYY-MM-DD)`);
  }
}

function validateCommonDocs(pin) {
  const checkpoint = readText('docs/implementation/JOINT_CHECKPOINT_STATUS_2026-02-08.md');
  requireUpdatedMarker(checkpoint, 'docs/implementation/JOINT_CHECKPOINT_STATUS_2026-02-08.md');
  requireIncludes(checkpoint, 'docs/implementation/JOINT_CHECKPOINT_STATUS_2026-02-08.md', `- MemoryKernel release tag: \`${pin.release_tag}\``);
  requireIncludes(checkpoint, 'docs/implementation/JOINT_CHECKPOINT_STATUS_2026-02-08.md', `- MemoryKernel commit: \`${pin.commit_sha}\``);

  const weekly = readText('docs/implementation/WEEKLY_INTEGRATION_REVIEW_2026-02-08.md');
  requireUpdatedMarker(weekly, 'docs/implementation/WEEKLY_INTEGRATION_REVIEW_2026-02-08.md');
  requireIncludes(weekly, 'docs/implementation/WEEKLY_INTEGRATION_REVIEW_2026-02-08.md', `- MemoryKernel release tag: \`${pin.release_tag}\``);
  requireIncludes(weekly, 'docs/implementation/WEEKLY_INTEGRATION_REVIEW_2026-02-08.md', `- MemoryKernel commit SHA: \`${pin.commit_sha}\``);

  const rollback = readText('docs/implementation/MEMORYKERNEL_ROLLBACK_DRILL_2026-02-08.md');
  requireUpdatedMarker(rollback, 'docs/implementation/MEMORYKERNEL_ROLLBACK_DRILL_2026-02-08.md');
  requireIncludes(rollback, 'docs/implementation/MEMORYKERNEL_ROLLBACK_DRILL_2026-02-08.md', `- release_tag: \`${pin.release_tag}\``);
  requireIncludes(rollback, 'docs/implementation/MEMORYKERNEL_ROLLBACK_DRILL_2026-02-08.md', `- commit_sha: \`${pin.commit_sha}\``);

  const matrix = readText('docs/MEMORYKERNEL_COMPATIBILITY_MATRIX.md');
  requireUpdatedMarker(matrix, 'docs/MEMORYKERNEL_COMPATIBILITY_MATRIX.md');
  requireIncludes(matrix, 'docs/MEMORYKERNEL_COMPATIBILITY_MATRIX.md', `- MemoryKernel release tag: \`${pin.release_tag}\``);
  requireIncludes(matrix, 'docs/MEMORYKERNEL_COMPATIBILITY_MATRIX.md', `- MemoryKernel commit SHA: \`${pin.commit_sha}\``);
  requireIncludes(matrix, 'docs/MEMORYKERNEL_COMPATIBILITY_MATRIX.md', `- Expected service contract version: \`${pin.expected_service_contract_version}\``);

  const diagnostics = readText('docs/implementation/MEMORYKERNEL_RUNTIME_DIAGNOSTICS_MATRIX.md');
  requireIncludes(
    diagnostics,
    'docs/implementation/MEMORYKERNEL_RUNTIME_DIAGNOSTICS_MATRIX.md',
    'MemoryKernel enrichment is optional and non-blocking.'
  );

  const operations = readText('docs/OPERATIONS.md');
  requireUpdatedMarker(operations, 'docs/OPERATIONS.md');
  for (const snippet of [
    'pnpm run check:memorykernel-pin',
    'pnpm run check:memorykernel-governance',
    'pnpm run check:memorykernel-handoff',
    'pnpm run check:memorykernel-handoff:service-v3-candidate',
    'pnpm run check:memorykernel-boundary',
    'pnpm run check:memorykernel-cutover-policy',
    'pnpm run test:memorykernel-cutover-dry-run',
    'pnpm run test:memorykernel-contract',
    'pnpm run test:memorykernel-governance-negative',
  ]) {
    requireIncludes(operations, 'docs/OPERATIONS.md', snippet);
  }

  const activeStatus = readText('docs/implementation/ACTIVE_RUNTIME_STATUS_2026-02-08.md');
  requireUpdatedMarker(activeStatus, 'docs/implementation/ACTIVE_RUNTIME_STATUS_2026-02-08.md');
  requireIncludes(
    activeStatus,
    'docs/implementation/ACTIVE_RUNTIME_STATUS_2026-02-08.md',
    `- MemoryKernel release tag: \`${pin.release_tag}\``
  );
  requireIncludes(
    activeStatus,
    'docs/implementation/ACTIVE_RUNTIME_STATUS_2026-02-08.md',
    `- MemoryKernel commit SHA: \`${pin.commit_sha}\``
  );
  requireIncludes(
    activeStatus,
    'docs/implementation/ACTIVE_RUNTIME_STATUS_2026-02-08.md',
    `- Service/API/baseline: \`${pin.expected_service_contract_version}\` / \`${pin.expected_api_contract_version}\` / \`${pin.expected_integration_baseline}\``
  );
  requireIncludes(
    activeStatus,
    'docs/implementation/ACTIVE_RUNTIME_STATUS_2026-02-08.md',
    'Canonical producer runtime/service contract source of truth: `/Users/d/Projects/MemoryKernel` on `main`.'
  );

  const queue = readText('docs/implementation/NEXT_EXECUTION_QUEUE.md');
  requireUpdatedMarker(queue, 'docs/implementation/NEXT_EXECUTION_QUEUE.md');

  const authority = readText('docs/implementation/MONOREPO_CONSOLIDATION_AUTHORITY_MODEL.md');
  requireUpdatedMarker(authority, 'docs/implementation/MONOREPO_CONSOLIDATION_AUTHORITY_MODEL.md');
  requireIncludes(
    authority,
    'docs/implementation/MONOREPO_CONSOLIDATION_AUTHORITY_MODEL.md',
    'Atomic Update Rule'
  );

  const migrationChecklist = readText('docs/implementation/MONOREPO_MIGRATION_GATE_CHECKLIST.md');
  requireUpdatedMarker(
    migrationChecklist,
    'docs/implementation/MONOREPO_MIGRATION_GATE_CHECKLIST.md'
  );
  requireIncludes(
    migrationChecklist,
    'docs/implementation/MONOREPO_MIGRATION_GATE_CHECKLIST.md',
    'Gate D: Negative Drift Proof'
  );

  const runbook = readText('docs/implementation/MONOREPO_OPERATOR_HANDOFF_RUNBOOK.md');
  requireUpdatedMarker(runbook, 'docs/implementation/MONOREPO_OPERATOR_HANDOFF_RUNBOOK.md');
  requireIncludes(
    runbook,
    'docs/implementation/MONOREPO_OPERATOR_HANDOFF_RUNBOOK.md',
    'Incident Rollback Workflow'
  );
}

function validatePhaseMode(pin) {
  const decision = readText('docs/implementation/RUNTIME_CUTOVER_DECISION_RECORD_2026-02-08.md');
  const gateReview = readText('docs/implementation/JOINT_RUNTIME_CUTOVER_GATE_REVIEW_2026-02-08.md');
  const phaseStatus = readText('docs/implementation/PHASE_EXECUTION_STATUS_2026-02-08.md');
  const checkpoint = readText('docs/implementation/SERVICE_V3_CUTOVER_DECISION_CHECKPOINT_2026-02-08.md');

  if (pin.expected_service_contract_version === 'service.v2') {
    requireIncludes(decision, 'docs/implementation/RUNTIME_CUTOVER_DECISION_RECORD_2026-02-08.md', 'Runtime cutover execution: **NO-GO**');
    requireIncludes(gateReview, 'docs/implementation/JOINT_RUNTIME_CUTOVER_GATE_REVIEW_2026-02-08.md', 'Runtime cutover: **NO-GO**');
    requireIncludes(phaseStatus, 'docs/implementation/PHASE_EXECUTION_STATUS_2026-02-08.md', 'Phase 8: runtime cutover execution | Joint | Blocked (by policy)');
    requireIncludes(checkpoint, 'docs/implementation/SERVICE_V3_CUTOVER_DECISION_CHECKPOINT_2026-02-08.md', 'Runtime cutover decision: NO-GO');
    return;
  }

  if (pin.expected_service_contract_version === 'service.v3') {
    requireIncludes(decision, 'docs/implementation/RUNTIME_CUTOVER_DECISION_RECORD_2026-02-08.md', 'Runtime cutover execution: **GO**');
    requireIncludes(gateReview, 'docs/implementation/JOINT_RUNTIME_CUTOVER_GATE_REVIEW_2026-02-08.md', 'Runtime cutover: **GO**');
    requireIncludes(phaseStatus, 'docs/implementation/PHASE_EXECUTION_STATUS_2026-02-08.md', 'Phase 8: runtime cutover execution | Joint | Complete');
    requireIncludes(checkpoint, 'docs/implementation/SERVICE_V3_CUTOVER_DECISION_CHECKPOINT_2026-02-08.md', 'Runtime cutover decision: GO');
    return;
  }

  fail(`unsupported expected_service_contract_version: ${pin.expected_service_contract_version}`);
}

function main() {
  if (!fs.existsSync(pinPath)) {
    fail(`missing integration pin: ${pinPath}`);
  }

  const pin = JSON.parse(fs.readFileSync(pinPath, 'utf8'));
  validateCommonDocs(pin);
  validatePhaseMode(pin);

  console.log('MemoryKernel governance bundle validation passed.');
}

main();
