import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const pinPath = path.join(root, 'config', 'memorykernel-integration-pin.json');
const cutoverGatesPath = path.join(
  root,
  'docs',
  'implementation',
  'SERVICE_V3_CUTOVER_GATES.md'
);
const phase6ScaffoldPath = path.join(
  root,
  'docs',
  'implementation',
  'PHASE6_CONSUMER_CUTOVER_GOVERNANCE_2026-02-08.md'
);
const phase4CloseoutPath = path.join(
  root,
  'docs',
  'implementation',
  'PHASE4_CONSUMER_REHEARSAL_CLOSEOUT_2026-02-08.md'
);
const handoffPath =
  process.env.MEMORYKERNEL_HANDOFF_PAYLOAD_PATH?.trim() ||
  '/Users/d/Projects/MemoryKernel/docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json';

function fail(message) {
  console.error(`MemoryKernel cutover policy validation failed: ${message}`);
  process.exit(1);
}

function parseJson(filePath) {
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (error) {
    fail(`unable to parse JSON at ${filePath}: ${error.message}`);
  }
}

function requireFile(filePath) {
  if (!fs.existsSync(filePath)) {
    fail(`required file is missing: ${path.relative(root, filePath)}`);
  }
}

function requireIncludes(filePath, snippet) {
  const text = fs.readFileSync(filePath, 'utf8');
  if (!text.includes(snippet)) {
    fail(`${path.relative(root, filePath)} must include: ${snippet}`);
  }
}

function main() {
  requireFile(pinPath);
  requireFile(cutoverGatesPath);
  requireFile(phase6ScaffoldPath);
  requireFile(phase4CloseoutPath);

  const pin = parseJson(pinPath);
  if (pin.expected_service_contract_version !== 'service.v2') {
    fail(
      `runtime baseline must remain service.v2 in this phase; got ${pin.expected_service_contract_version}`
    );
  }
  if (pin.expected_api_contract_version !== 'api.v1') {
    fail(`runtime baseline API contract must remain api.v1; got ${pin.expected_api_contract_version}`);
  }
  if (pin.expected_integration_baseline !== 'integration/v1') {
    fail(
      `runtime baseline integration must remain integration/v1; got ${pin.expected_integration_baseline}`
    );
  }

  requireIncludes(
    cutoverGatesPath,
    'Runtime cutover: requires explicit separate joint approval after rehearsal completion.'
  );
  requireIncludes(phase6ScaffoldPath, 'Runtime cutover remains disabled until explicit joint approval');
  requireIncludes(phase4CloseoutPath, '- Rehearsal continuation verdict: `GO`');
  requireIncludes(phase4CloseoutPath, '- Runtime cutover verdict: `NO-GO`');

  if (fs.existsSync(handoffPath)) {
    const handoff = parseJson(handoffPath);
    if (handoff.handoff_mode === 'service-v3-candidate') {
      if (handoff.expected_service_contract_version !== 'service.v3') {
        fail(
          `candidate handoff expected_service_contract_version must be service.v3, got ${handoff.expected_service_contract_version}`
        );
      }
      if (!handoff.active_runtime_baseline || typeof handoff.active_runtime_baseline !== 'object') {
        fail('candidate handoff must include active_runtime_baseline object');
      }
      if (handoff.active_runtime_baseline.expected_service_contract_version !== 'service.v2') {
        fail(
          `candidate handoff active_runtime_baseline.expected_service_contract_version must be service.v2, got ${handoff.active_runtime_baseline.expected_service_contract_version}`
        );
      }
      if (handoff.rehearsal_candidate?.requires_runtime_cutover !== false) {
        fail('candidate handoff must explicitly set rehearsal_candidate.requires_runtime_cutover=false');
      }
      const requiredCommands = [
        'pnpm run check:memorykernel-handoff:service-v3-candidate',
        'pnpm run check:memorykernel-pin',
        'pnpm run test:memorykernel-contract',
        'pnpm run test:ci',
      ];
      const actualCommands = new Set(handoff.required_consumer_validation_commands || []);
      for (const command of requiredCommands) {
        if (!actualCommands.has(command)) {
          fail(`candidate handoff missing required_consumer_validation_commands entry: ${command}`);
        }
      }
    }
  }

  console.log('MemoryKernel cutover policy validation passed.');
}

main();
