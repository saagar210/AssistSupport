import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";

const readJson = (file) => (existsSync(file) ? JSON.parse(readFileSync(file, "utf8")) : null);

const summary = {
  generatedAt: new Date().toISOString(),
  bundle: readJson(".perf-results/bundle.json"),
  build: readJson(".perf-results/build-time.json"),
  memory: readJson(".perf-results/memory.json"),
  api: readJson(".perf-results/api-summary.json"),
};

mkdirSync(".perf-results", { recursive: true });
writeFileSync(".perf-results/summary.json", JSON.stringify(summary, null, 2));
