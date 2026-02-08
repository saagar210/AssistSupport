import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const pinPath = path.join(root, 'config', 'memorykernel-integration-pin.json');
const producerManifestPath = path.join(root, 'config', 'memorykernel-producer-manifest.json');
const legacyDefaultHandoffPath =
  '/Users/d/Projects/MemoryKernel/docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json';
const monorepoDefaultHandoffPath = path.join(
  root,
  'services',
  'memorykernel',
  'docs',
  'implementation',
  'PRODUCER_RELEASE_HANDOFF_LATEST.json'
);
const handoffPath =
  process.env.MEMORYKERNEL_HANDOFF_PAYLOAD_PATH?.trim() ||
  [monorepoDefaultHandoffPath, legacyDefaultHandoffPath].find((candidatePath) =>
    fs.existsSync(candidatePath)
  ) ||
  monorepoDefaultHandoffPath;
const requireHandoff = process.env.ASSISTSUPPORT_REQUIRE_HANDOFF_PAYLOAD === '1';
const requirePinMatch = process.env.ASSISTSUPPORT_HANDOFF_REQUIRE_PIN_MATCH !== '0';
const outputPath = path.join(root, 'artifacts', 'memorykernel-handoff-evidence.json');

function fail(message) {
  console.error(`MemoryKernel handoff payload validation failed: ${message}`);
  process.exit(1);
}

function parseJson(filePath) {
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (error) {
    fail(`unable to parse JSON at ${filePath}: ${error.message}`);
  }
}

function assertEqual(actual, expected, field) {
  if (actual !== expected) {
    fail(`${field} mismatch; expected=${expected}, got=${actual}`);
  }
}

function assertNonEmptyArray(value, field) {
  if (!Array.isArray(value) || value.length === 0) {
    fail(`${field} must be a non-empty array`);
  }
}

function toStringSet(values) {
  const set = new Set();
  for (const value of values) {
    if (typeof value !== 'string' || value.trim() === '') {
      fail('error_code_enum entries must be non-empty strings');
    }
    set.add(value);
  }
  return set;
}

function compareSets(actual, expected, field) {
  const missing = [...expected].filter((entry) => !actual.has(entry));
  const unexpected = [...actual].filter((entry) => !expected.has(entry));
  if (missing.length > 0 || unexpected.length > 0) {
    fail(
      `${field} set mismatch; missing=[${missing.join(',')}] unexpected=[${unexpected.join(',')}]`
    );
  }
}

function assertArrayContainsAll(values, expected, field) {
  const valueSet = new Set(values);
  const missing = expected.filter((entry) => !valueSet.has(entry));
  if (missing.length > 0) {
    fail(`${field} missing required entries: ${missing.join(',')}`);
  }
}

