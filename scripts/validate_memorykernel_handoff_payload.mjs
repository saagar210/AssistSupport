import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const pinPath = path.join(root, 'config', 'memorykernel-integration-pin.json');
const producerManifestPath = path.join(root, 'config', 'memorykernel-producer-manifest.json');
const defaultHandoffPath =
  '/Users/d/Projects/MemoryKernel/docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json';
const handoffPath = process.env.MEMORYKERNEL_HANDOFF_PAYLOAD_PATH?.trim() || defaultHandoffPath;
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

  const expectedServiceContractVersion =
    process.env.MEMORYKERNEL_EXPECTED_SERVICE_CONTRACT_VERSION?.trim() ||
    pin.expected_service_contract_version;
  const expectedApiContractVersion =
    process.env.MEMORYKERNEL_EXPECTED_API_CONTRACT_VERSION?.trim() ||
    pin.expected_api_contract_version;
  const expectedIntegrationBaseline =
    process.env.MEMORYKERNEL_EXPECTED_INTEGRATION_BASELINE?.trim() ||
    pin.expected_integration_baseline;

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

  assertEqual(
    handoff.expected_service_contract_version,
    expectedServiceContractVersion,
    'expected_service_contract_version'
  );
  assertEqual(
    handoff.expected_api_contract_version,
    expectedApiContractVersion,
    'expected_api_contract_version'
  );
  assertEqual(
    handoff.integration_baseline,
    expectedIntegrationBaseline,
    'integration_baseline'
  );
  assertEqual(
    handoff.manifest_contract_version,
    producerManifest.manifest_contract_version,
    'manifest_contract_version'
  );

  assertNonEmptyArray(handoff.error_code_enum, 'error_code_enum');
  const handoffSet = toStringSet(handoff.error_code_enum);
  if (expectedServiceContractVersion === pin.expected_service_contract_version) {
    const manifestSet = toStringSet(producerManifest.error_code_enum ?? []);
    compareSets(handoffSet, manifestSet, 'error_code_enum');
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
      manifest_contract_version: 'pass',
      error_code_enum_set_equality:
        expectedServiceContractVersion === pin.expected_service_contract_version
          ? 'pass'
          : 'skipped-service-version-differs-from-current-pin',
      verification_commands_present: 'pass',
      handoff_generated_at_utc_parseable: 'pass',
    },
    mode: requirePinMatch ? 'pinned-baseline' : 'rehearsal-candidate',
    status: 'passed',
  };

  fs.writeFileSync(outputPath, `${JSON.stringify(evidence, null, 2)}\n`);
  console.log('MemoryKernel handoff payload validation passed.');
  console.log(`Wrote MemoryKernel handoff evidence: ${outputPath}`);
}

main();
