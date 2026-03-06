import { mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";

const SOURCE_ROOT = "src-tauri/src";
const REPORT_PATH = "docs/reports/lock-await-exceptions.md";
const TAG = "LOCK_AWAIT_EXCEPTION:";

const listRustFiles = (dir) => {
  const entries = readdirSync(dir, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...listRustFiles(fullPath));
      continue;
    }
    if (entry.isFile() && fullPath.endsWith(".rs")) {
      files.push(fullPath);
    }
  }
  return files.sort((a, b) => a.localeCompare(b));
};

const getTagReason = (line) => {
  const index = line.indexOf(TAG);
  if (index === -1) return null;
  const reason = line.slice(index + TAG.length).trim();
  return reason.length > 0 ? reason : null;
};

const exceptions = [];
const errors = [];

for (const filePath of listRustFiles(SOURCE_ROOT)) {
  const lines = readFileSync(filePath, "utf8").split(/\r?\n/);
  for (let idx = 0; idx < lines.length; idx += 1) {
    const line = lines[idx];
    if (!line.includes("allow(clippy::await_holding_lock)")) continue;

    const sameLine = getTagReason(line);
    const prevLine = idx > 0 ? getTagReason(lines[idx - 1]) : null;
    const nextLine = idx + 1 < lines.length ? getTagReason(lines[idx + 1]) : null;
    const reason = sameLine ?? prevLine ?? nextLine;

    if (!reason) {
      errors.push(
        `${filePath}:${idx + 1} has allow(clippy::await_holding_lock) without ${TAG} justification`,
      );
      continue;
    }

    exceptions.push({
      filePath,
      line: idx + 1,
      reason,
    });
  }
}

if (errors.length > 0) {
  const detail = errors.map((error) => `- ${error}`).join("\n");
  throw new Error(`Lock-await exception policy validation failed:\n${detail}`);
}

const generatedAt = new Date().toISOString();
const body =
  exceptions.length === 0
    ? `${[
        "# Lock-Await Exceptions",
        "",
        `Generated: ${generatedAt}`,
        "",
        "No lock-await lint exceptions are currently allowlisted.",
        "",
      ].join("\n")}`
    : `${[
        "# Lock-Await Exceptions",
        "",
        `Generated: ${generatedAt}`,
        "",
        "| File | Line | Justification |",
        "|---|---:|---|",
        ...exceptions.map(
          (entry) =>
            `| ${entry.filePath} | ${entry.line} | ${entry.reason.replaceAll("|", "\\|")} |`,
        ),
        "",
      ].join("\n")}`;

mkdirSync(path.dirname(REPORT_PATH), { recursive: true });
writeFileSync(REPORT_PATH, body);

console.log(
  `Lock-await exception report generated (${exceptions.length} entries): ${REPORT_PATH}`,
);
