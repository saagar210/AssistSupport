import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";

const staged = execFileSync(
  "/usr/bin/git",
  ["diff", "--cached", "--name-only", "--diff-filter=ACMR"],
  { encoding: "utf8" },
)
  .split("\n")
  .map((line) => line.trim())
  .filter(Boolean);

if (staged.length === 0) {
  console.error("No staged files.");
  process.exit(1);
}

const lower = staged.map((f) => f.toLowerCase());
const hasDocs = lower.some((f) => f.endsWith(".md") || f.includes("docs/"));
const hasTests = lower.some((f) => f.includes("test") || f.includes("spec"));
const hasCi = lower.some((f) => f.startsWith(".github/workflows/"));
const hasPerf = lower.some((f) => f.includes("/perf/") || f.includes("lighthouserc"));
const hasDeps = lower.some((f) => f.endsWith("package.json") || f.includes("lock"));

let type = "feat";
if (hasCi) type = "ci";
else if (hasPerf) type = "perf";
else if (hasTests) type = "test";
else if (hasDocs) type = "docs";
else if (hasDeps) type = "build";

let scope = "repo";
if (lower.some((f) => f.includes("scripts/git/"))) scope = "git";
else if (lower.some((f) => f.includes("scripts/perf/"))) scope = "perf";
else if (lower.some((f) => f.startsWith(".github/"))) scope = "ci";

const summary = `${type}(${scope}): update ${staged.length} file${staged.length === 1 ? "" : "s"}`;
const out = `.git/CODEX_COMMIT_MSG_PROPOSAL`;
writeFileSync(out, `${summary}\n`);
console.log(summary);
console.log(`written: ${out}`);
