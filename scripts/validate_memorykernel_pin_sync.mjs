import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const pinPath = path.join(root, 'config', 'memorykernel-integration-pin.json');
const matrixPath = path.join(root, 'docs', 'MEMORYKERNEL_COMPATIBILITY_MATRIX.md');
const producerManifestPath = path.join(root, 'config', 'memorykernel-producer-manifest.json');
const EXPECTED_PRODUCER_MANIFEST_CONTRACT_VERSION = 'producer-contract-manifest.v1';
const EXPECTED_SERVICE_V2_ERROR_CODES = [
  'invalid_json',
  'validation_error',
  'context_package_not_found',
  'write_conflict',
  'write_failed',
  'schema_unavailable',
  'migration_failed',
  'query_failed',
  'context_lookup_failed',
  'internal_error',
];

function fail(message) {
  console.error(`MemoryKernel pin validation failed: ${message}`);
  process.exit(1);
}

function sha256Hex(payload) {
  return crypto.createHash('sha256').update(payload).digest('hex');
}

function parseGitHubRepo(repoUrl) {
  const match = repoUrl.match(/^https:\/\/github\.com\/([^/]+)\/([^/]+?)(?:\.git)?$/);
  if (!match) {
    fail(`memorykernel_repo must be a GitHub HTTPS URL, got ${repoUrl}`);
  }
  return { owner: match[1], repo: match[2] };
}

function buildManifestApiUrl(pin) {
  const { owner, repo } = parseGitHubRepo(pin.memorykernel_repo);
  return `https://api.github.com/repos/${owner}/${repo}/contents/contracts/integration/v1/producer-contract-manifest.json?ref=${pin.release_tag}`;
}

function validateProducerManifestFields(pin, producerManifest) {
  if (
    producerManifest.manifest_contract_version !==
    EXPECTED_PRODUCER_MANIFEST_CONTRACT_VERSION
  ) {
    fail(
      `producer manifest contract version must be ${EXPECTED_PRODUCER_MANIFEST_CONTRACT_VERSION}, got ${producerManifest.manifest_contract_version}`
    );
  }

  const producerManifestStringFields = [
    'release_tag',
    'commit_sha',
    'expected_service_contract_version',
    'expected_api_contract_version',
    'integration_baseline',
  ];

  for (const field of producerManifestStringFields) {
    if (
      typeof producerManifest[field] !== 'string' ||
      producerManifest[field].trim() === ''
    ) {
      fail(`producer manifest field ${field} must be a non-empty string`);
    }
  }

  if (
    !Array.isArray(producerManifest.error_code_enum) ||
    producerManifest.error_code_enum.length === 0
  ) {
    fail('producer manifest field error_code_enum must be a non-empty array');
  }

  const producerErrorCodes = producerManifest.error_code_enum;
  if (producerErrorCodes.some((code) => typeof code !== 'string' || code.trim() === '')) {
    fail('producer manifest error_code_enum entries must be non-empty strings');
  }

  if (new Set(producerErrorCodes).size !== producerErrorCodes.length) {
    fail('producer manifest error_code_enum contains duplicate values');
  }

  const expectedErrorCodeSet = new Set(EXPECTED_SERVICE_V2_ERROR_CODES);
  const producerErrorCodeSet = new Set(producerErrorCodes);
  const missingExpectedCodes = EXPECTED_SERVICE_V2_ERROR_CODES.filter(
    (code) => !producerErrorCodeSet.has(code)
  );
  const unexpectedCodes = producerErrorCodes.filter(
    (code) => !expectedErrorCodeSet.has(code)
  );

  if (missingExpectedCodes.length > 0 || unexpectedCodes.length > 0) {
    fail(
      `producer manifest error_code_enum mismatch; missing=[${missingExpectedCodes.join(',')}] unexpected=[${unexpectedCodes.join(',')}]`
    );
  }

  if (producerManifest.release_tag !== pin.release_tag) {
    fail('producer manifest release_tag does not match pin release_tag');
  }
  if (producerManifest.commit_sha !== pin.commit_sha) {
    fail('producer manifest commit_sha does not match pin commit_sha');
  }
  if (
    producerManifest.expected_service_contract_version !==
    pin.expected_service_contract_version
  ) {
    fail(
      'producer manifest expected_service_contract_version does not match pin expected_service_contract_version'
    );
  }
  if (
    producerManifest.expected_api_contract_version !== pin.expected_api_contract_version
  ) {
    fail(
      'producer manifest expected_api_contract_version does not match pin expected_api_contract_version'
    );
  }
  if (producerManifest.integration_baseline !== pin.expected_integration_baseline) {
    fail(
      'producer manifest integration_baseline does not match pin expected_integration_baseline'
    );
  }
}

