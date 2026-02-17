import { existsSync, mkdirSync, readdirSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";

const distAssetsDir = path.join("dist", "assets");
const extensions = new Set([
  ".js",
  ".css",
  ".mjs",
  ".map",
  ".woff",
  ".woff2",
  ".ttf",
  ".svg",
  ".json",
]);

const sizes = {};
let totalBytes = 0;

if (existsSync(distAssetsDir)) {
  for (const file of readdirSync(distAssetsDir)) {
    const ext = path.extname(file);
    if (!extensions.has(ext)) {
      continue;
    }

    const fullPath = path.join(distAssetsDir, file);
    const size = statSync(fullPath).size;
    sizes[file] = size;
    totalBytes += size;
  }
}

mkdirSync(".perf-results", { recursive: true });
writeFileSync(
  ".perf-results/bundle.json",
  JSON.stringify(
    {
      totalBytes,
      assets: sizes,
      capturedAt: new Date().toISOString(),
      source: "dist/assets",
    },
    null,
    2,
  ),
);
