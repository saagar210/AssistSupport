import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const pinPath = path.join(root, 'config', 'memorykernel-integration-pin.json');
const matrixPath = path.join(root, 'docs', 'MEMORYKERNEL_COMPATIBILITY_MATRIX.md');

function fail(message) {
  console.error(`MemoryKernel pin validation failed: ${message}`);
  process.exit(1);
}

if (!fs.existsSync(pinPath)) {
  fail(`missing pin manifest at ${pinPath}`);
}
if (!fs.existsSync(matrixPath)) {
  fail(`missing compatibility matrix at ${matrixPath}`);
}

const pin = JSON.parse(fs.readFileSync(pinPath, 'utf8'));
const matrix = fs.readFileSync(matrixPath, 'utf8');

const requiredStringFields = [
  'memorykernel_repo',
  'release_tag',
  'commit_sha',
  'expected_service_contract_version',
  'expected_api_contract_version',
  'expected_integration_baseline',
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

console.log('MemoryKernel pin + compatibility matrix validation passed.');