async function validateRemoteManifestHash(pin, expectedHash) {
  if (process.env.ASSISTSUPPORT_VALIDATE_REMOTE_MANIFEST !== '1') {
    return;
  }

  const remoteUrl = buildManifestApiUrl(pin);
  const token = process.env.MEMORYKERNEL_GITHUB_TOKEN?.trim();
  if (!token) {
    fail(
      'ASSISTSUPPORT_VALIDATE_REMOTE_MANIFEST=1 requires MEMORYKERNEL_GITHUB_TOKEN for cross-repo manifest access'
    );
  }

  const headers = {
    Accept: 'application/vnd.github.raw+json',
    Authorization: `Bearer ${token}`,
    'X-GitHub-Api-Version': '2022-11-28',
  };

  let response;
  try {
    response = await fetch(remoteUrl, {
      signal: AbortSignal.timeout(10_000),
      headers,
    });
  } catch (error) {
    fail(`failed to fetch producer manifest from ${remoteUrl}: ${error.message}`);
  }

  if (!response.ok) {
    fail(
      `failed to fetch producer manifest from ${remoteUrl}: HTTP ${response.status}`
    );
  }

  const remoteManifestRaw = await response.text();
  const remoteHash = sha256Hex(remoteManifestRaw);
  if (remoteHash !== expectedHash) {
    fail(
      `remote producer manifest hash mismatch; expected ${expectedHash}, got ${remoteHash}`
    );
  }

  let remoteManifest;
  try {
    remoteManifest = JSON.parse(remoteManifestRaw);
  } catch {
    fail(`remote producer manifest at ${remoteUrl} is not valid JSON`);
  }

  validateProducerManifestFields(pin, remoteManifest);
}

async function main() {
  if (!fs.existsSync(pinPath)) {
    fail(`missing pin manifest at ${pinPath}`);
  }
  if (!fs.existsSync(matrixPath)) {
    fail(`missing compatibility matrix at ${matrixPath}`);
  }
  if (!fs.existsSync(producerManifestPath)) {
    fail(`missing producer manifest mirror at ${producerManifestPath}`);
  }

  const pin = JSON.parse(fs.readFileSync(pinPath, 'utf8'));
  const matrix = fs.readFileSync(matrixPath, 'utf8');
  const producerManifestRaw = fs.readFileSync(producerManifestPath, 'utf8');
  const producerManifest = JSON.parse(producerManifestRaw);

  const requiredStringFields = [
    'memorykernel_repo',
    'release_tag',
    'commit_sha',
    'expected_service_contract_version',
    'expected_api_contract_version',
    'expected_integration_baseline',
    'expected_producer_manifest_sha256',
    'default_base_url',
  ];

  for (const field of requiredStringFields) {
    if (typeof pin[field] !== 'string' || pin[field].trim() === '') {
      fail(`pin field ${field} must be a non-empty string`);
    }
  }

  if (!Number.isInteger(pin.default_timeout_ms) || pin.default_timeout_ms < 100) {
    fail('pin field default_timeout_ms must be an integer >= 100');
  }

  if (!/^v\d+\.\d+\.\d+$/.test(pin.release_tag)) {
    fail(`release_tag must be semver tag (vX.Y.Z), got ${pin.release_tag}`);
  }

  if (!/^[0-9a-f]{40}$/.test(pin.commit_sha)) {
    fail(`commit_sha must be a 40-character SHA, got ${pin.commit_sha}`);
  }

  if (!/^[0-9a-f]{64}$/.test(pin.expected_producer_manifest_sha256)) {
    fail(
      `expected_producer_manifest_sha256 must be a 64-character SHA-256 hex, got ${pin.expected_producer_manifest_sha256}`
    );
  }

  const mirroredHash = sha256Hex(producerManifestRaw);
  if (mirroredHash !== pin.expected_producer_manifest_sha256) {
    fail(
      `mirrored producer manifest hash mismatch; expected ${pin.expected_producer_manifest_sha256}, got ${mirroredHash}`
    );
  }

  validateProducerManifestFields(pin, producerManifest);
  await validateRemoteManifestHash(pin, pin.expected_producer_manifest_sha256);

  const expectedBaselineTagLine = `- MemoryKernel release tag: \`${pin.release_tag}\``;
  const expectedBaselineShaLine = `- MemoryKernel commit SHA: \`${pin.commit_sha}\``;
  if (!matrix.includes(expectedBaselineTagLine)) {
    fail('compatibility matrix baseline tag does not match pin manifest');
  }
  if (!matrix.includes(expectedBaselineShaLine)) {
    fail('compatibility matrix baseline commit does not match pin manifest');
  }

  const escapedTag = pin.release_tag.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const escapedSha = pin.commit_sha.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const rowPattern = new RegExp(`\\|[^\\n]*\\|\\s*${escapedTag}\\s*\\|\\s*${escapedSha}\\s*\\|`);
  if (!rowPattern.test(matrix)) {
    fail('compatibility matrix approved version row does not include the pin tag/SHA pair');
  }

  if (!matrix.includes(pin.expected_service_contract_version)) {
    fail('compatibility matrix does not mention expected service contract version from pin');
  }
  if (!matrix.includes(pin.expected_api_contract_version)) {
    fail('compatibility matrix does not mention expected API contract version from pin');
  }
  if (!matrix.includes(pin.expected_integration_baseline)) {
    fail('compatibility matrix does not mention expected integration baseline from pin');
  }

  console.log(
    'MemoryKernel pin + compatibility matrix + producer manifest validation passed.'
  );
}

main().catch((error) => fail(error.message));
