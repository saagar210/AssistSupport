#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: run_trilogy_compliance_suite.sh [options]

Run the monorepo trilogy compliance regression suite mapped to seven standards:
NIST SP 800-53, ISO 27001, SOC 2, HIPAA, PCI DSS, GDPR, FISMA/FedRAMP.

Options:
  --memorykernel-root <path> Path to monorepo root (default: script/..)
  --skip-baseline            Skip baseline runtime checks (parity/artifacts/smoke)
  -h, --help                 Show help
USAGE
}

resolve_path() {
  local path="$1"
  if [[ -d "$path" ]]; then
    (cd "$path" && pwd -P)
  else
    printf "%s\n" "$path"
  fi
}

memorykernel_root=""
skip_baseline=0

while (($# > 0)); do
  case "$1" in
    --memorykernel-root)
      memorykernel_root="${2:-}"
      shift 2
      ;;
    --skip-baseline)
      skip_baseline=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
if [[ -z "$memorykernel_root" ]]; then
  memorykernel_root=$(resolve_path "$script_dir/..")
else
  memorykernel_root=$(resolve_path "$memorykernel_root")
fi

if (( skip_baseline == 0 )); then
  echo "[baseline] contract parity"
  "$memorykernel_root/scripts/verify_contract_parity.sh" --canonical-root "$memorykernel_root"

  echo "[baseline] compatibility artifacts"
  "$memorykernel_root/scripts/verify_trilogy_compatibility_artifacts.sh" --memorykernel-root "$memorykernel_root"

  echo "[baseline] trilogy smoke"
  "$memorykernel_root/scripts/run_trilogy_smoke.sh" --memorykernel-root "$memorykernel_root"
fi

checks=(
  "check_nist_80053.sh"
  "check_iso27001.sh"
  "check_soc2.sh"
  "check_hipaa.sh"
  "check_pci_dss.sh"
  "check_gdpr.sh"
  "check_fisma_fedramp.sh"
)

for check_script in "${checks[@]}"; do
  echo "[suite] ${check_script}"
  "$memorykernel_root/scripts/compliance/${check_script}" "$memorykernel_root"
done

echo "Trilogy compliance suite passed."
