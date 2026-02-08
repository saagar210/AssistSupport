#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: generate_release_evidence_bundle.sh [options]

Generate a deterministic release-evidence bundle for MemoryKernel integration governance.

Options:
  --memorykernel-root <path> Path to MemoryKernel repo root (default: script/..)
  --out-json <path>          Output JSON path (default: docs/implementation/RELEASE_EVIDENCE_BUNDLE_LATEST.json)
  -h, --help                 Show this help
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
out_json=""

while (($# > 0)); do
  case "$1" in
    --memorykernel-root)
      memorykernel_root="${2:-}"
      shift 2
      ;;
    --out-json)
      out_json="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
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

if [[ -z "$out_json" ]]; then
  out_json="$memorykernel_root/docs/implementation/RELEASE_EVIDENCE_BUNDLE_LATEST.json"
elif [[ "$out_json" = /* ]]; then
  out_json="$(resolve_path "$(dirname "$out_json")")/$(basename "$out_json")"
else
  out_json="$memorykernel_root/$out_json"
fi

manifest="$memorykernel_root/contracts/integration/v1/producer-contract-manifest.json"
handoff="$memorykernel_root/docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json"
source_of_truth="$memorykernel_root/contracts/integration/v1/service-contract-source-of-truth.json"
slo_policy="$memorykernel_root/contracts/integration/v1/service-slo-policy.json"
openapi="$memorykernel_root/openapi/openapi.yaml"
phase_plan="$memorykernel_root/docs/implementation/REMAINING_ROADMAP_EXECUTION_PLAN_PRODUCER.md"
cutover_checklist="$memorykernel_root/docs/implementation/SERVICE_V3_CUTOVER_DAY_CHECKLIST.md"
rollback_protocol="$memorykernel_root/docs/implementation/SERVICE_V3_ROLLBACK_COMMUNICATION_PROTOCOL.md"

for path in \
  "$manifest" \
  "$handoff" \
  "$source_of_truth" \
  "$slo_policy" \
  "$openapi" \
  "$phase_plan" \
  "$cutover_checklist" \
  "$rollback_protocol"; do
  if [[ ! -f "$path" ]]; then
    echo "required file missing: $path" >&2
    exit 1
  fi
done

mkdir -p "$(dirname "$out_json")"

python3 - "$memorykernel_root" "$manifest" "$handoff" "$source_of_truth" "$slo_policy" "$openapi" "$phase_plan" "$cutover_checklist" "$rollback_protocol" "$out_json" <<'PY'
import datetime as dt
import hashlib
import json
import subprocess
import sys
from pathlib import Path

(
    root,
    manifest_path,
    handoff_path,
    source_path,
    slo_path,
    openapi_path,
    phase_plan_path,
    cutover_checklist_path,
    rollback_protocol_path,
    out_path,
) = sys.argv[1:11]

root_path = Path(root)
manifest = json.loads(Path(manifest_path).read_text(encoding="utf-8"))
handoff = json.loads(Path(handoff_path).read_text(encoding="utf-8"))
source = json.loads(Path(source_path).read_text(encoding="utf-8"))
slo = json.loads(Path(slo_path).read_text(encoding="utf-8"))


def rel(path: Path) -> str:
    return str(path.resolve().relative_to(root_path.resolve()))


def file_sha(path: Path) -> str:
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    return digest


def git_head(repo: Path) -> str:
    try:
        output = subprocess.check_output(
            ["git", "-C", str(repo), "rev-parse", "HEAD"],
            stderr=subprocess.DEVNULL,
            text=True,
        ).strip()
    except Exception:
        output = "unknown"
    return output

artifacts = [
    Path(manifest_path),
    Path(handoff_path),
    Path(source_path),
    Path(slo_path),
    Path(openapi_path),
    Path(phase_plan_path),
    Path(cutover_checklist_path),
    Path(rollback_protocol_path),
]

bundle = {
    "evidence_bundle_version": "release-evidence-bundle.v1",
    "generated_at_utc": dt.datetime.now(dt.timezone.utc).isoformat().replace("+00:00", "Z"),
    "generator": "scripts/generate_release_evidence_bundle.sh",
    "git_commit_head": git_head(root_path),
    "runtime_baseline": {
        "release_tag": manifest["release_tag"],
        "commit_sha": manifest["commit_sha"],
        "expected_service_contract_version": manifest["expected_service_contract_version"],
        "expected_api_contract_version": manifest["expected_api_contract_version"],
        "integration_baseline": manifest["integration_baseline"],
    },
    "policy_assertions": {
        "error_code_enum": manifest["error_code_enum"],
        "service_contract_source_version": source["service_contract_source_version"],
        "slo_policy_version": slo["slo_policy_version"],
    },
    "artifacts": [
        {
            "path": rel(path),
            "sha256": file_sha(path),
        }
        for path in artifacts
    ],
    "required_validation_commands": [
        "cargo fmt --all -- --check",
        "cargo clippy --workspace --all-targets --all-features -- -D warnings",
        "cargo test --workspace --all-targets --all-features",
        "./scripts/verify_service_contract_alignment.sh --memorykernel-root <repo>",
        "./scripts/verify_service_contract_source_of_truth.sh --memorykernel-root <repo>",
        "./scripts/verify_producer_contract_manifest.sh --memorykernel-root <repo>",
        "./scripts/verify_producer_handoff_payload.sh --memorykernel-root <repo>",
        "./scripts/verify_service_slo_policy.sh --memorykernel-root <repo>",
        "./scripts/verify_release_evidence_bundle.sh --memorykernel-root <repo>",
        "./scripts/run_trilogy_compliance_suite.sh --memorykernel-root <repo> --skip-baseline",
    ],
    "compliance_suite": {
        "standards": [
            "FISMA/FedRAMP High",
            "GDPR",
            "HIPAA",
            "ISO 27001",
            "NIST 800-53",
            "PCI DSS",
            "SOC 2",
        ],
        "entrypoint": "scripts/run_trilogy_compliance_suite.sh",
    },
    "handoff_mode": handoff.get("handoff_mode", "unknown"),
}

out = Path(out_path)
out.write_text(json.dumps(bundle, indent=2) + "\n", encoding="utf-8")
print(f"Wrote release evidence bundle: {out}")
PY
