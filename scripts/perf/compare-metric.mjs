import { readFileSync } from "node:fs";

const [baselinePath, currentPath, metric, maxRatio] = process.argv.slice(2);

if (!baselinePath || !currentPath || !metric || !maxRatio) {
  console.error("Usage: node compare-metric.mjs <baselinePath> <currentPath> <metric> <maxRatio>");
  process.exit(1);
}

const baseline = JSON.parse(readFileSync(baselinePath, "utf8"));
const current = JSON.parse(readFileSync(currentPath, "utf8"));

const baselineValue = baseline[metric];
const currentValue = current[metric];

if (!Number.isFinite(baselineValue) || !Number.isFinite(currentValue)) {
  console.error(`Metric "${metric}" must be numeric in both baseline and current files.`);
  process.exit(1);
}

const ratio =
  baselineValue === 0 ? (currentValue > 0 ? 1 : 0) : (currentValue - baselineValue) / baselineValue;
const allowed = Number(maxRatio);

console.log(
  JSON.stringify(
    {
      metric,
      baseline: baselineValue,
      current: currentValue,
      ratio,
      threshold: allowed,
    },
    null,
    2,
  ),
);

if (ratio > allowed) {
  console.error(
    `Regression on ${metric}: ${(ratio * 100).toFixed(2)}% > ${(allowed * 100).toFixed(2)}%`,
  );
  process.exit(1);
}
