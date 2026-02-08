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
const decisionRecordPath = path.join(
  root,
  'docs',
  'implementation',
  'RUNTIME_CUTOVER_DECISION_RECORD_2026-02-08.md'
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

function assertArrayContainsAll(values, expected, field) {
  const valueSet = new Set(Array.isArray(values) ? values : []);
  const missing = expected.filter((entry) => !valueSet.has(entry));
  if (missing.length > 0) {
    fail(`${field} missing required entries: ${missing.join(',')}`);
  }
}

function validatePreCutoverPolicy(pin) {
  if (pin.expected_service_contract_version !== 'service.v2') {
    fail(
      `pre-cutover mode requires service.v2 baseline; got ${pin.expected_service_contract_version}`
    );
  }
  requireIncludes(
    cutoverGatesPath,
    'Runtime cutover: requires explicit separate joint approval after rehearsal completion.'
  );
  requireIncludes(
    phase6ScaffoldPath,
    'Runtime cutover remains disabled until explicit joint approval'
  );
  requireIncludes(phase4CloseoutPath, '- Rehearsal continuation verdict: `GO`');
  requireIncludes(phase4CloseoutPath, '- Runtime cutover verdict: `NO-GO`');
  requireIncludes(decisionRecordPath, 'Runtime cutover execution: **NO-GO**');
}

function validatePostCutoverPolicy(pin) {
  if (pin.expected_service_contract_version !== 'service.v3') {
    fail(
      `post-cutover mode requires service.v3 baseline; got ${pin.expected_service_contract_version}`
    );
  }
  requireIncludes(cutoverGatesPath, 'service.v3');
  requireIncludes(phase4CloseoutPath, '- Rehearsal continuation verdict: `GO`');
  requireIncludes(decisionRecordPath, 'Runtime cutover execution: **GO**');
}

function validateHandoff(pin) {
  if (!fs.existsSync(handoffPath)) {
    return;
  }

  const handoff = parseJson(handoffPath);
  if (handoff.release_tag !== pin.release_tag) {
    fail(`handoff release_tag mismatch; expected ${pin.release_tag}, got ${handoff.release_tag}`);
  }
  if (handoff.commit_sha !== pin.commit_sha) {
    fail(`handoff commit_sha mismatch; expected ${pin.commit_sha}, got ${handoff.commit_sha}`);
  }
  if (handoff.expected_api_contract_version !== pin.expected_api_contract_version) {
    fail(
      `handoff expected_api_contract_version mismatch; expected ${pin.expected_api_contract_version}, got ${handoff.expected_api_contract_version}`
    );
  }

  if (handoff.handoff_mode === 'service-v3-candidate') {
    if (handoff.expected_service_contract_version !== 'service.v3') {
      fail(
        `candidate handoff expected_service_contract_version must be service.v3, got ${handoff.expected_service_contract_version}`
      );
    }
    if (!handoff.active_runtime_baseline || typeof handoff.active_runtime_baseline !== 'object') {
      fail('candidate handoff must include active_runtime_baseline object');
    }
    if (
      handoff.active_runtime_baseline.expected_service_contract_version !==
      pin.expected_service_contract_version
    ) {
      fail(
        'candidate handoff active_runtime_baseline.expected_service_contract_version must match current pin'
      );
    }
    if (handoff.rehearsal_candidate?.requires_runtime_cutover !== false) {
      fail('candidate handoff must set rehearsal_candidate.requires_runtime_cutover=false');
    }
    assertArrayContainsAll(
      handoff.required_consumer_validation_commands,
      [
        'pnpm run check:memorykernel-handoff:service-v3-candidate',
        'pnpm run check:memorykernel-pin',
        'pnpm run test:memorykernel-contract',
        'pnpm run test:ci',
      ],
      'required_consumer_validation_commands'
    );
    return;
  }

  if (handoff.handoff_mode !== 'stable') {
    fail(`unsupported handoff_mode: ${handoff.handoff_mode}`);
  }

  if (handoff.expected_service_contract_version !== pin.expected_service_contract_version) {
    fail(
      `stable handoff expected_service_contract_version mismatch; expected ${pin.expected_service_contract_version}, got ${handoff.expected_service_contract_version}`
    );
  }

  const stablePolicy = handoff.non_2xx_envelope_policy?.service_v3_stable;
  if (!stablePolicy || typeof stablePolicy !== 'object') {
    fail('stable handoff must include non_2xx_envelope_policy.service_v3_stable');
  }
  assertArrayContainsAll(
    stablePolicy.requires,
    ['service_contract_version', 'error.code', 'error.message'],
    'non_2xx_envelope_policy.service_v3_stable.requires'
  );
  assertArrayContainsAll(
    stablePolicy.forbids,
    ['legacy_error', 'api_contract_version'],
    'non_2xx_envelope_policy.service_v3_stable.forbids'
  );
}

function main() {
  requireFile(pinPath);
  requireFile(cutoverGatesPath);
  requireFile(phase6ScaffoldPath);
  requireFile(phase4CloseoutPath);
  requireFile(decisionRecordPath);

  const pin = parseJson(pinPath);
  if (pin.expected_api_contract_version !== 'api.v1') {
    fail(`runtime baseline API contract must remain api.v1; got ${pin.expected_api_contract_version}`);
  }
  if (pin.expected_integration_baseline !== 'integration/v1') {
    fail(
      `runtime baseline integration must remain integration/v1; got ${pin.expected_integration_baseline}`
    );
  }

  if (pin.expected_service_contract_version === 'service.v2') {
    validatePreCutoverPolicy(pin);
  } else if (pin.expected_service_contract_version === 'service.v3') {
    validatePostCutoverPolicy(pin);
  } else {
    fail(`unsupported expected_service_contract_version: ${pin.expected_service_contract_version}`);
  }

  validateHandoff(pin);

  console.log('MemoryKernel cutover policy validation passed.');
}

main();
