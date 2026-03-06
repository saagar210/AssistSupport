import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";

const RESULTS_DIR = ".perf-results";
const BASELINES_DIR = ".perf-baselines";

const runTimed = (command) => {
  const start = process.hrtime.bigint();
  const result = spawnSync("bash", ["-lc", command], {
    encoding: "utf8",
    stdio: "pipe",
    maxBuffer: 20 * 1024 * 1024,
  });
  const end = process.hrtime.bigint();
  const elapsedMs = Number(end - start) / 1_000_000;

  if (result.status !== 0) {
    throw new Error(
      [
        `Runtime probe command failed: ${command}`,
        `exit=${result.status}`,
        result.stdout?.trim() ? `stdout:\n${result.stdout}` : "",
        result.stderr?.trim() ? `stderr:\n${result.stderr}` : "",
      ]
        .filter(Boolean)
        .join("\n"),
    );
  }

  return elapsedMs;
};

const percentile = (values, pct) => {
  if (values.length === 0) return null;
  const sorted = [...values].sort((left, right) => left - right);
  const idx = Math.max(0, Math.ceil((pct / 100) * sorted.length) - 1);
  return sorted[idx];
};

const writeJson = (filePath, payload) => {
  writeFileSync(filePath, `${JSON.stringify(payload, null, 2)}\n`);
};

const bootstrapBaselineIfMissing = (name, payload) => {
  const baselinePath = `${BASELINES_DIR}/${name}.json`;
  if (existsSync(baselinePath)) return;

  writeJson(baselinePath, {
    ...payload,
    capturedAt: new Date().toISOString(),
    source: "bootstrap-from-runtime-probes",
  });
  console.log(`Bootstrapped missing baseline: ${baselinePath}`);
};

mkdirSync(RESULTS_DIR, { recursive: true });
mkdirSync(BASELINES_DIR, { recursive: true });

console.log("Running startup probe...");
const startupMs = runTimed("pnpm run check:workstation-preflight");
const startupPayload = {
  total_ms: Number(startupMs.toFixed(2)),
  probe: "pnpm run check:workstation-preflight",
  capturedAt: new Date().toISOString(),
};
writeJson(`${RESULTS_DIR}/startup.json`, startupPayload);
bootstrapBaselineIfMissing("startup", startupPayload);

const runP95Probe = ({ name, command, warmup = true, iterations = 3 }) => {
  if (warmup) {
    console.log(`Warm-up ${name} probe...`);
    runTimed(command);
  }

  console.log(`Collecting ${name} probe samples (${iterations} runs)...`);
  const samples = [];
  for (let i = 0; i < iterations; i += 1) {
    samples.push(Number(runTimed(command).toFixed(2)));
  }
  const p95 = percentile(samples, 95);
  const payload = {
    p95_ms: Number((p95 ?? 0).toFixed(2)),
    samples_ms: samples,
    probe: command,
    capturedAt: new Date().toISOString(),
  };

  writeJson(`${RESULTS_DIR}/${name}.json`, payload);
  bootstrapBaselineIfMissing(name, payload);
};

runP95Probe({
  name: "generation",
  command:
    "cd src-tauri && cargo test --test command_contract_fixtures generate_with_context_result_fixture_matches_contract -- --exact",
});

runP95Probe({
  name: "search",
  command:
    "cd src-tauri && cargo test --test command_contract_fixtures hybrid_search_response_fixture_matches_contract -- --exact",
});

const readJson = (filePath) =>
  existsSync(filePath) ? JSON.parse(readFileSync(filePath, "utf8")) : null;

const startup = readJson(`${RESULTS_DIR}/startup.json`);
const generation = readJson(`${RESULTS_DIR}/generation.json`);
const search = readJson(`${RESULTS_DIR}/search.json`);

if (!Number.isFinite(startup?.total_ms)) {
  throw new Error("Startup runtime metric capture failed; missing .perf-results/startup.json");
}
if (!Number.isFinite(generation?.p95_ms)) {
  throw new Error("Generation runtime metric capture failed; missing .perf-results/generation.json");
}
if (!Number.isFinite(search?.p95_ms)) {
  throw new Error("Search runtime metric capture failed; missing .perf-results/search.json");
}

console.log("Runtime probe capture completed.");
