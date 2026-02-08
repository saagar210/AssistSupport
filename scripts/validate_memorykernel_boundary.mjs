import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const adapterPath = path.join(root, 'src-tauri', 'src', 'commands', 'memory_kernel.rs');
const rustRoot = path.join(root, 'src-tauri', 'src');
const frontendRoot = path.join(root, 'src');

function fail(message) {
  console.error(`MemoryKernel boundary validation failed: ${message}`);
  process.exit(1);
}

function walkFiles(startPath, extension, files = []) {
  const entries = fs.readdirSync(startPath, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(startPath, entry.name);
    if (entry.isDirectory()) {
      walkFiles(fullPath, extension, files);
      continue;
    }
    if (entry.isFile() && fullPath.endsWith(extension)) {
      files.push(fullPath);
    }
  }
  return files;
}

function assertIncludes(filePath, snippet) {
  const text = fs.readFileSync(filePath, 'utf8');
  if (!text.includes(snippet)) {
    fail(`${path.relative(root, filePath)} must include '${snippet}'`);
  }
}

function assertNoEndpointLeak(filePath) {
  const relPath = path.relative(root, filePath);
  const text = fs.readFileSync(filePath, 'utf8');
  const leaks = ['/v1/health', '/v1/db/schema-version', '/v1/query/ask'].filter((needle) =>
    text.includes(needle)
  );
  if (leaks.length > 0) {
    fail(`${relPath} contains MemoryKernel endpoint strings outside adapter: ${leaks.join(', ')}`);
  }
}

function main() {
  if (!fs.existsSync(adapterPath)) {
    fail(`adapter boundary file is missing: ${path.relative(root, adapterPath)}`);
  }

  // Enforce that core endpoint contract calls remain in the adapter file.
  assertIncludes(adapterPath, '/v1/health');
  assertIncludes(adapterPath, '/v1/db/schema-version');
  assertIncludes(adapterPath, 'memory_kernel_query_ask');

  // Disallow service endpoint leaks outside adapter boundary in Rust sources.
  const rustFiles = walkFiles(rustRoot, '.rs');
  for (const file of rustFiles) {
    if (path.resolve(file) === path.resolve(adapterPath)) {
      continue;
    }
    assertNoEndpointLeak(file);
  }

  // Frontend must use Tauri invoke commands, not direct service endpoints.
  const frontendFiles = [
    ...walkFiles(frontendRoot, '.ts'),
    ...walkFiles(frontendRoot, '.tsx'),
  ];
  for (const file of frontendFiles) {
    assertNoEndpointLeak(file);
  }

  const hookPath = path.join(root, 'src', 'hooks', 'useMemoryKernelEnrichment.ts');
  const initPath = path.join(root, 'src', 'hooks', 'useInitialize.ts');
  assertIncludes(hookPath, "invoke<MemoryKernelEnrichmentResult>('memory_kernel_query_ask'");
  assertIncludes(initPath, "invoke<MemoryKernelPreflightStatus>('get_memory_kernel_preflight_status'");

  console.log('MemoryKernel boundary validation passed.');
}

main();
