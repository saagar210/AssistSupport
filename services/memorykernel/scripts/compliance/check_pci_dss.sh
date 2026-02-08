#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
# shellcheck source=common.sh
source "$script_dir/common.sh"

memorykernel_root=$(parse_memorykernel_root "${1:-}")

run_step "ci enforces warning-free lint gate" require_grep "cargo clippy --workspace --all-targets --all-features -- -D warnings" "$memorykernel_root/.github/workflows/ci.yml"
run_step "ci enforces benchmark threshold triplet" require_grep "--append-p95-max-ms" "$memorykernel_root/.github/workflows/ci.yml"
run_step "trust-gate attachment fixture present" require_file "$memorykernel_root/contracts/integration/v1/fixtures/trust-gate-attachment.sample.json"
run_step "outcome compatibility artifact exists" require_file "$memorykernel_root/components/outcome-memory/trilogy-compatibility.v1.json"
run_step "multi-agent compatibility artifact exists" require_file "$memorykernel_root/components/multi-agent-center/trilogy-compatibility.v1.json"
