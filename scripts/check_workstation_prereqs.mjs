import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const root = process.cwd();
const outputPath = path.join(root, 'artifacts', 'workstation-preflight-evidence.json');

function fail(message) {
  console.error(`Workstation preflight failed: ${message}`);
  process.exit(1);
}

function parseSemver(rawValue, toolName) {
  const match = rawValue.trim().match(/v?(\d+)\.(\d+)\.(\d+)/);
  if (!match) {
    fail(`unable to parse ${toolName} version from: ${rawValue}`);
  }
  return {
    major: Number.parseInt(match[1], 10),
    minor: Number.parseInt(match[2], 10),
    patch: Number.parseInt(match[3], 10),
    raw: rawValue.trim(),
  };
}

function compareSemver(actual, minimum) {
  if (actual.major !== minimum.major) {
    return actual.major - minimum.major;
  }
  if (actual.minor !== minimum.minor) {
    return actual.minor - minimum.minor;
  }
  return actual.patch - minimum.patch;
}

function assertMinVersion(toolName, actual, minimum) {
  if (compareSemver(actual, minimum) < 0) {
    fail(
      `${toolName} version must be >= ${minimum.major}.${minimum.minor}.${minimum.patch}; got ${actual.raw}`
    );
  }
}

function readCommandVersion(command, args) {
  try {
    return execFileSync(command, args, {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
    });
  } catch (error) {
    const details = error?.stderr?.toString().trim() || error.message;
    fail(`${command} is required and must be available in PATH (${details})`);
  }
}

function assertPathExists(relativePath) {
  const absolutePath = path.join(root, relativePath);
  if (!fs.existsSync(absolutePath)) {
    fail(`missing required repository path: ${relativePath}`);
  }
  return absolutePath;
}

function main() {
  const minimumVersions = {
    node: { major: 20, minor: 0, patch: 0 },
    pnpm: { major: 10, minor: 0, patch: 0 },
    rustc: { major: 1, minor: 75, patch: 0 },
    cargo: { major: 1, minor: 75, patch: 0 },
    git: { major: 2, minor: 0, patch: 0 },
  };

  const nodeVersion = parseSemver(process.versions.node, 'node');
  const pnpmVersion = parseSemver(readCommandVersion('pnpm', ['--version']), 'pnpm');
  const rustcVersion = parseSemver(readCommandVersion('rustc', ['--version']), 'rustc');
  const cargoVersion = parseSemver(readCommandVersion('cargo', ['--version']), 'cargo');
  const gitVersion = parseSemver(readCommandVersion('git', ['--version']), 'git');

  assertMinVersion('node', nodeVersion, minimumVersions.node);
  assertMinVersion('pnpm', pnpmVersion, minimumVersions.pnpm);
  assertMinVersion('rustc', rustcVersion, minimumVersions.rustc);
  assertMinVersion('cargo', cargoVersion, minimumVersions.cargo);
  assertMinVersion('git', gitVersion, minimumVersions.git);

  const requiredPaths = [
    'package.json',
    'pnpm-lock.yaml',
    'config/memorykernel-integration-pin.json',
    'config/memorykernel-producer-manifest.json',
    'docs/MEMORYKERNEL_COMPATIBILITY_MATRIX.md',
    'services/memorykernel/Cargo.toml',
    'services/memorykernel/scripts/verify_producer_handoff_payload.sh',
  ];
  const verifiedPaths = requiredPaths.map(assertPathExists);

  const warnings = [];
  if (os.platform() !== 'darwin') {
    warnings.push(
      'non-macOS platform detected; Tauri desktop packaging is only validated on macOS workstations'
    );
  }

  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  const evidence = {
    schema_version: 'workstation-preflight.v1',
    generated_at_utc: new Date().toISOString(),
    repo_root: root,
    platform: {
      os: os.platform(),
      release: os.release(),
      arch: os.arch(),
    },
    versions: {
      node: nodeVersion.raw,
      pnpm: pnpmVersion.raw,
      rustc: rustcVersion.raw,
      cargo: cargoVersion.raw,
      git: gitVersion.raw,
    },
    verified_paths: verifiedPaths.map((absolutePath) => path.relative(root, absolutePath)),
    warnings,
  };
  fs.writeFileSync(outputPath, `${JSON.stringify(evidence, null, 2)}\n`);

  if (warnings.length > 0) {
    console.warn(`Workstation preflight warnings: ${warnings.join(' | ')}`);
  }
  console.log(`Workstation preflight passed. Evidence: ${outputPath}`);
}

main();
