import fs from "node:fs/promises";
import { performance } from "node:perf_hooks";

function parseDuration(raw) {
  if (!raw) {
    return 60_000;
  }

  const trimmed = raw.trim();
  const match = trimmed.match(/^(\d+)(ms|s|m)$/i);
  if (!match) {
    throw new Error(`Unsupported API_DURATION: ${raw}`);
  }

  const value = Number.parseInt(match[1], 10);
  const unit = match[2].toLowerCase();
  if (unit === "ms") {
    return value;
  }
  if (unit === "s") {
    return value * 1_000;
  }
  return value * 60_000;
}

function percentile(values, percentileRank) {
  if (values.length === 0) {
    return 0;
  }

  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.min(
    sorted.length - 1,
    Math.max(0, Math.ceil((percentileRank / 100) * sorted.length) - 1),
  );
  return sorted[index];
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

const baseUrl = (process.env.BASE_URL || "").replace(/\/+$/, "");
if (!baseUrl) {
  throw new Error("BASE_URL is required for perf:api");
}

const readyPath = process.env.API_READY_PATH || "/ready";
const searchPath = process.env.API_SEARCH_PATH || "/search";
const authToken = process.env.AUTH_TOKEN || "";
const durationMs = parseDuration(process.env.API_DURATION || "30s");
const vus = Number.parseInt(process.env.API_VUS || "1", 10);
const query = process.env.API_QUERY || "vpn access";
const topK = Number.parseInt(process.env.API_TOP_K || "5", 10);
const intervalMs = Number.parseInt(process.env.API_INTERVAL_MS || "500", 10);
const p95BudgetMs = Number.parseInt(process.env.API_P95_MS || "350", 10);
const p99BudgetMs = Number.parseInt(process.env.API_P99_MS || "700", 10);
const summaryPath = process.env.API_SUMMARY_PATH || ".perf-results/api-summary.json";

const requestHeaders = {
  "Content-Type": "application/json",
};

if (authToken) {
  requestHeaders.Authorization = `Bearer ${authToken}`;
}

const requestBody = JSON.stringify({
  query,
  top_k: Number.isFinite(topK) && topK > 0 ? topK : 5,
  fusion_strategy: "adaptive",
});

const readyResponse = await fetch(`${baseUrl}${readyPath}`);
if (!readyResponse.ok) {
  throw new Error(`Readiness check failed with status ${readyResponse.status}`);
}

const warmupResponse = await fetch(`${baseUrl}${searchPath}`, {
  method: "POST",
  headers: requestHeaders,
  body: requestBody,
});
if (!warmupResponse.ok) {
  throw new Error(`Warmup search failed with status ${warmupResponse.status}`);
}

const deadline = performance.now() + durationMs;
const latencies = [];
let failures = 0;
let checksFailed = 0;
let requestCount = 0;

async function worker() {
  while (performance.now() < deadline) {
    const startedAt = performance.now();
    let response;
    let statusOk = false;
    let bodyOk = false;

    try {
      response = await fetch(`${baseUrl}${searchPath}`, {
        method: "POST",
        headers: requestHeaders,
        body: requestBody,
      });

      statusOk = response.status === 200;
      let payload = null;
      try {
        payload = await response.json();
      } catch {
        payload = null;
      }
      bodyOk = payload?.status === "success";
    } catch {
      failures += 1;
      checksFailed += 1;
      requestCount += 1;
      continue;
    } finally {
      latencies.push(performance.now() - startedAt);
    }

    requestCount += 1;
    if (!statusOk) {
      failures += 1;
    }
    if (!(statusOk && bodyOk)) {
      checksFailed += 1;
    }

    await sleep(Number.isFinite(intervalMs) && intervalMs >= 0 ? intervalMs : 500);
  }
}

await Promise.all(Array.from({ length: Math.max(1, vus) }, () => worker()));

const p95 = percentile(latencies, 95);
const p99 = percentile(latencies, 99);
const failureRate = requestCount === 0 ? 1 : failures / requestCount;
const checksRate = requestCount === 0 ? 0 : (requestCount - checksFailed) / requestCount;

const summary = {
  baseUrl,
  searchPath,
  requestCount,
  metrics: {
    p95Ms: Number(p95.toFixed(2)),
    p99Ms: Number(p99.toFixed(2)),
    failureRate: Number(failureRate.toFixed(4)),
    checksRate: Number(checksRate.toFixed(4)),
  },
  budgets: {
    p95Ms: p95BudgetMs,
    p99Ms: p99BudgetMs,
    failureRate: 0.01,
    checksRate: 0.99,
  },
};

await fs.mkdir(".perf-results", { recursive: true });
await fs.writeFile(summaryPath, `${JSON.stringify(summary, null, 2)}\n`);

const failuresToReport = [];
if (requestCount === 0) {
  failuresToReport.push("No API requests completed during perf run");
}
if (failureRate >= 0.01) {
  failuresToReport.push(`http_req_failed rate ${failureRate.toFixed(4)} >= 0.01`);
}
if (checksRate <= 0.99) {
  failuresToReport.push(`checks rate ${checksRate.toFixed(4)} <= 0.99`);
}
if (p95 >= p95BudgetMs) {
  failuresToReport.push(`p95 ${p95.toFixed(2)}ms >= ${p95BudgetMs}ms`);
}
if (p99 >= p99BudgetMs) {
  failuresToReport.push(`p99 ${p99.toFixed(2)}ms >= ${p99BudgetMs}ms`);
}

if (failuresToReport.length > 0) {
  console.error("API performance gate failed:");
  for (const failure of failuresToReport) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log("API performance gate passed");
console.log(JSON.stringify(summary, null, 2));
