#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
# shellcheck source=common.sh
source "$script_dir/common.sh"

memorykernel_root=$(parse_memorykernel_root "${1:-}")

run_step "trilogy closeout script present" require_file "$memorykernel_root/scripts/run_trilogy_phase_8_11_closeout.sh"
run_step "soak verification script present" require_file "$memorykernel_root/scripts/run_trilogy_soak.sh"
run_step "release gate references deterministic smoke requirement" require_grep "deterministic" "$memorykernel_root/docs/implementation/trilogy-release-gate.md"
run_step "outcome threshold semantics locked in artifact" require_json_expr "$memorykernel_root/components/outcome-memory/trilogy-compatibility.v1.json" '.benchmark_threshold_semantics.threshold_triplet_required == true and .benchmark_threshold_semantics.non_zero_exit_on_any_violation == true'
run_step "fedramp external authorization caveat documented" require_grep "authorization still requires external" "$memorykernel_root/docs/compliance/TRILOGY_CONTROL_MATRIX.md"
