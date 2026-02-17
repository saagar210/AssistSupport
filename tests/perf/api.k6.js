import http from "k6/http";
import { check, sleep } from "k6";

export const options = {
  vus: 20,
  duration: "1m",
  thresholds: {
    http_req_failed: ["rate<0.01"],
    http_req_duration: [`p(95)<${__ENV.API_P95_MS || 350}`, `p(99)<${__ENV.API_P99_MS || 700}`],
    checks: ["rate>0.99"],
  },
};

export default function run() {
  const target = `${__ENV.BASE_URL}/api/settings`;
  const res = http.get(target);
  check(res, { "status is 200": (r) => r.status === 200 });
  sleep(0.2);
}
