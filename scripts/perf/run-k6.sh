#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: bash scripts/perf/run-k6.sh <script> [k6 args...]"
  exit 1
fi

script_path="$1"
shift

if [[ ! -f "$script_path" ]]; then
  echo "k6 script not found: $script_path"
  exit 1
fi

mkdir -p .perf-results

if command -v k6 >/dev/null 2>&1; then
  exec k6 run "$script_path" "$@"
fi

summary_path=""
for arg in "$@"; do
  if [[ "$arg" == --summary-export=* ]]; then
    summary_path="${arg#--summary-export=}"
    break
  fi
done

if command -v node >/dev/null 2>&1; then
  API_SUMMARY_PATH="${summary_path:-.perf-results/api-summary.json}" \
    exec node scripts/perf/api-load.mjs
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "k6 is not installed and Docker is unavailable. Install k6 or Docker to run perf:api."
  exit 1
fi

docker_base_url="${BASE_URL:-}"
if [[ -n "$docker_base_url" ]]; then
  docker_base_url="${docker_base_url/127.0.0.1/host.docker.internal}"
  docker_base_url="${docker_base_url/localhost/host.docker.internal}"
fi

exec docker run --rm \
  --add-host=host.docker.internal:host-gateway \
  -v "$PWD":/work \
  -w /work \
  -e BASE_URL="$docker_base_url" \
  -e AUTH_TOKEN="${AUTH_TOKEN:-}" \
  -e API_P95_MS="${API_P95_MS:-}" \
  -e API_P99_MS="${API_P99_MS:-}" \
  -e API_DURATION="${API_DURATION:-}" \
  -e API_VUS="${API_VUS:-}" \
  -e API_SLEEP_SECONDS="${API_SLEEP_SECONDS:-}" \
  -e API_INTERVAL_MS="${API_INTERVAL_MS:-}" \
  -e API_QUERY="${API_QUERY:-}" \
  -e API_TOP_K="${API_TOP_K:-}" \
  -e API_SEARCH_PATH="${API_SEARCH_PATH:-}" \
  -e API_READY_PATH="${API_READY_PATH:-}" \
  grafana/k6:0.49.0 \
  run "$script_path" "$@"
