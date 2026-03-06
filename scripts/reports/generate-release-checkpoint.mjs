import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";

const readJson = (filePath) =>
  existsSync(filePath) ? JSON.parse(readFileSync(filePath, "utf8")) : null;

const scorecard = readJson(".perf-results/program-scorecard.json");
const dora = readJson(".perf-results/dora.json");
const lifecycle = readJson("contracts/tauri/v1/command-lifecycle.json");
const today = new Date().toISOString();

const lines = [
  "# Fortnightly Release Checkpoint",
  "",
  `Generated: ${today}`,
  "",
  "## Release Gate Summary",
  "",
  `- Program scorecard status: ${scorecard?.summary?.status ?? "unknown"}`,
  `- Failing metrics: ${
    scorecard?.summary?.failingMetrics?.length
      ? scorecard.summary.failingMetrics.join(", ")
      : "none"
  }`,
  `- Runtime metrics present: ${
    ["startup_total_ms", "generation_p95_ms", "search_p95_ms"].every(
      (metricId) =>
        scorecard?.metrics?.find((metric) => metric.id === metricId && metric.status !== "fail"),
    )
      ? "yes"
      : "no"
  }`,
  `- Command lifecycle entries: ${Object.keys(lifecycle?.commands ?? {}).length}`,
  "",
  "## DORA Snapshot",
  "",
  `- Lead time (hours): ${Number.isFinite(dora?.lead_time_hours) ? dora.lead_time_hours.toFixed(2) : "n/a"}`,
  `- Deploy frequency (per week): ${
    Number.isFinite(dora?.deployment_frequency_per_week)
      ? dora.deployment_frequency_per_week.toFixed(2)
      : "n/a"
  }`,
  `- Change fail rate: ${Number.isFinite(dora?.change_fail_rate) ? dora.change_fail_rate.toFixed(2) : "n/a"}`,
  `- MTTR (hours): ${Number.isFinite(dora?.mttr_hours) ? dora.mttr_hours.toFixed(2) : "n/a"}`,
  "",
  "## Required Review",
  "",
  "- [ ] Unresolved blockers triaged",
  "- [ ] Compatibility facade removals approved",
  "- [ ] Release readiness docs updated",
  "- [ ] Rollback checklist validated",
];

mkdirSync("docs/reports", { recursive: true });
writeFileSync("docs/reports/fortnightly-release-checkpoint.md", `${lines.join("\n")}\n`);
console.log("Generated docs/reports/fortnightly-release-checkpoint.md");
