import { mkdirSync, writeFileSync } from "node:fs";

if (typeof global.gc !== "function") {
  console.error("Run with --expose-gc to enable memory smoke test.");
  process.exit(1);
}

const measureMb = () => process.memoryUsage().heapUsed / 1024 / 1024;

global.gc();
const beforeMb = measureMb();

const holder = [];
for (let i = 0; i < 50_000; i += 1) {
  holder.push({ i, value: `value-${i}` });
}

holder.length = 0;
global.gc();

const afterMb = measureMb();
const deltaMb = afterMb - beforeMb;
const thresholdMb = Number(process.env.MEMORY_MAX_DELTA_MB || 10);
const status = deltaMb > thresholdMb ? "warn" : "pass";

mkdirSync(".perf-results", { recursive: true });
writeFileSync(
  ".perf-results/memory.json",
  JSON.stringify(
    {
      beforeMb,
      afterMb,
      deltaMb,
      thresholdMb,
      status,
      capturedAt: new Date().toISOString(),
    },
    null,
    2,
  ),
);

if (deltaMb > thresholdMb) {
  console.error(`Memory growth exceeded threshold: ${deltaMb.toFixed(2)}MB > ${thresholdMb}MB`);
  process.exit(1);
}
