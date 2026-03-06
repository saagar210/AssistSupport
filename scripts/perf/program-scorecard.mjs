import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";

const readJson = (filePath) =>
  existsSync(filePath) ? JSON.parse(readFileSync(filePath, "utf8")) : null;

const ratioDelta = (baseline, current) => {
  if (!Number.isFinite(baseline) || !Number.isFinite(current)) return null;
  if (baseline === 0) return current === 0 ? 0 : 1;
  return (current - baseline) / baseline;
};

const compareMetric = ({ id, label, baseline, current, maxRatio, unit }) => {
  if (!Number.isFinite(baseline) || !Number.isFinite(current)) {
    return {
      id,
      label,
      status: "fail",
      unit,
      baseline,
      current,
      deltaRatio: null,
      thresholdRatio: maxRatio,
      note: "Metric unavailable; capture flow is mandatory for phase-exit.",
    };
  }

  const deltaRatio = ratioDelta(baseline, current);
  const status = deltaRatio !== null && deltaRatio <= maxRatio ? "pass" : "fail";
  return {
    id,
    label,
    status,
    unit,
    baseline,
    current,
    deltaRatio,
    thresholdRatio: maxRatio,
  };
};

const compareAbsolute = ({ id, label, value, maxValue, unit }) => {
  if (!Number.isFinite(value)) {
    return {
      id,
      label,
      status: "fail",
      unit,
      baseline: null,
      current: value,
      deltaRatio: null,
      thresholdRatio: null,
      note: "Metric unavailable; capture flow is mandatory for phase-exit.",
    };
  }

  return {
    id,
    label,
    status: value <= maxValue ? "pass" : "fail",
    unit,
    baseline: null,
    current: value,
    deltaRatio: null,
    thresholdRatio: null,
    thresholdAbsolute: maxValue,
  };
};

const bundleBaseline = readJson(".perf-baselines/bundle.json");
const bundleCurrent = readJson(".perf-results/bundle.json");
const buildBaseline = readJson(".perf-baselines/build-time.json");
const buildCurrent = readJson(".perf-results/build-time.json");
const memoryCurrent = readJson(".perf-results/memory.json");
const startupBaseline = readJson(".perf-baselines/startup.json");
const startupCurrent = readJson(".perf-results/startup.json");
const generationBaseline = readJson(".perf-baselines/generation.json");
const generationCurrent = readJson(".perf-results/generation.json");
const searchBaseline = readJson(".perf-baselines/search.json");
const searchCurrent = readJson(".perf-results/search.json");
const doraSnapshot = readJson(".perf-results/dora.json");

const metrics = [
  compareMetric({
    id: "bundle_total_bytes",
    label: "Bundle Size",
    baseline: bundleBaseline?.totalBytes,
    current: bundleCurrent?.totalBytes,
    maxRatio: 0.1,
    unit: "bytes",
  }),
  compareMetric({
    id: "build_ms",
    label: "Build Duration",
    baseline: buildBaseline?.buildMs,
    current: buildCurrent?.buildMs,
    maxRatio: 0.15,
    unit: "ms",
  }),
  compareAbsolute({
    id: "memory_delta_mb",
    label: "Memory Delta",
    value: memoryCurrent?.deltaMb,
    maxValue: Number.isFinite(memoryCurrent?.thresholdMb) ? memoryCurrent.thresholdMb : 10,
    unit: "MB",
  }),
  compareMetric({
    id: "startup_total_ms",
    label: "Startup Time",
    baseline: startupBaseline?.total_ms,
    current: startupCurrent?.total_ms,
    maxRatio: 0.2,
    unit: "ms",
  }),
  compareMetric({
    id: "generation_p95_ms",
    label: "Draft Generation p95",
    baseline: generationBaseline?.p95_ms,
    current: generationCurrent?.p95_ms,
    maxRatio: 0.15,
    unit: "ms",
  }),
  compareMetric({
    id: "search_p95_ms",
    label: "Search p95",
    baseline: searchBaseline?.p95_ms,
    current: searchCurrent?.p95_ms,
    maxRatio: 0.15,
    unit: "ms",
  }),
];

