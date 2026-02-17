import http from "k6/http";
import { check, sleep } from "k6";

const p95 = Number(__ENV.API_P95_MS || 350);
const p99 = Number(__ENV.API_P99_MS || 700);

export const options = {
  vus: 20,
  duration: "1m",
  thresholds: {
    http_req_failed: ["rate<0.01"],
    http_req_duration: [`p(95)<${p95}`, `p(99)<${p99}`],
    checks: ["rate>0.99"],
  },
};

export default function () {
  const base = __ENV.BASE_URL || "http://localhost:3000";
  const res = http.get(`${base}/api/health`);
  check(res, { "status 200": (r) => r.status === 200 });
  sleep(0.2);
}
