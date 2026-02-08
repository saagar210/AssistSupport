#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: verify_release_evidence_bundle.sh [options]

Validate release-evidence bundle structure and baseline consistency.

Options:
  --memorykernel-root <path> Path to MemoryKernel repo root (default: script/..)
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

while (($# > 0)); do
  case "$1" in
    --memorykernel-root)
      memorykernel_root="${2:-}"
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

manifest="$memorykernel_root/contracts/integration/v1/producer-contract-manifest.json"
published_bundle="$memorykernel_root/docs/implementation/RELEASE_EVIDENCE_BUNDLE_LATEST.json"
generator="$memorykernel_root/scripts/generate_release_evidence_bundle.sh"

for path in "$manifest" "$published_bundle" "$generator"; do
  if [[ ! -f "$path" ]]; then
    echo "required file missing: $path" >&2
    exit 1
  fi
done

tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT

generated_bundle="$tmp_dir/generated-release-evidence-bundle.json"
"$generator" --memorykernel-root "$memorykernel_root" --out-json "$generated_bundle" >/dev/null

echo "[check] release evidence bundle integrity"
python3 - "$manifest" "$published_bundle" "$generated_bundle" "$memorykernel_root" <<'PY'
import json
import sys
from pathlib import Path

manifest_path, published_path, generated_path, root = sys.argv[1:5]

manifest = json.loads(Path(manifest_path).read_text(encoding="utf-8"))
published = json.loads(Path(published_path).read_text(encoding="utf-8"))
generated = json.loads(Path(generated_path).read_text(encoding="utf-8"))
root_path = Path(root)

required_keys = {
    "evidence_bundle_version",
    "generated_at_utc",
    "generator",
    "runtime_baseline",
    "policy_assertions",
    "artifacts",
    "required_validation_commands",
    "compliance_suite",
    "handoff_mode",
}

for name, payload in (("published", published), ("generated", generated)):
    missing = sorted(required_keys - set(payload.keys()))
    if missing:
        raise SystemExit(f"{name} evidence bundle missing keys: {missing}")

for key in (
    "release_tag",
    "commit_sha",
    "expected_service_contract_version",
    "expected_api_contract_version",
    "integration_baseline",
):
    expected = manifest[key]
    if published["runtime_baseline"].get(key) != expected:
        raise SystemExit(f"published bundle runtime_baseline.{key} mismatch")
    if generated["runtime_baseline"].get(key) != expected:
        raise SystemExit(f"generated bundle runtime_baseline.{key} mismatch")

if published["policy_assertions"]["error_code_enum"] != manifest["error_code_enum"]:
    raise SystemExit("published bundle error_code_enum mismatch")

if generated["policy_assertions"]["error_code_enum"] != manifest["error_code_enum"]:
    raise SystemExit("generated bundle error_code_enum mismatch")

for entry in published["artifacts"]:
    path = entry.get("path")
    digest = entry.get("sha256")
    if not path or not digest:
        raise SystemExit("published bundle artifact entries require path and sha256")
    abs_path = (root_path / path).resolve()
    if not abs_path.exists():
        raise SystemExit(f"published bundle references missing artifact path: {path}")

print("Release evidence bundle checks passed.")
PY
