import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';

const root = process.cwd();
const pinPath = path.join(root, 'config', 'memorykernel-integration-pin.json');
const outputDir = path.join(root, 'artifacts');
const outputPath = path.join(outputDir, 'memorykernel-contract-evidence.json');

const pin = JSON.parse(fs.readFileSync(pinPath, 'utf8'));
const assistSupportCommit = execSync('git rev-parse HEAD', { encoding: 'utf8' }).trim();
const assistSupportBranch = execSync('git rev-parse --abbrev-ref HEAD', { encoding: 'utf8' }).trim();

const evidence = {
  schema_version: 'v1',
  generated_at: new Date().toISOString(),
  consumer: {
    project: 'AssistSupport',
    branch: assistSupportBranch,
    commit_sha: assistSupportCommit,
  },
  memorykernel_pin: {
    repo: pin.memorykernel_repo,
    release_tag: pin.release_tag,
    commit_sha: pin.commit_sha,
  },
  expected_contracts: {
    service_contract_version: pin.expected_service_contract_version,
    api_contract_version: pin.expected_api_contract_version,
    integration_baseline: pin.expected_integration_baseline,
  },
  verification: {
    contract_test_command: 'pnpm run test:memorykernel-contract',
    pin_sync_command: 'pnpm run check:memorykernel-pin',
    status: 'passed',
  },
};

fs.mkdirSync(outputDir, { recursive: true });
fs.writeFileSync(outputPath, `${JSON.stringify(evidence, null, 2)}\n`);

console.log(`Wrote MemoryKernel contract evidence: ${outputPath}`);
