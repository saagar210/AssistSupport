#!/usr/bin/env bash
set -euo pipefail

ARTIFACT_PATH="${1:-trilogy-compatibility.v1.json}"

if [[ ! -f "${ARTIFACT_PATH}" ]]; then
  echo "error: missing compatibility artifact: ${ARTIFACT_PATH}" >&2
  exit 1
fi

jq -e '
  .artifact_version == "trilogy_compatibility.v1" and
  .project.name == "OutcomeMemory" and
  .project.version == "0.1.0" and
  .supported_memorykernel_contract_baseline == "integration/v1" and
  .required_stable_embed_api == ["run_cli", "run_outcome_with_db", "run_outcome", "run_benchmark"] and
  .benchmark_threshold_semantics.threshold_triplet_required == true and
  .benchmark_threshold_semantics.required_flags == ["--append-p95-max-ms", "--replay-p95-max-ms", "--gate-p95-max-ms"] and
  .benchmark_threshold_semantics.non_zero_exit_on_any_violation == true
' "${ARTIFACT_PATH}" >/dev/null

echo "trilogy compatibility artifact OK: ${ARTIFACT_PATH}"
