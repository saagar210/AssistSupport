#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: run_trilogy_phase_8_11_closeout.sh [options]

Execute a deterministic closeout run for Phase 8-11 trilogy readiness.
This script always runs local trilogy gates and can optionally validate
hosted GitHub prerequisites/evidence when repository identifiers are supplied.

Options:
  --memorykernel-root <path>   Path to MemoryKernel repo root (default: script/..)
  --outcome-root <path>        Path to OutcomeMemory repo root (default: components/outcome-memory)
  --multi-agent-root <path>    Path to MultiAgentCenter repo root (default: components/multi-agent-center)
  --memorykernel-repo <owner/repo> Hosted MemoryKernel repo id for hosted run checks
  --outcome-repo <owner/repo>      Hosted OutcomeMemory repo id for hosted run checks
  --multi-agent-repo <owner/repo>  Hosted MultiAgentCenter repo id for hosted run checks
  --require-hosted             Fail if hosted repo checks are requested but missing/incomplete
  --skip-soak                  Skip soak step (default: run soak)
  --soak-iterations <n>        Iterations for soak step (default: 1)
  --report-out <path>          Markdown report path (default: docs/implementation/trilogy-closeout-report-latest.md)
  -h, --help                   Show help
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

escape_ticks() {
  local value="$1"
  printf "%s" "$value" | sed 's/`/\\`/g'
}

append_section() {
  local title="$1"
  {
    echo ""
    echo "## ${title}"
    echo ""
  } >>"$report_out"
}

run_step() {
  local label="$1"
  local command="$2"
  local log_file="$3"

  append_section "$label"
  {
    echo '```bash'
    echo "$command"
    echo '```'
    echo ""
  } >>"$report_out"

  if bash -lc "$command" >"$log_file" 2>&1; then
    echo "- Result: PASS" >>"$report_out"
    return 0
  fi

  echo "- Result: FAIL" >>"$report_out"
  {
    echo ""
    echo "<details><summary>Command output</summary>"
    echo ""
    echo '```text'
    cat "$log_file"
    echo '```'
    echo "</details>"
  } >>"$report_out"
  return 1
}

memorykernel_root=""
outcome_root=""
multi_agent_root=""
memorykernel_repo=""
outcome_repo=""
multi_agent_repo=""
require_hosted=0
skip_soak=0
soak_iterations=1
report_out=""

while (($# > 0)); do
  case "$1" in
    --memorykernel-root)
      memorykernel_root="${2:-}"
      shift 2
      ;;
    --outcome-root)
      outcome_root="${2:-}"
      shift 2
      ;;
    --multi-agent-root)
      multi_agent_root="${2:-}"
      shift 2
      ;;
    --memorykernel-repo)
      memorykernel_repo="${2:-}"
      shift 2
      ;;
    --outcome-repo)
      outcome_repo="${2:-}"
      shift 2
      ;;
    --multi-agent-repo)
      multi_agent_repo="${2:-}"
      shift 2
      ;;
    --require-hosted)
      require_hosted=1
      shift
      ;;
    --skip-soak)
      skip_soak=1
      shift
      ;;
    --soak-iterations)
      soak_iterations="${2:-}"
      shift 2
      ;;
    --report-out)
      report_out="${2:-}"
      shift 2
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

if ! [[ "$soak_iterations" =~ ^[0-9]+$ ]] || (( soak_iterations < 1 )); then
  echo "--soak-iterations must be a positive integer" >&2
  exit 2
fi

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
if [[ -z "$memorykernel_root" ]]; then
  memorykernel_root=$(resolve_path "$script_dir/..")
else
  memorykernel_root=$(resolve_path "$memorykernel_root")
fi
if [[ -z "$outcome_root" ]]; then
  if [[ -d "$memorykernel_root/components/outcome-memory" ]]; then
    outcome_root=$(resolve_path "$memorykernel_root/components/outcome-memory")
  else
    outcome_root=$(resolve_path "$memorykernel_root/../OutcomeMemory")
  fi
else
  outcome_root=$(resolve_path "$outcome_root")
fi
if [[ -z "$multi_agent_root" ]]; then
  if [[ -d "$memorykernel_root/components/multi-agent-center" ]]; then
    multi_agent_root=$(resolve_path "$memorykernel_root/components/multi-agent-center")
  else
    multi_agent_root=$(resolve_path "$memorykernel_root/../MultiAgentCenter")
  fi
else
  multi_agent_root=$(resolve_path "$multi_agent_root")
fi
if [[ -z "$report_out" ]]; then
  report_out="$memorykernel_root/docs/implementation/trilogy-closeout-report-latest.md"
fi

mkdir -p "$(dirname "$report_out")"
tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT

started_at=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
{
  echo "# Trilogy Phase 8-11 Closeout Report"
  echo ""
  echo "- Started (UTC): \`$started_at\`"
  echo "- MemoryKernel root: \`$(escape_ticks "$memorykernel_root")\`"
  echo "- OutcomeMemory root: \`$(escape_ticks "$outcome_root")\`"
  echo "- MultiAgentCenter root: \`$(escape_ticks "$multi_agent_root")\`"
  if [[ -n "$memorykernel_repo" ]]; then
    echo "- MemoryKernel hosted repo: \`$(escape_ticks "$memorykernel_repo")\`"
  fi
  if [[ -n "$outcome_repo" ]]; then
    echo "- OutcomeMemory hosted repo: \`$(escape_ticks "$outcome_repo")\`"
  fi
  if [[ -n "$multi_agent_repo" ]]; then
    echo "- MultiAgentCenter hosted repo: \`$(escape_ticks "$multi_agent_repo")\`"
  fi
  echo ""
  echo "## Local Gate Results"
  echo ""
  echo "## Hosted Evidence Checks"
  echo ""
  echo "## Closeout Summary"
  echo ""
} >"$report_out"