const dora = {
  lead_time_hours: Number.isFinite(doraSnapshot?.lead_time_hours)
    ? doraSnapshot.lead_time_hours
    : process.env.DORA_LEAD_TIME_HOURS
      ? Number.parseFloat(process.env.DORA_LEAD_TIME_HOURS)
      : null,
  deployment_frequency_per_week: Number.isFinite(doraSnapshot?.deployment_frequency_per_week)
    ? doraSnapshot.deployment_frequency_per_week
    : process.env.DORA_DEPLOY_FREQUENCY_PER_WEEK
      ? Number.parseFloat(process.env.DORA_DEPLOY_FREQUENCY_PER_WEEK)
      : null,
  change_fail_rate: Number.isFinite(doraSnapshot?.change_fail_rate)
    ? doraSnapshot.change_fail_rate
    : process.env.DORA_CHANGE_FAIL_RATE
      ? Number.parseFloat(process.env.DORA_CHANGE_FAIL_RATE)
      : null,
  mttr_hours: Number.isFinite(doraSnapshot?.mttr_hours)
    ? doraSnapshot.mttr_hours
    : process.env.DORA_MTTR_HOURS
      ? Number.parseFloat(process.env.DORA_MTTR_HOURS)
      : null,
  source: doraSnapshot?.source ?? "env-fallback",
};

const failed = metrics.filter((metric) => metric.status === "fail");
const scorecard = {
  generatedAt: new Date().toISOString(),
  summary: {
    status: failed.length > 0 ? "fail" : "pass",
    failingMetrics: failed.map((metric) => metric.id),
  },
  metrics,
  dora,
};

const ratioPct = (value) => (value === null ? "n/a" : `${(value * 100).toFixed(2)}%`);
const formatNumber = (value) => (Number.isFinite(value) ? String(value) : "n/a");

const markdownLines = [
  "# Program Scorecard",
  "",
  `Generated: ${scorecard.generatedAt}`,
  "",
  `Overall status: **${scorecard.summary.status.toUpperCase()}**`,
  "",
  "| Metric | Status | Baseline | Current | Delta | Threshold |",
  "|---|---|---:|---:|---:|---:|",
  ...metrics.map((metric) => {
    const threshold = Number.isFinite(metric.thresholdAbsolute)
      ? `<= ${metric.thresholdAbsolute} ${metric.unit}`
      : `<= ${ratioPct(metric.thresholdRatio)}`;
    return `| ${metric.label} | ${metric.status} | ${formatNumber(metric.baseline)} | ${formatNumber(metric.current)} | ${ratioPct(metric.deltaRatio)} | ${threshold} |`;
  }),
  "",
  "## DORA Snapshot",
  "",
  `- Lead time (hours): ${formatNumber(dora.lead_time_hours)}`,
  `- Deployment frequency (per week): ${formatNumber(dora.deployment_frequency_per_week)}`,
  `- Change fail rate: ${formatNumber(dora.change_fail_rate)}`,
  `- MTTR (hours): ${formatNumber(dora.mttr_hours)}`,
  `- Source: ${dora.source}`,
  "",
  "## Notes",
  "",
  "- Runtime metrics are mandatory for phase-exit; missing startup/generation/search captures fail this scorecard.",
  "- Run `pnpm perf:runtime` before `pnpm perf:scorecard` to refresh runtime artifacts.",
];

mkdirSync(".perf-results", { recursive: true });
writeFileSync(".perf-results/program-scorecard.json", JSON.stringify(scorecard, null, 2));
writeFileSync(".perf-results/program-scorecard.md", `${markdownLines.join("\n")}\n`);

console.log("Program scorecard generated at .perf-results/program-scorecard.{json,md}");

if (scorecard.summary.status === "fail") {
  console.error(
    `Program scorecard failed. Failing metrics: ${scorecard.summary.failingMetrics.join(", ") || "none listed"}`,
  );
  process.exit(1);
}
