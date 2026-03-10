import http from "k6/http";
import { check, sleep } from "k6";

const baseUrl = (__ENV.BASE_URL || "").replace(/\/+$/, "");
const authToken = __ENV.AUTH_TOKEN || "";
const searchPath = __ENV.API_SEARCH_PATH || "/search";
const readyPath = __ENV.API_READY_PATH || "/ready";
const searchQuery = __ENV.API_QUERY || "vpn access";
const topK = Number.parseInt(__ENV.API_TOP_K || "5", 10);
const sleepSeconds = Number.parseFloat(__ENV.API_SLEEP_SECONDS || "0.5");

export const options = {
  vus: Number.parseInt(__ENV.API_VUS || "1", 10),
  duration: __ENV.API_DURATION || "30s",
  thresholds: {
    http_req_failed: ["rate<0.01"],
    http_req_duration: [`p(95)<${__ENV.API_P95_MS || 350}`, `p(99)<${__ENV.API_P99_MS || 700}`],
    checks: ["rate>0.99"],
  },
};

export function setup() {
  if (!baseUrl) {
    throw new Error("BASE_URL is required for perf:api");
  }

  const readyResponse = http.get(`${baseUrl}${readyPath}`);
  check(readyResponse, {
    "ready status is 200": (response) => response.status === 200,
  });

  const headers = {
    "Content-Type": "application/json",
  };

  if (authToken) {
    headers.Authorization = `Bearer ${authToken}`;
  }

  const payload = JSON.stringify({
    query: searchQuery,
    top_k: Number.isFinite(topK) && topK > 0 ? topK : 5,
    fusion_strategy: "adaptive",
  });

  const warmupResponse = http.post(`${baseUrl}${searchPath}`, payload, { headers });
  check(warmupResponse, {
    "warmup status is 200": (response) => response.status === 200,
  });

  return {
    headers,
    payload,
  };
}

export default function run({ headers, payload }) {
  const res = http.post(`${baseUrl}${searchPath}`, payload, { headers });
  check(res, {
    "status is 200": (response) => response.status === 200,
    "response status is success": (response) => {
      if (response.status !== 200) {
        return false;
      }

      try {
        return response.json("status") === "success";
      } catch {
        return false;
      }
    },
  });
  sleep(Number.isFinite(sleepSeconds) && sleepSeconds >= 0 ? sleepSeconds : 0.5);
}
