#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
# shellcheck source=common.sh
source "$script_dir/common.sh"

memorykernel_root=$(parse_memorykernel_root "${1:-}")

run_step "ci workflow includes parity check" require_grep "verify_contract_parity.sh" "$memorykernel_root/.github/workflows/ci.yml"
run_step "ci workflow includes compatibility artifact check" require_grep "verify_trilogy_compatibility_artifacts.sh" "$memorykernel_root/.github/workflows/ci.yml"
run_step "ci workflow includes service contract alignment check" require_grep "verify_service_contract_alignment.sh" "$memorykernel_root/.github/workflows/ci.yml"
run_step "ci workflow includes producer manifest alignment check" require_grep "verify_producer_contract_manifest.sh" "$memorykernel_root/.github/workflows/ci.yml"
run_step "multi-agent trace architecture documented" require_file "$memorykernel_root/components/multi-agent-center/docs/architecture.md"
run_step "outcome integration handoff documented" require_file "$memorykernel_root/components/outcome-memory/docs/integration-handoff.md"
run_step "trilogy compatibility matrix documented" require_file "$memorykernel_root/docs/implementation/trilogy-compatibility-matrix.md"
