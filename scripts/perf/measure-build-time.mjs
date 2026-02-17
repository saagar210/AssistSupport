import { spawnSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";

const start = Date.now();
const result = spawnSync("pnpm", ["build:ui"], {
  stdio: "inherit",
  shell: process.platform === "win32",
});
const end = Date.now();

mkdirSync(".perf-results", { recursive: true });
writeFileSync(
  ".perf-results/build-time.json",
  JSON.stringify(
    {
      buildMs: end - start,
      capturedAt: new Date().toISOString(),
      command: "pnpm build:ui",
    },
    null,
    2,
  ),
);

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}
