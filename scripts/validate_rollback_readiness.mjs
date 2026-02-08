import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';

const root = process.cwd();
const baselineTag = process.env.ASSISTSUPPORT_BASELINE_TAG ?? 'revamp-baseline-2026-02-08';
const evidencePath =
  process.env.ASSISTSUPPORT_ROLLBACK_EVIDENCE_PATH ?? 'artifacts/rollback-readiness-evidence.json';

const requiredArtifacts = [
  'docs/revamp/ROLLBACK_PLAN.md',
  'docs/revamp/BASELINE_METRICS.md',
  'docs/revamp/FEATURE_FLAG_MATRIX.md',
];

function fail(message) {
  console.error(`Rollback readiness validation failed: ${message}`);
  process.exit(1);
}

function ensureFile(relPath) {
  const absPath = path.join(root, relPath);
  if (!fs.existsSync(absPath)) {
    fail(`missing required artifact: ${relPath}`);
  }
}

function runGit(cmd) {
  try {
    return execSync(cmd, { cwd: root, stdio: ['ignore', 'pipe', 'pipe'] }).toString().trim();
  } catch (error) {
    const stderr = error?.stderr?.toString?.().trim();
    fail(`command failed "${cmd}": ${stderr || error.message}`);
  }
}

function writeEvidence(relPath, payload) {
  const absPath = path.join(root, relPath);
  fs.mkdirSync(path.dirname(absPath), { recursive: true });
  fs.writeFileSync(absPath, `${JSON.stringify(payload, null, 2)}\n`, 'utf8');
}

function main() {
  requiredArtifacts.forEach(ensureFile);

  const tagSha = runGit(`git rev-parse --verify ${baselineTag}`);
  const headSha = runGit('git rev-parse HEAD');

  try {
    execSync(`git merge-base --is-ancestor ${baselineTag} HEAD`, {
      cwd: root,
      stdio: 'ignore',
    });
  } catch {
    fail(`baseline tag ${baselineTag} is not an ancestor of current HEAD`);
  }

  const diffFilesRaw = runGit(`git diff --name-only ${baselineTag}..HEAD`);
  const diffFiles = diffFilesRaw ? diffFilesRaw.split('\n').filter(Boolean) : [];
  const diffStat = runGit(`git diff --shortstat ${baselineTag}..HEAD`) || 'No delta';

  const payload = {
    validated_at: new Date().toISOString(),
    baseline_tag: baselineTag,
    baseline_sha: tagSha,
    head_sha: headSha,
    changed_file_count_since_baseline: diffFiles.length,
    diff_shortstat: diffStat,
    required_artifacts: requiredArtifacts,
    rollback_ready: true,
  };

  writeEvidence(evidencePath, payload);
  console.log(
    `Rollback readiness validation passed (baseline=${baselineTag}, changed_files=${diffFiles.length}).`,
  );
  console.log(`Wrote rollback readiness evidence: ${path.join(root, evidencePath)}`);
}

main();