run_step "Contract Parity" \
  "$memorykernel_root/scripts/verify_contract_parity.sh --canonical-root '$memorykernel_root' --outcome-root '$outcome_root' --multi-agent-root '$multi_agent_root'" \
  "$tmp_dir/contract_parity.log"

run_step "Compatibility Artifact Validation" \
  "$memorykernel_root/scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root '$memorykernel_root' --outcome-root '$outcome_root' --multi-agent-root '$multi_agent_root'" \
  "$tmp_dir/artifact_validation.log"

run_step "Trilogy Smoke Gate" \
  "$memorykernel_root/scripts/run_trilogy_smoke.sh --memorykernel-root '$memorykernel_root' --outcome-root '$outcome_root' --multi-agent-root '$multi_agent_root'" \
  "$tmp_dir/smoke.log"

if (( skip_soak == 0 )); then
  run_step "Trilogy Soak Gate" \
    "$memorykernel_root/scripts/run_trilogy_soak.sh --memorykernel-root '$memorykernel_root' --outcome-root '$outcome_root' --multi-agent-root '$multi_agent_root' --iterations $soak_iterations" \
    "$tmp_dir/soak.log"
else
  append_section "Trilogy Soak Gate"
  echo "- Result: SKIPPED (requested via \`--skip-soak\`)" >>"$report_out"
fi

run_step "Rust Format" \
  "cargo fmt --manifest-path '$memorykernel_root/Cargo.toml' --all -- --check" \
  "$tmp_dir/fmt.log"

run_step "Rust Lint" \
  "cargo clippy --manifest-path '$memorykernel_root/Cargo.toml' --workspace --all-targets --all-features -- -D warnings" \
  "$tmp_dir/clippy.log"

run_step "Rust Test" \
  "cargo test --manifest-path '$memorykernel_root/Cargo.toml' --workspace --all-targets --all-features" \
  "$tmp_dir/test.log"

run_step "Outcome Benchmark Threshold Gate" \
  "cargo run --manifest-path '$memorykernel_root/Cargo.toml' -p memory-kernel-cli -- outcome benchmark run --volume 100 --volume 500 --volume 2000 --repetitions 3 --append-p95-max-ms 8 --replay-p95-max-ms 250 --gate-p95-max-ms 8 --json" \
  "$tmp_dir/benchmark.log"

run_step "Seven-Standard Compliance Suite" \
  "$memorykernel_root/scripts/run_trilogy_compliance_suite.sh --memorykernel-root '$memorykernel_root' --skip-baseline" \
  "$tmp_dir/compliance_suite.log"

append_section "Hosted Evidence Checks"
hosted_failures=0

if [[ -z "$outcome_repo" || -z "$multi_agent_repo" || -z "$memorykernel_repo" ]]; then
  {
    echo "- Result: SKIPPED (missing one or more hosted repo identifiers)"
    echo "- Required inputs: \`--memorykernel-repo\`, \`--outcome-repo\`, \`--multi-agent-repo\`."
  } >>"$report_out"
  if (( require_hosted == 1 )); then
    hosted_failures=1
  fi
else
  run_step "OutcomeMemory Variable Check" \
    "gh variable list -R '$outcome_repo' | awk '\$1 == \"MEMORYKERNEL_CANONICAL_REPO\" { print \$2 }' | rg -x '$memorykernel_repo'" \
    "$tmp_dir/outcome_var.log" || hosted_failures=1

  run_step "OutcomeMemory Smoke Workflow Success Check" \
    "count=\$(gh run list -R '$outcome_repo' --workflow smoke.yml --limit 20 --json status,conclusion --jq 'map(select(.status==\"completed\" and .conclusion==\"success\")) | length'); [[ \$count -gt 0 ]]" \
    "$tmp_dir/outcome_runs.log" || hosted_failures=1

  run_step "MultiAgentCenter Trilogy Guard Success Check" \
    "count=\$(gh run list -R '$multi_agent_repo' --workflow trilogy-guard.yml --limit 20 --json status,conclusion --jq 'map(select(.status==\"completed\" and .conclusion==\"success\")) | length'); [[ \$count -gt 0 ]]" \
    "$tmp_dir/multi_agent_runs.log" || hosted_failures=1

  run_step "MemoryKernel Release Workflow Success Check" \
    "count=\$(gh run list -R '$memorykernel_repo' --workflow release.yml --limit 20 --json status,conclusion --jq 'map(select(.status==\"completed\" and .conclusion==\"success\")) | length'); [[ \$count -gt 0 ]]" \
    "$tmp_dir/memorykernel_runs.log" || hosted_failures=1
fi

append_section "Closeout Summary"
finished_at=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
{
  echo "- Finished (UTC): \`$finished_at\`"
  echo "- Report path: \`$(escape_ticks "$report_out")\`"
} >>"$report_out"

if (( hosted_failures == 1 )); then
  {
    echo "- Hosted status: INCOMPLETE"
    echo "- Reason: One or more hosted checks failed or were required but unavailable."
  } >>"$report_out"
  echo "Closeout finished with hosted check failures. See $report_out"
  exit 1
fi

echo "- Hosted status: PASS or SKIPPED (not required)" >>"$report_out"
echo "Closeout completed successfully. Report: $report_out"
