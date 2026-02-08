#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
# shellcheck source=common.sh
source "$script_dir/common.sh"

memorykernel_root=$(parse_memorykernel_root "${1:-}")

run_step "append-only behavior requirement documented" require_grep "append-only" "$memorykernel_root/docs/spec/requirements.md"
run_step "lineage requirements documented" require_grep "supersedes" "$memorykernel_root/docs/spec/requirements.md"
run_step "authority/confidence separation documented" require_grep "MKR-006" "$memorykernel_root/docs/spec/requirements.md"
run_step "context package explainability requirement documented" require_grep "selected items, excluded items" "$memorykernel_root/docs/spec/requirements.md"
run_step "operations recovery runbook present" require_file "$memorykernel_root/docs/operations/recovery-runbook.md"
