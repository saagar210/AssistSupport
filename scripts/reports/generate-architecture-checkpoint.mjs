import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";

const readJson = (filePath) =>
  existsSync(filePath) ? JSON.parse(readFileSync(filePath, "utf8")) : null;
const readText = (filePath) => (existsSync(filePath) ? readFileSync(filePath, "utf8") : null);

const today = new Date().toISOString();
const scorecard = readJson(".perf-results/program-scorecard.json");
const dora = readJson(".perf-results/dora.json");
const lifecycle = readJson("contracts/tauri/v1/command-lifecycle.json");
const backendSummary = readText("artifacts/backend/gates-summary.md");
const lockReport = readText("docs/reports/lock-await-exceptions.md");

const lifecycleEntries = Object.values(lifecycle?.commands ?? {});
const lifecycleStats = lifecycleEntries.reduce(
  (acc, entry) => {
    const status = typeof entry?.status === "string" ? entry.status : "unknown";
    acc[status] = (acc[status] ?? 0) + 1;
    return acc;
  },
  { active: 0, deprecated: 0, alias_of: 0, unknown: 0 },
);

const backendPassCount = backendSummary
  ? backendSummary
      .split("\n")
      .filter((line) => line.includes("| PASS |")).length
  : 0;
const backendFailCount = backendSummary
  ? backendSummary
      .split("\n")
      .filter((line) => line.includes("| FAIL |")).length
  : 0;

const lockExceptionCount = lockReport?.includes("| File | Line | Justification |")
  ? lockReport
      .split("\n")
      .filter((line) => line.startsWith("| src-tauri/"))
      .length
  : 0;

const lines = [
  "# Weekly Architecture Checkpoint",
  "",
  `Generated: ${today}`,
  "",
  "## Gate Health",
  "",
  `- Backend gate passes: ${backendPassCount}`,
  `- Backend gate failures: ${backendFailCount}`,
  `- Program scorecard status: ${scorecard?.summary?.status ?? "unknown"}`,
  `- Failing scorecard metrics: ${
    scorecard?.summary?.failingMetrics?.length
      ? scorecard.summary.failingMetrics.join(", ")
      : "none"
  }`,
  "",
  "## Command Lifecycle",
  "",
  `- Current release cycle: ${lifecycle?.current_release_cycle ?? "unknown"}`,
  `- Active commands: ${lifecycleStats.active ?? 0}`,
  `- Deprecated commands: ${lifecycleStats.deprecated ?? 0}`,
  `- Alias commands: ${lifecycleStats.alias_of ?? 0}`,
  "",
  "## Lock Policy",
  "",
  `- Allowlisted lock-await exceptions: ${lockExceptionCount}`,
  `- Lock exception report: \`docs/reports/lock-await-exceptions.md\``,
  "",
  "## DORA Trend Snapshot",
  "",
  `- Lead time (hours): ${Number.isFinite(dora?.lead_time_hours) ? dora.lead_time_hours.toFixed(2) : "n/a"}`,
  `- Deployment frequency (per week): ${
    Number.isFinite(dora?.deployment_frequency_per_week)
      ? dora.deployment_frequency_per_week.toFixed(2)
      : "n/a"
  }`,
  `- Change fail rate: ${Number.isFinite(dora?.change_fail_rate) ? dora.change_fail_rate.toFixed(2) : "n/a"}`,
  `- MTTR (hours): ${Number.isFinite(dora?.mttr_hours) ? dora.mttr_hours.toFixed(2) : "n/a"}`,
  "",
  "## Review Checklist",
  "",
  "- [ ] Unresolved transitional adapters reviewed",
  "- [ ] New lock-await exceptions reviewed with owner + justification",
  "- [ ] Command lifecycle changes reviewed for removal policy compliance",
  "- [ ] Golden journey parity status reviewed",
];

mkdirSync("docs/reports", { recursive: true });
writeFileSync("docs/reports/weekly-architecture-checkpoint.md", `${lines.join("\n")}\n`);
console.log("Generated docs/reports/weekly-architecture-checkpoint.md");
