import { execSync } from 'node:child_process';

const defaultBaseRef = (() => {
  try {
    return execSync('git symbolic-ref refs/remotes/origin/HEAD', { encoding: 'utf8' }).trim().replace('refs/remotes/', '');
  } catch {
    return 'origin/main';
  }
})();

const baseRef = process.env.GITHUB_BASE_REF ? `origin/${process.env.GITHUB_BASE_REF}` : defaultBaseRef;
const diff = execSync(`git diff --name-only ${baseRef}...HEAD`, { encoding: 'utf8' })
  .split('\n')
  .map((line) => line.trim())
  .filter(Boolean);

const isProdCode = (file) => /^(src|app|server|api)\//.test(file) && !/\.(test|spec)\.[cm]?[jt]sx?$/.test(file);
const isTest = (file) => /^tests\//.test(file) || /\.(test|spec)\.[cm]?[jt]sx?$/.test(file);
const isDoc = (file) => /^docs\//.test(file) || /^openapi\//.test(file) || file === 'README.md';
const isApiSurface = (file) => /^(src|app|server|api)\/.*(route|controller|handler|webhook|api|command)/.test(file);
const isArchChange = (file) => /^src\/(auth|db|infra|queue|events|architecture)\//.test(file) || /^infra\//.test(file);
const isAdr = (file) => /^docs\/adr\/\d{4}-.*\.md$/.test(file);

const prodChanged = diff.some(isProdCode);
const testsChanged = diff.some(isTest);
const apiChanged = diff.some(isApiSurface);
const docsChanged = diff.some(isDoc);
const archChanged = diff.some(isArchChange);
const adrChanged = diff.some(isAdr);

const failures = [];
if (prodChanged && !testsChanged) failures.push('Policy failure: production code changed without test updates.');
if (apiChanged && !docsChanged) failures.push('Policy failure: API/command changes without docs/OpenAPI updates.');
if (archChanged && !adrChanged) failures.push('Policy failure: architecture-impacting change without ADR.');

if (failures.length > 0) {
  for (const failure of failures) console.error(failure);
  process.exit(1);
}

console.log('Policy checks passed.');
