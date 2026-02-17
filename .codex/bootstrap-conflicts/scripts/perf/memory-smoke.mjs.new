import { mkdirSync, writeFileSync } from "node:fs";

if (typeof globalThis.gc !== "function") {
  console.error("Run with --expose-gc.");
  process.exit(1);
}

globalThis.gc();
const before = process.memoryUsage().heapUsed;

const allocations = [];
for (let i = 0; i < 20_000; i += 1) {
  allocations.push({ i, payload: `row-${i}`.repeat(4) });
}

globalThis.gc();
const after = process.memoryUsage().heapUsed;
const deltaMb = Number(((after - before) / (1024 * 1024)).toFixed(2));
const maxDelta = Number(process.env.MEMORY_MAX_DELTA_MB || 10);

mkdirSync(".perf-results", { recursive: true });
writeFileSync(
  ".perf-results/memory.json",
  JSON.stringify(
    {
      heapBefore: before,
      heapAfter: after,
      deltaMb,
      maxDeltaMb: maxDelta,
      capturedAt: new Date().toISOString(),
    },
    null,
    2,
  ),
);

if (deltaMb > maxDelta) {
  console.error(`Memory growth too high: ${deltaMb}MB > ${maxDelta}MB`);
  process.exit(1);
}