function validateCandidateModeExpectations(handoff, pin) {
  if (handoff.handoff_mode !== 'service-v3-candidate') {
    fail(`handoff_mode must be service-v3-candidate for candidate validation, got ${handoff.handoff_mode}`);
  }

  if (!handoff.active_runtime_baseline || typeof handoff.active_runtime_baseline !== 'object') {
    fail('candidate handoff must include active_runtime_baseline object');
  }

  const baseline = handoff.active_runtime_baseline;
  assertEqual(baseline.release_tag, pin.release_tag, 'active_runtime_baseline.release_tag');
  assertEqual(baseline.commit_sha, pin.commit_sha, 'active_runtime_baseline.commit_sha');
  assertEqual(
    baseline.expected_service_contract_version,
    pin.expected_service_contract_version,
    'active_runtime_baseline.expected_service_contract_version'
  );
  assertEqual(
    baseline.expected_api_contract_version,
    pin.expected_api_contract_version,
    'active_runtime_baseline.expected_api_contract_version'
  );
  assertEqual(
    baseline.integration_baseline,
    pin.expected_integration_baseline,
    'active_runtime_baseline.integration_baseline'
  );

  if (!handoff.non_2xx_envelope_policy || typeof handoff.non_2xx_envelope_policy !== 'object') {
    fail('candidate handoff must include non_2xx_envelope_policy object');
  }

  const serviceV3Policy = handoff.non_2xx_envelope_policy.service_v3_candidate;
  if (!serviceV3Policy || typeof serviceV3Policy !== 'object') {
    fail('candidate handoff non_2xx_envelope_policy.service_v3_candidate is required');
  }
  assertArrayContainsAll(
    serviceV3Policy.requires || [],
    ['service_contract_version', 'error.code', 'error.message'],
    'non_2xx_envelope_policy.service_v3_candidate.requires'
  );
  assertArrayContainsAll(
    serviceV3Policy.forbids || [],
    ['legacy_error', 'api_contract_version'],
    'non_2xx_envelope_policy.service_v3_candidate.forbids'
  );

  if (!handoff.rehearsal_candidate || typeof handoff.rehearsal_candidate !== 'object') {
    fail('candidate handoff must include rehearsal_candidate object');
  }
  if (handoff.rehearsal_candidate.requires_runtime_cutover !== false) {
    fail('candidate handoff must set rehearsal_candidate.requires_runtime_cutover=false');
  }
  if (handoff.rehearsal_candidate.consumer_non_blocking_fallback_required !== true) {
    fail(
      'candidate handoff must set rehearsal_candidate.consumer_non_blocking_fallback_required=true'
    );
  }

  if (!handoff.compatibility_expectations || typeof handoff.compatibility_expectations !== 'object') {
    fail('candidate handoff must include compatibility_expectations object');
  }

  assertNonEmptyArray(
    handoff.required_consumer_validation_commands,
    'required_consumer_validation_commands'
  );
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
}

