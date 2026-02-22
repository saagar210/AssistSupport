import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";

const workflowDir = ".github/workflows";
const repoRoot = process.cwd();
const packageJson = JSON.parse(readFileSync("package.json", "utf8"));
const scripts = new Set(Object.keys(packageJson.scripts ?? {}));
const ignoredCommands = new Set([
  "install",
  "audit",
  "outdated",
  "exec",
]);

const failures = [];

function pathExistsForWorkflow(relativePath, workflowFile, workingDirectory = ".") {
  if (!relativePath || relativePath.startsWith("${")) {
    return true;
  }

  const normalized = relativePath
    .replace(/^["']|["']$/g, "")
    .replace(/\\$/g, "");

  if (/^(venv|\.venv)\//.test(normalized)) {
    return true;
  }

  if (normalized.startsWith("-")) {
    return true;
  }

  const absolute = path.resolve(repoRoot, workingDirectory, normalized);
  if (!existsSync(absolute)) {
    failures.push(
      `${workflowFile}: missing path "${normalized}" (resolved from working-directory "${workingDirectory}")`
    );
    return false;
  }
  return true;
}

function processRunLine(line, workflowFile, workingDirectory) {
  const pnpmMatch = line.match(/^\s*pnpm\s+(?:run\s+)?([A-Za-z0-9:_-]+)\b/);
  if (pnpmMatch?.[1]) {
    const command = pnpmMatch[1];
    if (!ignoredCommands.has(command) && !scripts.has(command)) {
      failures.push(`${workflowFile}: missing package.json script "${command}"`);
    }
  }

  for (const match of line.matchAll(/(?:^|&&|\|\|)\s*cd\s+([^\s;&|]+)/g)) {
    pathExistsForWorkflow(match[1], workflowFile, workingDirectory);
  }

  for (const match of line.matchAll(/\b(?:bash|sh|node|python3?|source)\s+([^\s"'`|&;]+)/g)) {
    const candidate = match[1];
    if (!candidate || candidate.startsWith("-") || candidate.startsWith("${")) {
      continue;
    }
    if (candidate.includes("/") || /\.(sh|mjs|js|py)$/.test(candidate)) {
      pathExistsForWorkflow(candidate, workflowFile, workingDirectory);
    }
  }
}

for (const file of readdirSync(workflowDir)) {
  if (!file.endsWith(".yml") && !file.endsWith(".yaml")) {
    continue;
  }

  const fullPath = path.join(workflowDir, file);
  const text = readFileSync(fullPath, "utf8");
  const lines = text.split("\n");
  let currentWorkingDirectory = ".";
  let blockMode = false;
  let blockIndent = 0;

  for (const rawLine of lines) {
    const line = rawLine.replace(/\r/g, "");
    const indent = line.match(/^\s*/)?.[0].length ?? 0;

    if (blockMode && indent <= blockIndent && line.trim() !== "") {
      blockMode = false;
    }

    if (/^\s*-\s+(name|uses|run):/.test(line)) {
      currentWorkingDirectory = ".";
    }

    const workingDirectoryMatch = line.match(/^\s*working-directory:\s*([^\s#]+)/);
    if (workingDirectoryMatch?.[1]) {
      currentWorkingDirectory = workingDirectoryMatch[1].replace(/^["']|["']$/g, "");
      pathExistsForWorkflow(currentWorkingDirectory, fullPath);
      continue;
    }

    const runMatch = line.match(/^(\s*)run:\s*(.*)$/);
    if (runMatch) {
      const runBody = runMatch[2].trim();
      if (runBody === "|" || runBody === ">") {
        blockMode = true;
        blockIndent = runMatch[1].length;
      } else if (runBody.length > 0) {
        processRunLine(runBody, fullPath, currentWorkingDirectory);
      }
      continue;
    }

    if (blockMode && line.trim() !== "") {
      processRunLine(line.trim(), fullPath, currentWorkingDirectory);
    }
  }
}

if (failures.length > 0) {
  console.error("Workflow command drift detected:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log("Workflow command drift check passed.");
