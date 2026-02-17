import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";

function nextBundle() {
  const manifestPath = ".next/build-manifest.json";
  if (!existsSync(manifestPath)) return null;

  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  const pages = manifest.pages || {};
  const pageSizes = {};

  for (const [route, files] of Object.entries(pages)) {
    let total = 0;
    for (const file of files) {
      const full = path.join(".next", file.replace(/^\/?/, ""));
      try {
        total += statSync(full).size;
      } catch {}
    }
    pageSizes[route] = total;
  }

  return {
    source: "next",
    totalBytes: Object.values(pageSizes).reduce((a, b) => a + b, 0),
    pages: pageSizes,
  };
}

function viteBundle() {
  const distAssets = "dist/assets";
  if (!existsSync(distAssets)) return null;

  const result = { source: "vite", totalBytes: 0, assets: {} };
  for (const file of readdirSync(distAssets)) {
    const full = path.join(distAssets, file);
    try {
      const size = statSync(full).size;
      result.assets[file] = size;
      result.totalBytes += size;
    } catch {}
  }
  return result;
}

const run = async () => {
  const report = nextBundle() || (await viteBundle()) || { source: "none", totalBytes: 0 };
  mkdirSync(".perf-results", { recursive: true });
  writeFileSync(
    ".perf-results/bundle.json",
    JSON.stringify({ ...report, capturedAt: new Date().toISOString() }, null, 2),
  );
};

run().catch((err) => {
  console.error(err);
  process.exit(1);
});
