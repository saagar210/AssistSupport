import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import path from "node:path";

const gitBin = "/usr/bin/git";

const staged = execFileSync(
  gitBin,
  ["diff", "--cached", "--name-only", "--diff-filter=ACMR"],
  {
    encoding: "utf8",
  },
)
  .split("\n")
  .map((line) => line.trim())
  .filter(Boolean);

if (staged.length === 0) {
  console.error("No staged files found. Stage files first.");
  process.exit(1);
}

const topSegments = staged.map((file) => file.split("/")[0]).filter(Boolean);
const counts = new Map();
for (const seg of topSegments) {
  counts.set(seg, (counts.get(seg) ?? 0) + 1);
}
const scope =
  [...counts.entries()].sort((a, b) => b[1] - a[1])[0]?.[0] ??
  path.basename(process.cwd()).toLowerCase();

const lower = staged.map((f) => f.toLowerCase());
const allDocs = lower.every((f) => f.endsWith(".md"));
const hasTests = lower.some(
  (f) => f.includes("/tests/") || f.includes(".test.") || f.includes(".spec."),
);
const hasCi = lower.some((f) => f.startsWith(".github/workflows/"));
const hasPerf = lower.some(
  (f) => f.includes("/perf/") || f.includes("lighthouserc"),
);
const hasDeps = lower.some(
  (f) =>
    f.endsWith("package.json") ||
    f.endsWith("pnpm-lock.yaml") ||
    f.endsWith("yarn.lock"),
);

let type = "feat";
if (allDocs) type = "docs";
else if (hasCi) type = "ci";
else if (hasPerf) type = "perf";
else if (hasTests) type = "test";
else if (hasDeps) type = "chore";

const focus =
  staged.length === 1
    ? `update ${path.basename(staged[0])}`
    : `update ${staged.length} files for ${scope} changes`;

const candidate = `${type}(${scope}): ${focus}`.slice(0, 72);
const outputPath = ".git/CODEX_COMMIT_MESSAGE_CANDIDATE";

writeFileSync(outputPath, `${candidate}\n`, "utf8");

console.log(candidate);
console.log(`Saved candidate to ${outputPath}`);