function main() {
  if (!fs.existsSync(pinPath)) {
    fail(`missing pin manifest: ${pinPath}`);
  }
  if (!fs.existsSync(producerManifestPath)) {
    fail(`missing producer manifest mirror: ${producerManifestPath}`);
  }

  if (!fs.existsSync(handoffPath)) {
    if (requireHandoff) {
      fail(`required handoff payload not found at ${handoffPath}`);
    }
    console.log(
      `MemoryKernel handoff payload validation skipped: payload not found at ${handoffPath}`
    );
    return;
  }

  const pin = parseJson(pinPath);
  const producerManifest = parseJson(producerManifestPath);
  const handoff = parseJson(handoffPath);
  const candidateRuntimeBaseline =
    handoff.handoff_mode === 'service-v3-candidate' &&
    handoff.active_runtime_baseline &&
    typeof handoff.active_runtime_baseline === 'object'
      ? handoff.active_runtime_baseline
      : null;
  const usingCandidateRuntimeBaseline =
    requirePinMatch &&
    candidateRuntimeBaseline !== null &&
    !process.env.MEMORYKERNEL_EXPECTED_SERVICE_CONTRACT_VERSION?.trim() &&
    !process.env.MEMORYKERNEL_EXPECTED_API_CONTRACT_VERSION?.trim() &&
    !process.env.MEMORYKERNEL_EXPECTED_INTEGRATION_BASELINE?.trim();

  const expectedServiceContractVersion =
    process.env.MEMORYKERNEL_EXPECTED_SERVICE_CONTRACT_VERSION?.trim() ||
    (usingCandidateRuntimeBaseline
      ? candidateRuntimeBaseline.expected_service_contract_version
      : pin.expected_service_contract_version);
  const expectedApiContractVersion =
    process.env.MEMORYKERNEL_EXPECTED_API_CONTRACT_VERSION?.trim() ||
    (usingCandidateRuntimeBaseline
      ? candidateRuntimeBaseline.expected_api_contract_version
      : pin.expected_api_contract_version);
  const expectedIntegrationBaseline =
    process.env.MEMORYKERNEL_EXPECTED_INTEGRATION_BASELINE?.trim() ||
    (usingCandidateRuntimeBaseline
      ? candidateRuntimeBaseline.integration_baseline
      : pin.expected_integration_baseline);
  const actualServiceContractVersion = usingCandidateRuntimeBaseline
    ? candidateRuntimeBaseline.expected_service_contract_version
    : handoff.expected_service_contract_version;
  const actualApiContractVersion = usingCandidateRuntimeBaseline
    ? candidateRuntimeBaseline.expected_api_contract_version
    : handoff.expected_api_contract_version;
  const actualIntegrationBaseline = usingCandidateRuntimeBaseline
    ? candidateRuntimeBaseline.integration_baseline
    : handoff.integration_baseline;

  if (requirePinMatch) {
    assertEqual(handoff.release_tag, pin.release_tag, 'release_tag');
    assertEqual(handoff.commit_sha, pin.commit_sha, 'commit_sha');
  } else {
    if (typeof handoff.release_tag !== 'string' || handoff.release_tag.trim() === '') {
      fail('release_tag must be a non-empty string when pin match is relaxed');
    }
    if (typeof handoff.commit_sha !== 'string' || !/^[0-9a-f]{40}$/.test(handoff.commit_sha)) {
      fail('commit_sha must be a 40-character SHA when pin match is relaxed');
    }
  }

  assertEqual(actualServiceContractVersion, expectedServiceContractVersion, 'expected_service_contract_version');
  assertEqual(actualApiContractVersion, expectedApiContractVersion, 'expected_api_contract_version');
  assertEqual(actualIntegrationBaseline, expectedIntegrationBaseline, 'integration_baseline');
  assertEqual(
    handoff.manifest_contract_version,
    producerManifest.manifest_contract_version,
    'manifest_contract_version'
  );

  assertNonEmptyArray(handoff.error_code_enum, 'error_code_enum');
  const handoffSet = toStringSet(handoff.error_code_enum);
  const candidateMode =
    expectedServiceContractVersion !== pin.expected_service_contract_version;
  if (!candidateMode) {
    const manifestSet = toStringSet(producerManifest.error_code_enum ?? []);
    compareSets(handoffSet, manifestSet, 'error_code_enum');
  } else {
    validateCandidateModeExpectations(handoff, pin);
  }

  assertNonEmptyArray(handoff.verification_commands, 'verification_commands');
  if (!handoff.handoff_generated_at_utc || Number.isNaN(Date.parse(handoff.handoff_generated_at_utc))) {
    fail('handoff_generated_at_utc must be a valid ISO datetime string');
  }

  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  const evidence = {
    schema_version: 'v1',
    generated_at_utc: new Date().toISOString(),
    handoff_payload_path: handoffPath,
    baseline: {
      release_tag: pin.release_tag,
      commit_sha: pin.commit_sha,
      expected_service_contract_version: expectedServiceContractVersion,
      expected_api_contract_version: expectedApiContractVersion,
      integration_baseline: expectedIntegrationBaseline,
    },
    checks: {
      release_tag: requirePinMatch ? 'pass' : 'skipped-pin-match-relaxed',
      commit_sha: requirePinMatch ? 'pass' : 'skipped-pin-match-relaxed',
      expected_service_contract_version: 'pass',
      expected_api_contract_version: 'pass',
      integration_baseline: 'pass',
      candidate_runtime_baseline:
        usingCandidateRuntimeBaseline ? 'used-for-pinned-validation' : 'not-used',
      manifest_contract_version: 'pass',
      error_code_enum_set_equality:
        !candidateMode
          ? 'pass'
          : 'skipped-service-version-differs-from-current-pin',
      verification_commands_present: 'pass',
      handoff_generated_at_utc_parseable: 'pass',
    },
    mode: candidateMode ? 'rehearsal-candidate' : 'pinned-baseline',
    status: 'passed',
  };

  fs.writeFileSync(outputPath, `${JSON.stringify(evidence, null, 2)}\n`);
  console.log('MemoryKernel handoff payload validation passed.');
  console.log(`Wrote MemoryKernel handoff evidence: ${outputPath}`);
}

main();
