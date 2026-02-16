import { onCLS, onINP, onLCP, onTTFB, type Metric } from "web-vitals";

const report = (metric: Metric) => {
  console.log("[web-vitals]", metric.name, metric.value, metric.rating);
};

export function initWebVitals() {
  onCLS(report);
  onINP(report);
  onLCP(report);
  onTTFB(report);
}
